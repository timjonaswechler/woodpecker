"""Warp contracts observe application progress, not just command acknowledgements."""
import time

from tests.catalog import scenario
from tests.interfaces.json import check_result

START, PACE, STOP = "tick.warp.start", "tick.warp.set_pace", "tick.warp.stop"


@scenario("warp", "exact_ticks_and_idle", commands=(START,))
def exact_ticks_and_idle(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        assert before["ticks"] == 0, "Session hat vor dem ersten expliziten Warp bereits getickt"
        result = driver.command(START, {"ticks": 10})
        assert result == {"requested_ticks": 10, "executed_ticks": 10, "outcome": "completed"}, result
        after = ctx.probe(driver)
        assert after["ticks"] == before["ticks"] + 10, (before, after)
        time.sleep(0.1)
        assert ctx.probe(driver) == after, "Session läuft ohne expliziten Warp weiter"


@scenario("warp", "zero_ticks", commands=(START, STOP))
def zero_ticks(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        driver.command(START, {"ticks": 0}, expected_error="invalid_tick_count")
        assert driver.command(STOP, {}) == {"was_running": False}
        assert ctx.probe(driver) == before


@scenario("warp", "pace_change_and_stop", commands=(START, PACE, STOP))
def pace_change_and_stop(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        pending = driver.submit(START, {
            "ticks": 100000, "pace": {"kind": "ticks_per_second", "target": 5}})
        time.sleep(0.3)
        progressing = ctx.probe(driver)
        assert before["ticks"] < progressing["ticks"] < before["ticks"] + 100000, progressing
        changed = driver.command(PACE, {"pace": {"kind": "ticks_per_second", "target": 10}})
        assert changed == {"pace": {"kind": "ticks_per_second", "target": 10}}, changed
        assert driver.command(STOP, {}) == {"was_running": True}
        result = check_result(driver.outcome(pending, START))
        after = ctx.probe(driver)
        assert result["outcome"] == "stopped" and 0 < result["executed_ticks"] < 100000, result
        assert after["ticks"] - before["ticks"] == result["executed_ticks"], (result, after)
        time.sleep(0.1)
        assert ctx.probe(driver) == after
