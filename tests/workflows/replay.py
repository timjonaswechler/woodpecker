"""Record real Signalstation input, then replay it through the other interface."""
from copy import deepcopy
import json
import math
from pathlib import Path
import time

from tests.catalog import scenario
from tests.interfaces.json import check_result
from tests.interfaces.repl import Repl
from tests.recording.contracts import read_recording
from tests.session.application import ROOT
from tests.session.controls import BUTTONS, KEYS, Controls
from tests.session.runtime import Context, until
from tests.session.signalstation import NAME, RESOURCES, TRANSFORM, Signalstation, nanos
from tests.workflows.settings import SettingsUi

CLOCK = {"kind": "fixed", "seconds_per_tick": 0.016666667}
RECORDABLE = ("input.", "tick.", "inspect.", "screenshot.")
STATE = ("load", "player", "view", "capture", "flow", "expedition", "practice", "coins", "ui", "settings")
NOTES = "Replay · Grüße 世界🙂\nZweite Zeile"


class Transcript:
    """Associate checkpoints with commands sent through the real public driver."""
    def __init__(self, driver):
        self.driver, self.session = driver, driver.session
        self.calls = []

    def command(self, name, arguments, expected_error=None):
        result = self.driver.command(name, arguments, expected_error=expected_error)
        if name.startswith(RECORDABLE):
            self.calls.append((name, deepcopy(arguments)))
        return result


class ReplayControls(SettingsUi):
    def named(self, name, *types):
        # Recorded queries must resolve names in the new session, not carry
        # an entity selector cached from the source process.
        return Controls.named(self, name, *types)


def read_value(row, type_path):
    assert row["command"] == "inspect.query", row
    assert row["arguments"]["selector"] == {"kind": "type", "type_path": type_path}, row
    outcome = row["outcome"]
    assert outcome["status"] == "completed", outcome
    items = outcome["output"]["items"]
    assert len(items) == 1, items
    result = items[0]["result"]
    assert result["type_path"] == type_path and result["value"]["status"] == "readable", result
    return result["value"]["value"]


def checkpoint_values(rows, indices, origin):
    assert set(indices) == {*STATE, "time", "camera"}, indices
    values = {name: read_value(rows[index], RESOURCES[name])
              for name, index in indices.items() if name in STATE}
    clock = read_value(rows[indices["time"]], RESOURCES["time"])
    values["time"] = {"elapsed_nanos": nanos(clock["elapsed"]) - origin,
                      "delta_nanos": nanos(clock["delta"])}
    outcome = rows[indices["camera"]]["outcome"]
    assert outcome["status"] == "completed", outcome
    cameras = []
    for item in outcome["output"]["items"]:
        components = {c["component"]["type_path"]: c["value"] for c in item["result"]["components"]}
        if components[NAME] == {"status": "readable", "value": "Spielkamera"}:
            assert components[TRANSFORM]["status"] == "readable", components
            cameras.append(components[TRANSFORM]["value"])
    assert len(cameras) == 1, cameras
    values["camera"] = cameras[0]
    return values


def compare(expected, actual, path="checkpoint"):
    """Only f32 rounding is tolerated; missing fields or changed state fail."""
    assert type(expected) is type(actual), (path, expected, actual)
    if isinstance(expected, dict):
        assert expected.keys() == actual.keys(), (path, expected.keys(), actual.keys())
        for key in expected:
            compare(expected[key], actual[key], f"{path}.{key}")
    elif isinstance(expected, list):
        assert len(expected) == len(actual), (path, expected, actual)
        for index, (left, right) in enumerate(zip(expected, actual)):
            compare(left, right, f"{path}[{index}]")
    elif isinstance(expected, float):
        assert math.isclose(expected, actual, rel_tol=1e-6, abs_tol=1e-5), (path, expected, actual)
    else:
        assert expected == actual, (path, expected, actual)


def write_rows(path, rows):
    path.write_text("\n".join(json.dumps(row, ensure_ascii=False) for row in rows) + "\n")


