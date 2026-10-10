"""Replay uses current outcomes and stops before undispatched commands."""
import json

from tests.catalog import scenario
from tests.connection.protocol_faults.contracts import ended
from tests.recording.contracts import read_recording
from tests.interfaces.json import check_result
from tests.session.runtime import until, process_gone


def warp(ticks, outcome=None):
    return {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": ticks},
            "outcome": outcome or {"status": "unanswered"}}


def write_replay(root, name, commands):
    rows = [{"type": "recording_started", "format_version": 1}, *commands,
            {"type": "recording_ended", "outcome": "session_ended", "recorded_commands": len(commands)}]
    (root / name).write_text("\n".join(json.dumps(row) for row in rows) + "\n")


@scenario("replay", "effective_ticks_and_current_outcomes", capabilities=(),
          programs=("protocol",), independent=True)
def effective_ticks_and_current_outcomes(ctx):
    with ctx.session(ctx.program("protocol")) as driver:
        root = driver.session.artifacts
        write_replay(root, "source.jsonl", [warp(100, {"status": "completed", "output": {
            "requested_ticks": 100, "executed_ticks": 1, "outcome": "stopped"}}),
            {"type": "command", "command": "tick.warp.stop", "arguments": {},
             "outcome": {"status": "rejected", "error": {"code": "old", "message": "old"}}}])
        driver.command("recording.start", {"path": "current.jsonl"})
        assert driver.command("replay.start", {"path": "source.jsonl"}) == {"outcome": {"kind": "completed"}}
        stop = driver.submit("recording.stop", {})
        # Submit the replay before consuming Stop; it must wait for the footer.
        replay = driver.submit("replay.start", {"path": "current.jsonl"})
        assert check_result(driver.outcome(stop, "recording.stop"))["recorded_commands"] == 2
        assert check_result(driver.outcome(replay, "replay.start")) == {"outcome": {"kind": "completed"}}
        rows = read_recording(root / "current.jsonl")
        assert rows[1]["arguments"]["ticks"] == 1, rows
        assert rows[2]["outcome"] == {"status": "completed", "output": {"was_running": False}}, rows


@scenario("replay", "invalid_input_is_rejected_and_can_restart", capabilities=(),
          programs=("protocol",), independent=True)
def invalid_input_is_rejected_and_can_restart(ctx):
    with ctx.session(ctx.program("protocol")) as driver:
        root = driver.session.artifacts
        write_replay(root, "bad.jsonl", [warp(1), {"type": "unknown"}])
        for path, code in (("bad.jsonl", "invalid_recording"), ("missing.jsonl", "io"),
                           ("../escape.jsonl", "invalid_recording_path")):
            if code == "io":
                request = driver.submit("replay.start", {"path": path})
                result = driver.outcome(request, "replay.start")
                assert result["kind"] == "failed" and result["error"]["code"] == code, result
            else:
                driver.command("replay.start", {"path": path}, expected_error=code)
        assert not (root / "received.jsonl").exists(), "Ungültige Datei führte einen Präfix aus"
        write_replay(root, "valid.jsonl", [warp(1)])
        assert driver.command("replay.start", {"path": "valid.jsonl"}) == {"outcome": {"kind": "completed"}}


@scenario("replay", "exclusive_execution_and_stop_skips_tail", capabilities=(),
          programs=("protocol",), independent=True)
def exclusive_execution_and_stop_skips_tail(ctx):
    with ctx.session(ctx.program("protocol")) as driver:
        root = driver.session.artifacts
        write_replay(root, "long.jsonl", [warp(100), warp(1)])
        driver.command("recording.start", {"path": "executed.jsonl"})
        request = driver.submit("replay.start", {"path": "long.jsonl"})
        until(lambda: (root / "received.jsonl").is_file() and
              '"tick.warp.start"' in (root / "received.jsonl").read_text(),
              "Interner Warp wurde nicht gestartet")
        if ctx.interface == "repl":
            # The REPL rejects these locally while replay owns the session.
            driver.refuse("replay.start", {"path": "long.jsonl"}, "replay_in_progress")
            driver.refuse("tick.warp.stop", {}, "replay_in_progress")
            driver.refuse("shutdown", {}, "replay_in_progress")
        else:
            driver.command("replay.start", {"path": "long.jsonl"}, expected_error="replay_already_running")
            driver.command("tick.warp.stop", {}, expected_error="replay_in_progress")
            driver.refuse("shutdown", {}, "shutdown_commands_pending")
        assert driver.command("replay.stop", {}) == {"was_running": True}
        assert driver.outcome(request, "replay.start")["output"] == {"outcome": {"kind": "stopped"}}
        assert driver.command("replay.stop", {}) == {"was_running": False}
        driver.command("recording.stop", {})
        rows = read_recording(root / "executed.jsonl")
        warps = [r for r in rows[1:-1] if r["command"] == "tick.warp.start"]
        assert len(warps) == 1 and warps[0]["arguments"]["ticks"] == 100, rows


@scenario("replay", "technical_failure_blocks_and_exit_closes_recording", capabilities=(),
          programs=("protocol",), independent=True)
def technical_failure_blocks_and_exit_closes_recording(ctx):
    for mode in ("corrupt", "known_error", "wrong_name", "reject", "replay_exit"):
        with ctx.session(ctx.program("protocol", mode)) as driver:
            root = driver.session.artifacts
            write_replay(root, "source.jsonl", [warp(1), warp(7)])
            driver.command("recording.start", {"path": "result.jsonl"})
            result = driver.command("replay.start", {"path": "source.jsonl"})["outcome"]
            if mode == "reject":
                assert result == {"kind": "completed"}, result
            else:
                assert result["kind"] == "blocked", result
                assert result["code"] == ("session_ended" if mode == "replay_exit" else "command_protocol_failed"), result
            if mode == "replay_exit":
                ended(driver.session)
                rows = read_recording(root / "result.jsonl")
                assert rows[1] == warp(1) and rows[-1]["outcome"] == "session_ended", rows
                assert len(rows) == 3, rows
                pid = int((root / "pid").read_text())
                until(lambda: process_gone(pid), "Replay-Abbruch hinterlässt Prozess")
            else:
                driver.command("recording.stop", {})
