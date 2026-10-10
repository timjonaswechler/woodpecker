import json
import time

from tests.catalog import scenario
from tests.interfaces.json import check_result
from tests.recording.contracts import read_recording
from tests.session.runtime import until

START, STOP = "replay.start", "replay.stop"


@scenario("replay", "fresh_session_reproduces_effective_ticks", commands=(START, STOP))
def fresh_session_reproduces_effective_ticks(ctx):
    with ctx.session() as source:
        before = ctx.probe(source)
        source.command("recording.start", {"path": "recordings/source.jsonl"})
        requested = source.submit("tick.warp.start", {"ticks": 100000,
            "pace": {"kind": "ticks_per_second", "target": 10}})
        until(lambda: ctx.probe(source)["ticks"] > before["ticks"], "Aufgenommener Warp führt keine Ticks aus")
        source.command("tick.warp.stop", {})
        outcome = check_result(source.outcome(requested, "tick.warp.start"))
        assert outcome["outcome"] == "stopped" and 0 < outcome["executed_ticks"] < 100000, outcome
        source.command("recording.stop", {})
        path = source.session.artifacts / "recordings/source.jsonl"
        rows = read_recording(path)
        expected = ctx.probe(source)["ticks"] - before["ticks"]
        assert expected == outcome["executed_ticks"]
        assert rows[1]["arguments"]["ticks"] == 100000
        with ctx.session() as destination:
            initial = ctx.probe(destination)
            target = destination.session.artifacts / "source.jsonl"
            target.write_text(path.read_text())
            assert destination.command(START, {"path": "source.jsonl"}) == {
                "outcome": {"kind": "completed"}}
            assert ctx.probe(destination)["ticks"] == initial["ticks"] + expected
            assert destination.command(STOP, {}) == {"was_running": False}


@scenario("replay", "invalid_footer_prevents_valid_prefix", commands=(START,))
def invalid_footer_prevents_valid_prefix(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        path = driver.session.artifacts / "incomplete.jsonl"
        path.write_text("\n".join(json.dumps(row) for row in [
            {"type": "recording_started", "format_version": 1},
            {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 10},
             "outcome": {"status": "unanswered"}},
        ]) + "\n")
        driver.command(START, {"path": path.name}, expected_error="invalid_recording")
        assert ctx.probe(driver) == before


@scenario("replay", "stop_drains_active_warp", commands=(START, STOP))
def stop_drains_active_warp(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        path = driver.session.artifacts / "long.jsonl"
        path.write_text("\n".join(json.dumps(row) for row in [
            {"type": "recording_started", "format_version": 1},
            {"type": "command", "command": "tick.warp.start", "arguments": {
                "ticks": 100000, "pace": {"kind": "ticks_per_second", "target": 5}},
             "outcome": {"status": "unanswered"}},
            {"type": "recording_ended", "outcome": "session_ended", "recorded_commands": 1},
        ]) + "\n")
        pending = driver.submit(START, {"path": path.name})
        time.sleep(0.3)
        assert driver.command(STOP, {}) == {"was_running": True}
        assert check_result(driver.outcome(pending, START)) == {"outcome": {"kind": "stopped"}}
        after = ctx.probe(driver)
        assert before["ticks"] < after["ticks"] < before["ticks"] + 100000
        time.sleep(0.1)
        assert ctx.probe(driver) == after