def reset_baseline(game):
    controls = ReplayControls(game)
    controls.reset()
    # The overview camera normally eases toward its target. Leaving FPV places
    # it there immediately, through the same V shortcut a player uses. Reset
    # once more so expedition time also has the same origin in both sessions.
    controls.tap("v")
    controls.tap("v")
    controls.reset()


def prepare(game, warmup):
    engine = game.engine_ready()
    ready = game.wait_ready()
    controls = ReplayControls(game)
    seed = json.loads((game.application.path.parent / game.application.data["settings_seed"]).read_text())
    assert game.resource("settings") == seed
    game.advance(warmup)
    controls.move_to([720, 450])
    game.advance(1)
    reset_baseline(game)
    return {"engine": engine, "game": ready, "baseline_ticks": game.tick_count(),
            "settings": seed, "snapshot": game.snapshot("replay_baseline")}


def record(game):
    transcript = Transcript(game.driver)
    recorded = Signalstation(transcript, game.application)
    controls = ReplayControls(recorded)
    checkpoints = {}
    origin = game.tick_count() * game.application.data["tick_nanos"]
    path = "recordings/signalstation.jsonl"
    assert game.driver.command("recording.start", {"path": path}) == {"path": path}

    def checkpoint(name):
        indices = {}
        for key in (*STATE, "time"):
            indices[key] = len(transcript.calls)
            recorded.resource(key)
        indices["camera"] = len(transcript.calls)
        controls.named("Spielkamera", TRANSFORM)
        checkpoints[name] = indices

    checkpoint("baseline")
    controls.tap("v")
    controls.captured()
    controls.key("w")
    assert recorded.resource("player")["position"] == [-32.0, 22.0]
    checkpoint("queued_movement")
    recorded.advance(6)
    controls.key("w", False)
    recorded.advance(1)
    assert math.dist(recorded.resource("player")["position"], [-32, 21.55]) < 0.0001
    checkpoint("movement")

    controls.move_by([40, -20])
    recorded.advance(1)
    assert abs(recorded.resource("player")["yaw"] + 0.1) < 0.00001
    assert abs(recorded.resource("view")["pitch"] - 0.05) < 0.00001
    controls.click()
    practice = recorded.resource("practice")
    assert (practice["shots"], practice["hits"], practice["score"]) == (1, 0, 0), practice
    checkpoint("fpv_shot")

    controls.tap("m")
    assert recorded.resource("flow")["menu"]
    controls.focus_field("Notizen")
    transcript.command("input.text.input", {"text": NOTES})
    assert controls.saved()["notes"] == "", "Texteingabe wirkte vor dem Tick"
    checkpoint("queued_notes")
    recorded.advance(2)
    assert controls.saved()["notes"] == NOTES
    checkpoint("notes")
    controls.click_label("Zurück zum Spiel")
    controls.captured()
    recorded.advance(3, pace={"kind": "ticks_per_second", "target": 30})
    checkpoint("resumed")

    stopped = game.driver.command("recording.stop", {})
    rows = read_recording(game.artifacts / path, clock=CLOCK)
    assert stopped == {"path": path, "recorded_commands": len(transcript.calls)}, stopped
    commands = rows[1:-1]
    assert len(commands) == len(transcript.calls)
    for row, (name, arguments) in zip(commands, transcript.calls):
        assert row["command"] == name, (row, name)
        # Serialization adds optional default fields. Every submitted value must survive.
        assert all(row["arguments"][key] == value for key, value in arguments.items()), (row, arguments)
        assert row["outcome"]["status"] == "completed", row
        assert set(row) == {"type", "command", "arguments", "outcome"}, row
    assert rows[-1]["outcome"] == "stopped"
    effective = sum(row["outcome"]["output"]["executed_ticks"] for row in commands
                    if row["command"] == "tick.warp.start")
    assert effective == recorded.explicit_ticks == game.tick_count() - origin // game.application.data["tick_nanos"]
    expected = {name: checkpoint_values(commands, indices, origin) for name, indices in checkpoints.items()}
    assert expected["baseline"]["time"]["elapsed_nanos"] == 0
    assert expected["queued_movement"]["time"]["elapsed_nanos"] == 2 * game.application.data["tick_nanos"]
    assert expected["movement"]["time"]["elapsed_nanos"] == 9 * game.application.data["tick_nanos"]
    assert expected["resumed"]["time"]["elapsed_nanos"] == effective * game.application.data["tick_nanos"]
    (game.artifacts / "checks/replay-checkpoints.json").write_text(json.dumps(
        {"indices": checkpoints, "expected": expected}, indent=2, ensure_ascii=False) + "\n")
    frozen = (game.artifacts / path).read_bytes()
    game.driver.command("recording.stop", {}, expected_error="recording_not_active")
    game.advance(1)
    assert (game.artifacts / path).read_bytes() == frozen, "Aufnahme lief nach Stop weiter"
    return {"path": str(game.artifacts / path), "commands": len(commands), "ticks": effective,
            "origin_nanos": origin, "checkpoints": checkpoints, "expected": expected}


