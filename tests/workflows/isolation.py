"""Keep real gameplay, input and persisted data isolated across live sessions."""
import hashlib
import json
import math

from tests.catalog import scenario
from tests.interfaces.json import check_result
from tests.recording.contracts import read_recording
from tests.session.application import ROOT
from tests.session.controls import BUTTONS, KEYS, MOTION
from tests.session.runtime import until
from tests.session.signalstation import RESOURCES, Signalstation
from tests.workflows.settings import SettingsUi

ARTIFACT = "recordings/isolation.jsonl"
CLOCK = {"kind": "fixed", "seconds_per_tick": 0.016666667}


class Peer:
    def __init__(self, driver, app, name):
        self.driver, self.name = driver, name
        self.game = Signalstation(driver, app)
        self.controls = SettingsUi(self.game)
        self.root = self.game.artifacts
        self.profile = self.root / "workspace/config/settings.json"

    def observe(self, label):
        state = self.game.snapshot(label)
        state["input"] = {"keys": self.game.resource(KEYS), "buttons": self.game.resource(BUTTONS),
                          "motion": self.game.resource(MOTION), "pointer": self.controls.pointer()}
        state["inventory"] = self.controls.inventory()
        state["profile_sha256"] = digest(self.profile)
        (self.root / f"checks/{label}.json").write_text(json.dumps(state, indent=2, ensure_ascii=False) + "\n")
        return state

    def unchanged(self, before, label):
        after = self.observe(label)
        changed = [key for key in before if before[key] != after[key]]
        assert not changed, f"Session {self.name} wurde fremd verändert: {changed}; checks/{label}.json"

    def ready(self, seed):
        engine = self.game.engine_ready()
        ready = self.game.wait_ready()
        assert self.controls.saved() == json.loads(seed)
        assert self.profile.read_bytes() == seed
        self.controls.assert_reset()
        assert self.controls.pointer() is None
        assert self.game.resource(MOTION)["delta"] == [0.0, 0.0]
        assert not any(self.controls.inventory().values())
        assert not any(self.game.resource("expedition")["parts"])
        assert self.root.name == self.driver.session.id
        assert all((self.root / "workspace" / name).is_dir() for name in ("config", "data", "tmp"))
        return {"engine": engine, "game": ready, "profile": str(self.profile),
                "snapshot": self.observe("isolation_ready")}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def reset_pair(a, b):
    for peer in (a, b):
        peer.controls.reset()


def separate_paths(*peers):
    for relative in (".", "workspace/config", "workspace/data", "workspace/tmp"):
        paths = [(peer.root / relative).resolve() for peer in peers]
        assert len(set(paths)) == len(paths), (relative, paths)
        assert len({(path.stat().st_dev, path.stat().st_ino) for path in paths}) == len(paths), paths
    assert len({peer.driver.session.id for peer in peers}) == len(peers)
    return {peer.name: str(peer.root) for peer in peers}


def keyboard_pointer(a, b):
    before_a, before_b = a.observe("before_inputs"), b.observe("before_inputs")
    a.controls.key("arrow_right")
    b.controls.key("arrow_up")
    a.controls.move_to([90, 70])
    b.controls.move_to([400, 250])
    a.unchanged(before_a, "queued_inputs")
    b.unchanged(before_b, "queued_inputs")

    a.game.advance(4)
    assert math.dist(a.game.resource("player")["position"], [-31.7, 22.0]) < 0.0001
    assert a.game.resource(KEYS)["pressed"] == ["ArrowRight"]
    assert a.controls.pointer() == [90.0, 70.0]
    b.unchanged(before_b, "after_other_advanced")
    a.controls.key("arrow_right", False)
    a.game.advance(1)
    paused_a = a.observe("released_inputs")

    b.game.advance(6)
    assert math.dist(b.game.resource("player")["position"], [-32.0, 21.55]) < 0.0001
    assert b.game.resource(KEYS)["pressed"] == ["ArrowUp"]
    assert b.controls.pointer() == [400.0, 250.0]
    b.controls.key("arrow_up", False)
    b.game.advance(1)
    a.unchanged(paused_a, "after_other_advanced")
    assert b.game.resource(KEYS)["pressed"] == []
    return {"A": {"position": a.game.resource("player")["position"], "pointer": a.controls.pointer()},
            "B": {"position": b.game.resource("player")["position"], "pointer": b.controls.pointer()}}


