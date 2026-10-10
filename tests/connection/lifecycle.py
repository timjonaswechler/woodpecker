import time

from tests.catalog import scenario
from tests.session.runtime import until
from tests.interfaces.json import Json, check_result
from tests.interfaces.repl import Repl


@scenario("connection", "session_isolation_and_correlation")
def session_isolation_and_correlation(ctx):
    with ctx.session() as first, ctx.session() as second:
        a, b = ctx.probe(first), ctx.probe(second)
        assert first.session.id != second.session.id and a["process_id"] != b["process_id"]
        # Identical request IDs in independent sessions must remain independent.
        request_a = first.submit("tick.warp.start", {"ticks": 3})
        request_b = second.submit("tick.warp.start", {"ticks": 7})
        assert request_a == request_b, (request_a, request_b)
        assert check_result(first.outcome(request_a, "tick.warp.start"))["executed_ticks"] == 3
        assert check_result(second.outcome(request_b, "tick.warp.start"))["executed_ticks"] == 7
        assert ctx.probe(first)["ticks"] == a["ticks"] + 3
        expected = ctx.probe(second)
        assert expected["ticks"] == b["ticks"] + 7
        first.close()
        first.session.stop()
        assert second.session.detail()["state"] == "Ready"
        assert ctx.probe(second) == expected


@scenario("connection", "accepted_work_survives_disconnect")
def accepted_work_survives_disconnect(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        pending = driver.submit("tick.warp.start", {
            "ticks": 100000, "pace": {"kind": "ticks_per_second", "target": 10}})
        driver.close()
        time.sleep(0.3)
        attached = Json(driver.session) if ctx.interface == "json" else Repl(driver.session)
        try:
            after = ctx.probe(attached)
            assert before["ticks"] < after["ticks"] < before["ticks"] + 100000
            attached.command("tick.warp.stop", {})
            assert check_result(attached.outcome(pending, "tick.warp.start"))["outcome"] == "stopped"
        finally:
            attached.close()