def stopped_recording(game):
    controls = ReplayControls(game)
    controls.restore_settings()
    controls.tap("v")
    before = game.tick_count()
    path = "recordings/stopped-warp.jsonl"
    driver = game.driver
    driver.command("recording.start", {"path": path})
    controls.key("w")
    request = driver.submit("tick.warp.start", {"ticks": 1000,
        "pace": {"kind": "ticks_per_second", "target": 5}})
    try:
        until(lambda: game.tick_count() > before, "Aufgenommener Warp führt keine Ticks aus")
        driver.command("recording.stop", {}, expected_error="recording_commands_pending")
    finally:
        assert driver.command("tick.warp.stop", {}) == {"was_running": True}
        result = check_result(driver.outcome(request, "tick.warp.start"))
        controls.key("w", False)
        game.advance(1)
    assert result["outcome"] == "stopped" and 0 < result["executed_ticks"] < 1000, result
    stopped = driver.command("recording.stop", {})
    rows = read_recording(game.artifacts / path, clock=CLOCK)
    assert stopped["recorded_commands"] == len(rows) - 2, stopped
    assert game.tick_count() - before == result["executed_ticks"] + 1
    assert math.dist(game.resource("player")["position"], [-32, 22 - result["executed_ticks"] * 0.075]) < 0.0001
    return {"path": str(game.artifacts / path), "warp": result, "ticks": game.tick_count() - before,
            "player": game.resource("player")}


def rejected_recordings(game, recording):
    rows = [json.loads(line) for line in Path(recording["path"]).read_text().splitlines()]
    wrong_count = deepcopy(rows)
    wrong_count[-1]["recorded_commands"] += 1
    wrong_clock = deepcopy(rows)
    wrong_clock[0]["clock"]["seconds_per_tick"] = 0.02
    bad_command = deepcopy(rows)
    bad_command.insert(-1, {"type": "command", "command": "input.keyboard.press",
                           "arguments": {"key": 42},
                           "outcome": {"status": "completed", "output": None}})
    bad_command[-1]["recorded_commands"] += 1
    before = game.snapshot("before_invalid_replays")
    rejected = []
    for name, data, error in (("missing-footer", rows[:-1], "invalid_recording"),
                              ("wrong-count", wrong_count, "invalid_recording"),
                              ("bad-tail", bad_command, "invalid_recording"),
                              ("wrong-clock", wrong_clock, "replay_clock_mismatch")):
        path = f"{name}.jsonl"
        write_rows(game.artifacts / path, data)
        game.driver.command("replay.start", {"path": path}, expected_error=error)
        game.unchanged(before, f"after_{name}")
        assert game.driver.command("replay.stop", {}) == {"was_running": False}
        rejected.append({"file": path, "error": error})
    # A queued input must not escape the validation barrier on the next tick either.
    game.advance(1)
    ReplayControls(game).assert_reset()
    assert game.resource("settings") == before["settings"]
    return {"rejected": rejected, "valid_prefix_executed": False}