def fpv(a, b):
    before_b = b.observe("before_fpv")
    a.controls.tap("v")
    a.controls.captured()
    b.unchanged(before_b, "after_other_entered_fpv")
    assert b.game.resource("view")["mode"] != "FirstPerson"
    paused_a = a.observe("entered_fpv")
    b.controls.tap("v")
    b.controls.captured()
    a.unchanged(paused_a, "after_other_entered_fpv")

    # The same key and button may be held in both sessions without sharing state.
    a.controls.key("w")
    b.controls.key("w")
    a.controls.move_to([720, 450])
    b.controls.move_to([500, 300])
    a.controls.move_by([40, -20])
    b.controls.move_by([-80, 40])
    before_b = b.observe("queued_fpv")
    a.game.advance(1)
    assert abs(a.game.resource("player")["yaw"] + 0.1) < 0.00001
    assert abs(a.game.resource("view")["pitch"] - 0.05) < 0.00001
    assert a.controls.pointer() == [760.0, 430.0]
    assert a.game.resource(MOTION)["delta"] == [40.0, -20.0]
    b.unchanged(before_b, "after_other_turned")
    paused_a = a.observe("turned")
    b.game.advance(1)
    assert abs(b.game.resource("player")["yaw"] - 0.2) < 0.00001
    assert abs(b.game.resource("view")["pitch"] + 0.1) < 0.00001
    assert b.controls.pointer() == [420.0, 340.0]
    assert b.game.resource(MOTION)["delta"] == [-80.0, 40.0]
    a.unchanged(paused_a, "after_other_turned")
    a.controls.key("w", False)
    a.game.advance(1)
    assert a.game.resource(KEYS)["pressed"] == []
    assert b.game.resource(KEYS)["pressed"] == ["KeyW"]

    before_b = b.observe("before_shot")
    a.controls.button()
    a.game.advance(1)
    assert a.game.resource("practice")["shots"] == 1
    b.unchanged(before_b, "after_other_shot")
    assert b.game.resource("practice")["shots"] == 0
    paused_a = a.observe("after_shot")
    b.controls.button()
    b.game.advance(1)
    assert b.game.resource("practice")["shots"] == 1
    a.unchanged(paused_a, "after_other_shot")
    assert a.game.resource(BUTTONS)["pressed"] == b.game.resource(BUTTONS)["pressed"] == ["Left"]
    result = {peer.name: {"player": peer.game.resource("player"), "view": peer.game.resource("view"),
                         "camera": peer.controls.camera_pose(), "shots": peer.game.resource("practice")["shots"],
                         "cursor": peer.controls.cursor_free()} for peer in (a, b)}
    for peer in (a, b):
        peer.controls.release_all()
        peer.game.advance(1)
    return result


def progress(a, b):
    before_b = b.observe("before_pickup")
    route_a = a.controls.travel([-32.3, 0, -1])
    a.controls.face_station("Table part 0")
    picked_a = a.controls.interact()
    assert picked_a["expedition"]["parts"] == [True, False]
    assert picked_a["inventory"]["Bauteile"] == 1
    b.unchanged(before_b, "after_other_pickup")
    assert b.controls.inventory()["Bauteile"] == 0
    paused_a = a.observe("picked_up")

    # B must still be able to acquire its own copy of the very same object.
    route_b = b.controls.travel([-32.3, 0, -1])
    b.controls.face_station("Table part 0")
    picked_b = b.controls.interact()
    assert picked_b["expedition"]["parts"] == [True, False]
    assert picked_b["inventory"]["Bauteile"] == 1
    a.unchanged(paused_a, "after_other_pickup")
    paused_b = b.observe("picked_up")
    a.controls.reset()
    assert a.controls.inventory()["Bauteile"] == 0
    assert a.controls.progress()["parts"] == [False, False]
    b.unchanged(paused_b, "after_other_reset")
    return {"A": {"route": route_a, "picked_up": picked_a},
            "B": {"route": route_b, "picked_up": picked_b}, "reset_only_changed_A": True}


