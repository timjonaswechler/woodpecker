import json

from tests.catalog import scenario
from tests.session.runtime import until


def observed_report(driver, message):
    def available():
        reports = [e for e in driver.session.poll() if e["kind"] == "report"]
        assert len(reports) <= 1, reports
        return reports[0] if reports else None
    event = until(available, "Automatischer Report fehlt")
    assert event["report"]["failure"]["message"] == message, event
    assert event["report"]["failure"]["origin"]["kind"] == "tracing_error", event
    assert event["result"]["status"] == "submitted", event
    assert event["result"]["outcome"]["kind"] == "created", event
    relative = event["result"]["outcome"]["reference"]["reference"]["path"]
    path = driver.session.artifacts / relative
    text = path.read_text()
    assert message in text and event["report"]["title"] in text
    assert f"<!-- bug_hunter-signature: {event['report']['signature']['value']} -->" in text
    return event, path, text


@scenario("reports", "automatic_failure_persists_after_stop",
          capabilities=("state", "keyboard", "reports"))
def automatic_failure_persists_after_stop(ctx):
    with ctx.session() as driver:
        driver.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(driver)
        assert before["failures"] == 0, before
        driver.command("input.keyboard.press", {"key": "b"})
        assert ctx.probe(driver) == before
        driver.command("tick.warp.start", {"ticks": 1})
        driver.command("input.keyboard.release", {"key": "b"})
        driver.command("tick.warp.start", {"ticks": 1})
        assert ctx.probe(driver)["failures"] == 1
        event, path, text = observed_report(driver, ctx.app.data["failure_message"])
        failures = [e["event"]["failure"] for e in driver.session.events
                    if e["kind"] == "event" and e["event"]["kind"] == "failure"]
        assert failures == [event["report"]["failure"]], failures
        commands = {row["command"]["command"] for row in event["report"]["context"]["commands"]}
        assert {"input.keyboard.press", "tick.warp.start"} <= commands, commands
        snapshot = json.dumps(event, sort_keys=True)
        driver.close()
        driver.session.stop()
        assert [json.dumps(e, sort_keys=True) for e in driver.session.poll()
                if e["kind"] == "report"] == [snapshot]
        assert path.read_text() == text
