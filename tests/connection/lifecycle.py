"""Client ownership, request correlation and orderly process cleanup."""
import time

from tests.catalog import scenario
from tests.connection.clock import counter
from tests.interfaces.json import Json, check_result
from tests.interfaces.repl import Repl
from tests.recording.contracts import read_recording
from tests.session.runtime import process_gone, until


@scenario("connection", "clients_and_shutdown", programs=("activity",), independent=True,
          features=("shutdown",), commands=("shutdown",))
def clients_and_shutdown(ctx):
    with ctx.session(ctx.program("activity")) as first, ctx.session(ctx.program("activity")) as second:
        a, b = counter(first), counter(second)
        assert first.session.id != second.session.id and a["process_id"] != b["process_id"]

        def correlation():
            request_a = first.submit("tick.warp.start", {"ticks": 3})
            request_b = second.submit("tick.warp.start", {"ticks": 7})
            assert request_a == request_b, (request_a, request_b)
            assert check_result(first.outcome(request_a, "tick.warp.start"))["executed_ticks"] == 3
            assert check_result(second.outcome(request_b, "tick.warp.start"))["executed_ticks"] == 7
            assert counter(first)["ticks"] == a["ticks"] + 3
            assert counter(second)["ticks"] == b["ticks"] + 7

        ctx.steps.run("7.request_correlation", correlation)
        other = counter(second)

        def reconnect():
            before = counter(first)
            pending = first.submit("tick.warp.start", {
                "ticks": 100000, "pace": {"kind": "ticks_per_second", "target": 10}})
            first.close()
            attached = Json(first.session) if ctx.interface == "json" else Repl(first.session)
            try:
                until(lambda: counter(attached)["ticks"] > before["ticks"],
                      "Akzeptierter Warp stoppt beim Client-Disconnect")
                attached.command("tick.warp.stop", {})
                result = check_result(attached.outcome(pending, "tick.warp.start"))
                assert result["outcome"] == "stopped" and 0 < result["executed_ticks"] < 100000, result
                after = counter(attached)
                assert after["ticks"] == before["ticks"] + result["executed_ticks"]
                time.sleep(0.1)
                assert counter(attached) == after
                assert counter(second) == other
            finally:
                attached.close()

        ctx.steps.run("7.disconnect_and_reattach", reconnect, requires=("7.request_correlation",))

        def stop_recording():
            # Reattach through the requested interface; stop is session management.
            attached = Json(first.session) if ctx.interface == "json" else Repl(first.session)
            root = first.session.artifacts
            try:
                attached.command("recording.start", {"path": "shutdown.jsonl"})
                attached.command("tick.warp.start", {"ticks": 2})
            finally:
                attached.close()
            first.session.stop()
            rows = read_recording(root / "shutdown.jsonl")
            assert rows[-1]["outcome"] == "stopped", rows
            assert [r["command"] for r in rows[1:-1]] == ["tick.warp.start", "tick.warp.stop"], rows
            assert rows[1]["outcome"]["output"]["executed_ticks"] == 2
            assert rows[2]["outcome"] == {"status": "completed", "output": {"was_running": False}}
            assert process_gone(a["process_id"])
            assert counter(second) == other
            return {"recording": str(root / "shutdown.jsonl")}

        ctx.steps.run("7.stop_closes_recording", stop_recording,
                      requires=("7.disconnect_and_reattach",))

        def shutdown():
            assert second.command("shutdown", {}) is None
            until(lambda: second.session.detail()["state"] == "Ended", "Shutdown beendet Session nicht")
            events = second.session.poll()
            ends = [e for e in events if e["kind"] == "lifecycle" and e["state"] == "Ended"]
            assert len(ends) == 1 and ends[0]["error"] is None, events
            assert not any(e["kind"] == "event" and e["event"]["kind"] in ("ended", "failure")
                           for e in events), events
            until(lambda: process_gone(b["process_id"]), "App lebt nach regulärem Shutdown weiter")

        ctx.steps.run("7.regular_shutdown", shutdown)