def concurrent_settings(a, b):
    seed = a.controls.saved()
    assert seed == b.controls.saved()
    for peer, other, button in ((a, b, "+"), (b, a, "−")):
        unchanged = other.profile.read_bytes()
        peer.controls.open_menu()
        peer.controls.click_label(button, row="Lauftempo    4.5 m/s")
        assert other.profile.read_bytes() == unchanged
        peer.controls.focus_field("Notizen")

    rounds = []
    for number in (1, 2):
        expected = {a.name: f"Session A · Grüße 世界🙂\nRunde {number}",
                    b.name: f"Session B · unabhängig Ω🚀\nRunde {number}"}
        for peer in (a, b):
            peer.controls.select_all()
            before = peer.controls.saved()
            peer.driver.command("input.text.input", {"text": expected[peer.name]})
            assert peer.controls.saved() == before, "Text wurde ohne Tick gespeichert"
        ticks = {peer.name: peer.game.tick_count() for peer in (a, b)}
        # Both app processes run their autosave updates while their warps overlap.
        pending = []
        try:
            for peer in (a, b):
                request = peer.driver.submit("tick.warp.start", {"ticks": 3,
                    "pace": {"kind": "ticks_per_second", "target": 1}})
                pending.append((peer, request))
            observed = {peer.name: peer.game.tick_count() for peer in (a, b)}
            assert all(observed[peer.name] < ticks[peer.name] + 3 for peer in (a, b)), observed
            for peer, request in pending:
                assert check_result(peer.driver.outcome(request, "tick.warp.start")) == {
                    "requested_ticks": 3, "executed_ticks": 3, "outcome": "completed"}
                peer.game.explicit_ticks += 3
        finally:
            for peer, _ in pending:
                peer.driver.command("tick.warp.stop", {})
        saved = {}
        for peer, speed in ((a, 5.0), (b, 4.0)):
            assert peer.game.tick_count() == ticks[peer.name] + 3
            saved[peer.name] = peer.controls.saved()
            assert saved[peer.name] == {**seed, "speed": speed, "notes": expected[peer.name]}
            assert not peer.profile.with_suffix(".json.tmp").exists()
        rounds.append({"round": number, "submitted_requests": {p.name: r for p, r in pending},
                       "ticks_at_overlap": observed, "saved": saved})
    for peer in (a, b):
        peer.controls.click_label("Zurück zum Spiel")
    return {"rounds": rounds, "workspaces": separate_paths(a, b),
            "temporary_settings_paths": [str(p.profile.with_suffix(".json.tmp")) for p in (a, b)]}


def command_errors(a, b):
    before_a, before_b = a.observe("before_rejections"), b.observe("before_rejections")
    a.driver.command("input.keyboard.press", {"key": "not-a-key"}, expected_error="invalid_key")
    a.driver.command("input.pointer.move_to", {"position": [-1, 0]},
                     expected_error="pointer_position_out_of_bounds")
    a.unchanged(before_a, "after_rejections")
    b.unchanged(before_b, "after_other_rejections")
    for peer in (a, b):
        peer.game.advance(1)
        peer.controls.assert_reset()
        assert peer.driver.session.detail()["state"] == "EngineReady"
    return {"rejections_in_A": ["invalid_key", "pointer_position_out_of_bounds"], "both_sessions_usable": True}


def save_artifact(peer):
    assert peer.driver.command("recording.start", {"path": ARTIFACT}) == {"path": ARTIFACT}
    settings = peer.game.resource("settings")
    assert peer.driver.command("recording.stop", {}) == {"path": ARTIFACT, "recorded_commands": 1}
    rows = read_recording(peer.root / ARTIFACT, clock=CLOCK)
    assert rows[-1]["outcome"] == "stopped"
    row = rows[1]
    assert row["command"] == "inspect.query"
    assert row["arguments"]["selector"] == {"kind": "type", "type_path": RESOURCES["settings"]}
    assert row["outcome"]["status"] == "completed"
    value = row["outcome"]["output"]["items"][0]["result"]["value"]
    assert value == {"status": "readable", "value": settings}
    return {"session": peer.driver.session.id, "root": str(peer.root),
            "files": {str(path): digest(path) for path in (peer.profile, peer.root / ARTIFACT)},
            "recorded_settings": settings}


def assert_retained(artifacts):
    from pathlib import Path
    for artifact in artifacts:
        for path, expected in artifact["files"].items():
            assert digest(Path(path)) == expected, f"Session-Artefakt verändert oder verloren: {path}"


def stop_one(a, b):
    before = b.game.tick_count()
    progress_before = b.controls.progress()
    settings_before = b.controls.saved()
    pending = b.driver.submit("tick.warp.start", {"ticks": 10000,
        "pace": {"kind": "ticks_per_second", "target": 5}})
    try:
        until(lambda: b.game.tick_count() > before, "Session B führt keine Ticks aus")
        a.driver.close()
        a.driver.session.stop()
        assert a.driver.session.detail()["state"] == "Ended"
        after_stop = b.game.tick_count()
        until(lambda: b.game.tick_count() > after_stop, "Stop von A hat B angehalten")
        assert b.driver.session.detail()["state"] == "EngineReady"
    finally:
        stopped = b.driver.command("tick.warp.stop", {})
        outcome = check_result(b.driver.outcome(pending, "tick.warp.start"))
    assert stopped == {"was_running": True}, stopped
    assert outcome["outcome"] == "stopped" and 0 < outcome["executed_ticks"] < 10000, outcome
    assert b.game.tick_count() - before == outcome["executed_ticks"]
    assert b.controls.progress() == progress_before and b.controls.saved() == settings_before
    b.game.advance(1)
    b.controls.assert_reset()
    return {"stopped_session": a.driver.session.id, "surviving_session": b.driver.session.id,
            "ticks_before_stop": before, "ticks_at_stop": after_stop, "surviving_warp": outcome}