def replay_checkpoints(game, recording):
    controls = ReplayControls(game)
    reset_baseline(game)
    origin = game.tick_count() * game.application.data["tick_nanos"]
    if origin == recording["origin_nanos"]:
        game.advance(1)
        reset_baseline(game)
        origin = game.tick_count() * game.application.data["tick_nanos"]
    assert origin != recording["origin_nanos"]
    target = game.artifacts / "source.jsonl"
    target.write_bytes(Path(recording["path"]).read_bytes())
    game.driver.command("recording.start", {"path": "replayed.jsonl"})
    assert game.driver.command("replay.start", {"path": target.name}) == {"outcome": {"kind": "completed"}}
    game.driver.command("recording.stop", {})
    rows = read_recording(game.artifacts / "replayed.jsonl", clock=CLOCK)[1:-1]
    source = read_recording(target, clock=CLOCK)[1:-1]
    assert len(rows) == len(source) == recording["commands"]
    for expected, actual in zip(source, rows):
        assert (actual["command"], actual["arguments"]) == (expected["command"], expected["arguments"]), (expected, actual)
        assert actual["outcome"]["status"] == "completed", actual
    actual = {name: checkpoint_values(rows, indices, origin)
              for name, indices in recording["checkpoints"].items()}
    compare(recording["expected"], actual)
    assert game.tick_count() * game.application.data["tick_nanos"] - origin == recording["ticks"] * game.application.data["tick_nanos"]
    assert controls.saved()["notes"] == NOTES
    assert game.resource(KEYS)["pressed"] == game.resource(BUTTONS)["pressed"] == []
    assert game.driver.command("replay.stop", {}) == {"was_running": False}
    return {"source_origin_nanos": recording["origin_nanos"], "destination_origin_nanos": origin,
            "ticks": recording["ticks"], "checkpoints": actual, "recording": "replayed.jsonl"}


def replay_effective_ticks(game, recording):
    controls = ReplayControls(game)
    controls.restore_settings()
    controls.tap("v")
    before = game.tick_count()
    (game.artifacts / "stopped-warp.jsonl").write_bytes(Path(recording["path"]).read_bytes())
    game.driver.command("recording.start", {"path": "effective-warp.jsonl"})
    assert game.driver.command("replay.start", {"path": "stopped-warp.jsonl"}) == {"outcome": {"kind": "completed"}}
    game.driver.command("recording.stop", {})
    rows = read_recording(game.artifacts / "effective-warp.jsonl", clock=CLOCK)[1:-1]
    warps = [row for row in rows if row["command"] == "tick.warp.start"]
    assert [row["arguments"]["ticks"] for row in warps] == [recording["warp"]["executed_ticks"], 1], warps
    assert not any(row["command"] == "tick.warp.stop" for row in rows), rows
    assert game.tick_count() == before + recording["ticks"]
    compare(recording["player"], game.resource("player"), "stopped_warp.player")
    return {"requested_ticks": 1000, "effective_ticks": recording["warp"]["executed_ticks"],
            "total_ticks": recording["ticks"], "player": game.resource("player")}


