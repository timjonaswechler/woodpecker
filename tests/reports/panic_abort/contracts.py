"""Actual panic=abort build, observed through the server and public clients."""
from tests.catalog import scenario
from tests.connection.protocol_faults.contracts import ended
from tests.reports.observation import report_events, saved_report
from tests.session.runtime import until, process_gone


@scenario("reports", "panic_abort_reports_once_before_end", capabilities=(),
          programs=("abort",), independent=True)
def panic_abort_reports_once_before_end(ctx):
    with ctx.session(ctx.program("abort", "panic")) as driver:
        root = driver.session.artifacts
        request = driver.submit("tick.warp.start", {"ticks": 1})
        result = driver.outcome(request, "tick.warp.start")
        assert result["kind"] == "failed" and result["error"]["code"] == "ended", result
        events, terminal = ended(driver.session)
        failures = [e for e in events if e["kind"] == "failure"]
        assert len(failures) == 1 and events.index(failures[0]) < events.index(terminal), events
        failure = failures[0]["failure"]
        assert failure["message"] == "fixture panic\nsecond line", failure
        origin = failure["origin"]
        assert origin["kind"] == "panic" and origin["location"] is not None, origin
        assert origin["backtrace_status"] == "captured" and origin["backtrace"], origin
        assert "SIGABRT" in terminal["reason"]["status"], terminal
        assert (root / "prior-hook").read_text() == "called\n"
        event = report_events(driver.session)[0]
        assert event["report"]["failure"] == failure
        commands = event["report"]["context"]["commands"]
        assert len(commands) == 1 and commands[0]["outcome"]["status"] == "unanswered", commands
        saved_report(driver.session, event)
        pid = int((root / "fixture-pid").read_text())
        until(lambda: process_gone(pid), "Abort-Prozess lebt weiter")
