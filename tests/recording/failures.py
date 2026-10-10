"""Recording barriers and real transport/process failures."""
import os
import signal

from tests.catalog import scenario
from tests.connection.protocol_faults.contracts import ended
from tests.recording.contracts import read_recording
from tests.session.runtime import until, process_gone


@scenario("recording", "pending_barriers_and_submission_order", capabilities=(),
          programs=("protocol",), independent=True)
def pending_barriers_and_submission_order(ctx):
    with ctx.session(ctx.program("protocol")) as driver:
        root = driver.session.artifacts
        warp = driver.submit("tick.warp.start", {"ticks": 100})
        driver.command("recording.start", {"path": "pending.jsonl"}, expected_error="recording_commands_pending")
        assert not (root / "pending.jsonl").exists()
        driver.command("tick.warp.stop", {})
        driver.outcome(warp, "tick.warp.start")

        start = driver.submit("recording.start", {"path": "ordered.jsonl"})
        warp = driver.submit("tick.warp.start", {"ticks": 100})
        driver.outcome(start, "recording.start")
        driver.command("recording.stop", {}, expected_error="recording_commands_pending")
        driver.refuse("shutdown", {}, "shutdown_commands_pending")
        assert driver.command("tick.warp.stop", {}) == {"was_running": True}
        driver.outcome(warp, "tick.warp.start")
        driver.command("recording.start", {"path": "other.jsonl"}, expected_error="recording_already_active")
        driver.refuse("shutdown", {}, "shutdown_recording_active")
        assert driver.command("recording.stop", {}) == {"path": "ordered.jsonl", "recorded_commands": 2}
        rows = read_recording(root / "ordered.jsonl")
        assert [r["command"] for r in rows[1:-1]] == ["tick.warp.start", "tick.warp.stop"], rows
        assert rows[1]["outcome"]["output"]["executed_ticks"] == 0
        assert all("request_id" not in r for r in rows)
        driver.command("recording.start", {"path": "ordered.jsonl"}, expected_error="recording_path_exists")
        driver.command("recording.stop", {}, expected_error="recording_not_active")
        before_stop = (root / "ordered.jsonl").read_text()
        driver.close()
        driver.session.stop()
        assert (root / "ordered.jsonl").read_text() == before_stop


@scenario("recording", "protocol_failure_and_process_exit_are_preserved", capabilities=(),
          programs=("protocol",), independent=True)
def protocol_failure_and_process_exit_are_preserved(ctx):
    with ctx.session(ctx.program("protocol", "corrupt")) as driver:
        root = driver.session.artifacts
        driver.command("recording.start", {"path": "protocol.jsonl"})
        request = driver.submit("tick.warp.stop", {})
        result = driver.outcome(request, "tick.warp.stop")
        assert result["kind"] == "failed" and result["error"]["code"] == "invalid_response", result
        driver.command("recording.stop", {})
        rows = read_recording(root / "protocol.jsonl")
        assert rows[1]["outcome"]["status"] == "protocol_failed", rows

    with ctx.session(ctx.program("protocol")) as driver:
        root = driver.session.artifacts
        driver.command("recording.start", {"path": "ended.jsonl"})
        request = driver.submit("tick.warp.start", {"ticks": 100})
        pid = int((root / "pid").read_text())
        os.kill(pid, signal.SIGTERM)
        events, _ = ended(driver.session)
        assert len([e for e in events if e["kind"] == "failure"]) == 1, events
        result = driver.outcome(request, "tick.warp.start")
        assert result["kind"] == "failed" and result["error"]["code"] == "ended", result
        rows = read_recording(root / "ended.jsonl")
        assert rows[1]["outcome"] == {"status": "unanswered"}, rows
        assert rows[-1] == {"type": "recording_ended", "outcome": "session_ended", "recorded_commands": 1}
        until(lambda: process_gone(pid), "Getöteter Prozess wurde nicht aufgeräumt")