def stop_replay(game):
    controls = ReplayControls(game)
    controls.reset()
    # A deliberately long valid fixture leaves a visible tail that must not run.
    rows = [{"type": "recording_started", "format_version": 2, "clock": CLOCK},
            {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 1},
             "outcome": {"status": "unanswered"}},
            {"type": "command", "command": "tick.warp.start", "arguments": {
                "ticks": 10000, "pace": {"kind": "ticks_per_second", "target": 5}},
             "outcome": {"status": "unanswered"}},
            {"type": "command", "command": "input.keyboard.press", "arguments": {"key": "d"},
             "outcome": {"status": "unanswered"}},
            {"type": "recording_ended", "outcome": "session_ended", "recorded_commands": 3}]
    write_rows(game.artifacts / "long-replay.jsonl", rows)
    before = game.tick_count()
    driver = game.driver
    driver.command("recording.start", {"path": "interrupted-replay.jsonl"})
    request = driver.submit("replay.start", {"path": "long-replay.jsonl"})
    def first_tick_recorded():
        # Internal replay commands have no external Pending handle. The public
        # recording flushes their outcomes; wait for real execution, not a delay
        # guessed from loader speed. Only read fully written JSONL lines.
        lines = (game.artifacts / "interrupted-replay.jsonl").read_text().split("\n")[:-1]
        if len(lines) < 2:
            return False
        row = json.loads(lines[1])
        assert row["command"] == "tick.warp.start", row
        assert row["outcome"]["output"]["executed_ticks"] == 1, row
        return True
    try:
        until(first_tick_recorded, "Replay hat seinen ersten Tick nicht ausgeführt")
        time.sleep(0.3)
        if isinstance(driver, Repl):
            driver.refuse("input.keyboard.press", {"key": "w"}, "replay_in_progress")
        else:
            driver.command("input.keyboard.press", {"key": "w"}, expected_error="replay_in_progress")
    finally:
        stopped = driver.command("replay.stop", {})
        outcome = check_result(driver.outcome(request, "replay.start"))
        driver.command("recording.stop", {})
    assert stopped == {"was_running": True}, stopped
    assert outcome == {"outcome": {"kind": "stopped"}}, outcome
    rows = read_recording(game.artifacts / "interrupted-replay.jsonl", clock=CLOCK)[1:-1]
    assert [row["command"] for row in rows] == ["tick.warp.start", "tick.warp.start", "tick.warp.stop"], rows
    effective = rows[1]["outcome"]["output"]["executed_ticks"]
    assert rows[1]["outcome"]["output"]["outcome"] == "stopped"
    assert 0 <= effective < 10000 and game.tick_count() == before + 1 + effective, rows
    assert driver.command("replay.stop", {}) == {"was_running": False}
    after = game.snapshot("after_replay_stop")
    time.sleep(0.15)
    game.unchanged(after, "after_replay_stop_idle")
    game.advance(1)
    controls.assert_reset()
    return {"executed_ticks": 1 + effective, "interrupted_warp_ticks": effective,
            "tail_executed": False, "idle_after_stop": True}


@scenario("workflows", "signalstation_replay", capabilities=("signalstation", "keyboard", "pointer", "text"),
          features=("recording", "replay"),
          commands=("recording.start", "recording.stop", "replay.start", "replay.stop"))
def signalstation_replay(ctx):
    personal = ROOT / "test_app/settings.json"
    original = personal.read_bytes() if personal.exists() else None
    seed = ctx.app.path.parent / ctx.app.data["settings_seed"]
    original_seed = seed.read_bytes()
    with ctx.session() as source:
        game = Signalstation(source, ctx.app)
        ctx.steps.run("5.source_ready", lambda: prepare(game, 7))
        recording = ctx.steps.run("5.record", lambda: record(game), requires=("5.source_ready",))
        stopped = ctx.steps.run("5.recording_stop", lambda: stopped_recording(game), requires=("5.record",))
        source_id = source.session.id

    other_interface = "repl" if ctx.interface == "json" else "json"
    other = Context(ctx.server, ctx.app, ctx.mode, other_interface, ctx.steps)
    with other.session() as destination:
        assert destination.session.id != source_id
        game = Signalstation(destination, ctx.app)
        ctx.steps.run("5.destination_ready", lambda: prepare(game, 19))
        ctx.steps.run("5.reject_invalid", lambda: rejected_recordings(game, recording),
                      requires=("5.record", "5.destination_ready"))
        ctx.steps.run("5.replay_checkpoints", lambda: replay_checkpoints(game, recording),
                      requires=("5.record", "5.destination_ready"))
        ctx.steps.run("5.effective_warp", lambda: replay_effective_ticks(game, stopped),
                      requires=("5.recording_stop", "5.destination_ready"))
        ctx.steps.run("5.replay_stop", lambda: stop_replay(game), requires=("5.destination_ready",))

        def restored():
            controls = ReplayControls(game)
            controls.restore_settings()
            assert (personal.read_bytes() if personal.exists() else None) == original
            assert seed.read_bytes() == original_seed
            return {"settings": controls.saved(), "source_interface": ctx.interface,
                    "replay_interface": other_interface, "source_session": source_id,
                    "destination_session": destination.session.id,
                    "personal_settings_unchanged": True, "seed_unchanged": True}
        ctx.steps.run("5.restore", restored, requires=("5.destination_ready",))