@scenario("workflows", "signalstation_isolation", capabilities=("signalstation", "keyboard", "pointer", "text"),
          features=("connection",),
          commands=("inspect.query", "tick.warp.start", "tick.warp.stop", "input.keyboard.press",
                    "input.keyboard.release", "input.pointer.move_to", "input.pointer.move_by",
                    "input.pointer.press", "input.pointer.release", "input.text.input",
                    "recording.start", "recording.stop"))
def signalstation_isolation(ctx):
    personal = ROOT / "test_app/settings.json"
    original_personal = personal.read_bytes() if personal.exists() else None
    seed_path = ctx.app.path.parent / ctx.app.data["settings_seed"]
    seed = seed_path.read_bytes()
    artifacts, sessions = [], []
    with ctx.session() as first, ctx.session() as second:
        a, b = Peer(first, ctx.app, "A"), Peer(second, ctx.app, "B")
        sessions.extend((first.session, second.session))

        def ready():
            return {"A": a.ready(seed), "B": b.ready(seed), "paths": separate_paths(a, b)}

        ctx.steps.run("6.sessions_ready", ready)
        for name, check in (("keyboard_pointer", keyboard_pointer), ("fpv", fpv),
                            ("progress", progress), ("concurrent_settings", concurrent_settings),
                            ("command_errors", command_errors)):
            ctx.steps.run(f"6.{name}", lambda check=check: check(a, b), requires=("6.sessions_ready",),
                          precondition=lambda: reset_pair(a, b))

        def evidence():
            for peer in (a, b):
                artifacts.append(save_artifact(peer))
            assert artifacts[0]["recorded_settings"] != artifacts[1]["recorded_settings"]
            return artifacts.copy()

        ctx.steps.run("6.artifacts", evidence, requires=("6.concurrent_settings",))
        ctx.steps.run("6.stop_session", lambda: stop_one(a, b), requires=("6.sessions_ready",),
                      precondition=lambda: reset_pair(a, b))

        def fresh_session():
            before = b.observe("before_replacement")
            assert_retained(artifacts)
            with ctx.session() as replacement:
                c = Peer(replacement, ctx.app, "C")
                sessions.append(replacement.session)
                ready = c.ready(seed)
                paths = separate_paths(a, b, c)
                b.unchanged(before, "after_replacement_started")
                assert c.controls.saved() != b.controls.saved()
                artifacts.append(save_artifact(c))
                c.controls.key("arrow_right")
                c.game.advance(2)
                c.controls.key("arrow_right", False)
                c.game.advance(1)
                assert math.dist(c.game.resource("player")["position"], [-31.85, 22.0]) < 0.0001
                b.unchanged(before, "after_replacement_moved")
            b.unchanged(before, "after_replacement_stopped")
            assert b.driver.session.detail()["state"] == "EngineReady"
            b.game.advance(1)
            b.controls.assert_reset()
            assert_retained(artifacts)
            return {"ready": ready, "paths": paths, "survivor": b.driver.session.id,
                    "new_session_has_seed": True}

        ctx.steps.run("6.fresh_session", fresh_session, requires=("6.stop_session", "6.concurrent_settings"))

    def retained():
        assert len(artifacts) == len(sessions) == 3
        assert len({artifact["root"] for artifact in artifacts}) == 3
        assert_retained(artifacts)
        for session in sessions:
            assert session.detail()["state"] == "Ended"
        return {"artifacts": artifacts, "all_sessions_ended": True}

    ctx.steps.run("6.artifacts_retained", retained, requires=("6.artifacts", "6.fresh_session"))

    def unchanged_sources():
        assert (personal.read_bytes() if personal.exists() else None) == original_personal
        assert seed_path.read_bytes() == seed
        return {"personal_settings_unchanged": True, "seed_unchanged": True}

    ctx.steps.run("6.sources_unchanged", unchanged_sources)
