from tests.catalog import scenario
from tests.session.runtime import process_gone, until


@scenario("shutdown", "terminal_completion_and_cleanup", commands=("shutdown",))
def terminal_completion_and_cleanup(ctx):
    with ctx.session() as driver:
        pid = ctx.probe(driver)["process_id"]
        assert driver.command("shutdown", {}) is None
        until(lambda: driver.session.detail()["state"] == "Ended", "Shutdown beendet Session nicht")
        until(lambda: process_gone(pid), "Shutdown lässt App-Prozess zurück")
        events = driver.session.poll()
        ends = [e for e in events if e["kind"] == "lifecycle" and e["state"] == "Ended"]
        assert len(ends) == 1 and ends[0]["error"] is None, events
        assert not any(e["kind"] == "event" and e["event"]["kind"] == "ended" for e in events), events
