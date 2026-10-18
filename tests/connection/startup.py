"""Launch diagnostics must reach the public interface and clean up the child."""
import time

from tests.catalog import scenario
from tests.connection.protocol_faults.contracts import session_events
from tests.session.runtime import process_gone, until


@scenario("connection", "engine_startup_failures_are_bounded", interfaces=("json",),
          programs=("protocol",), independent=True)
def engine_startup_failures_are_bounded(ctx):
    for mode, code, ticks in (("engine_failed", "startup_asset_failed", 3),
                              ("engine_stalled", "startup_timeout", None)):
        def check(mode=mode, code=code, ticks=ticks):
            config = ctx.program("protocol", mode)
            with config.open("a") as file:
                file.write('\n[startup]\nkind = "engine"\ntimeout_seconds = 1\nmax_ticks = 20\n')
            with ctx.session(config, ready=False) as driver:
                started = time.monotonic()
                detail = until(lambda: (d if (d := driver.session.detail())["state"] == "Failed" else None),
                               "Startfehler erreicht keinen terminalen Zustand", timeout=15)
                assert time.monotonic() - started < 15
                assert detail["error"]["code"] == code, detail
                events = session_events(driver.session)
                failures = [e for e in events if e["kind"] == "startup_failed"]
                assert len(failures) == 1, events
                assert failures[0]["ticks"] == ticks and failures[0]["error"]["code"] == code, failures
                assert not any(e["kind"] == "engine_ready" for e in events), events
                if ticks is not None:
                    assert "missing fixture asset" in detail["error"]["message"], detail
                assert not any(e["kind"] == "report" for e in driver.session.events)
                pid = int((driver.session.artifacts / "pid").read_text())
                until(lambda: process_gone(pid), "Startfehler hinterlässt Prozess")
                return {"error": detail["error"], "ticks": ticks}

        ctx.steps.run(f"7.{mode}", check)
