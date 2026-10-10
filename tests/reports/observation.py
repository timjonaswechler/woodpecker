"""Failure observation and automatic reports through actual server sessions."""
import json

from tests.catalog import scenario
from tests.connection.protocol_faults.contracts import ended, session_events
from tests.session.runtime import until, process_gone


def report_events(session, count=1):
    def available():
        reports = [e for e in session.poll() if e["kind"] == "report"]
        assert len(reports) <= count, reports
        return reports if len(reports) == count else None
    return until(available, "Automatischer Report fehlt")


def saved_report(session, event):
    result = event["result"]
    assert result["status"] == "submitted" and result["outcome"]["kind"] == "created", event
    reference = result["outcome"]["reference"]
    assert reference["kind"] == "file", reference
    path = session.artifacts / reference["reference"]["path"]
    text = path.read_text()
    report = event["report"]
    assert report["title"] in text
    assert f"<!-- bug_hunter-signature: {report['signature']['value']} -->" in text
    context = report["context"]
    assert context["protocol_version"] == 3 and context["application"]["source"] is not None, context
    assert context["toolchain"]["cargo"].startswith("cargo "), context
    return path, text


@scenario("reports", "caught_worker_and_task_panics_keep_session_usable", capabilities=(),
          programs=("observation",), independent=True)
def caught_worker_and_task_panics_keep_session_usable(ctx):
    for mode, message in (("caught", "caught panic"), ("worker", "worker panic"),
                          ("unknown", None), ("task", "task panic")):
        with ctx.session(ctx.program("observation", mode)) as driver:
            root = driver.session.artifacts
            driver.command("tick.warp.start", {"ticks": 1})
            event = report_events(driver.session)[0]
            failure = event["report"]["failure"]
            assert failure["message"] == message, failure
            origin = failure["origin"]
            assert origin["kind"] == "panic" and origin["location"] is not None, origin
            assert origin["backtrace_status"] == "captured" and origin["backtrace"], origin
            assert (root / "prior-hook").read_text() == "called\n"
            path, text = saved_report(driver.session, event)
            snapshot = json.dumps(event, sort_keys=True)
            assert driver.session.detail()["state"] == "Ready"
            assert driver.command("tick.warp.start", {"ticks": 1})["executed_ticks"] == 1
            driver.close()
            driver.session.stop()
            assert path.read_text() == text
            assert [json.dumps(e, sort_keys=True) for e in report_events(driver.session)] == [snapshot]


@scenario("reports", "fatal_failures_precede_end_without_duplicate_exit", capabilities=(),
          programs=("observation",), independent=True)
def fatal_failures_precede_end_without_duplicate_exit(ctx):
    for mode in ("panic", "abort", "exit_zero", "corrupt_marker", "incomplete_marker"):
        with ctx.session(ctx.program("observation", mode)) as driver:
            request = driver.submit("tick.warp.start", {"ticks": 1})
            result = driver.outcome(request, "tick.warp.start")
            assert result["kind"] == "failed" and result["error"]["code"] == "ended", result
            events, terminal = ended(driver.session)
            failures = [e for e in events if e["kind"] == "failure"]
            assert len(failures) == 1, events
            failure = failures[0]["failure"]
            assert events.index(failures[0]) < events.index(terminal), events
            expected = "panic" if mode in ("panic", "abort") else "process_exit"
            assert failure["origin"]["kind"] == expected, failure
            errors = [e for e in events if e["kind"] == "observation_error"]
            assert len(errors) == int(mode in ("corrupt_marker", "incomplete_marker")), events
            if errors:
                assert errors[0]["code"] == "invalid_report_marker"
                assert events.index(errors[0]) < events.index(failures[0])
            event = report_events(driver.session)[0]
            assert event["report"]["failure"] == failure
            saved_report(driver.session, event)
            pid = int((driver.session.artifacts / "fixture-pid").read_text())
            until(lambda: process_gone(pid), "Fataler App-Prozess lebt weiter")


@scenario("reports", "tracing_requires_opt_in_and_obeys_filters", capabilities=(),
          programs=("observation",), independent=True)
def tracing_requires_opt_in_and_obeys_filters(ctx):
    for mode, enabled, message in (("trace", True, "observed trace"),
                                  ("fields", True, 'count=3, valid=true, text="Grüße"'),
                                  ("trace", False, None), ("filtered", True, None),
                                  ("handled", True, None)):
        with ctx.session(ctx.program("observation", mode, tracing=enabled)) as driver:
            driver.command("tick.warp.start", {"ticks": 1})
            if message:
                event = report_events(driver.session)[0]
                failure = event["report"]["failure"]
                assert failure["message"] == message, failure
                assert failure["origin"]["kind"] == "tracing_error", failure
                assert failure["origin"]["target"] == "fixture", failure
                saved_report(driver.session, event)
            driver.close()
            driver.session.stop()
            failures = [e for e in session_events(driver.session) if e["kind"] == "failure"]
            assert len(failures) == int(message is not None), failures
            assert len([e for e in driver.session.events if e["kind"] == "report"]) == int(message is not None)


@scenario("reports", "startup_failures_remain_launch_diagnostics", capabilities=(), interfaces=("json",),
          programs=("observation",), independent=True)
def startup_failures_remain_launch_diagnostics(ctx):
    for mode, tracing, message in (("pre_ready", False, "panic before Ready"),
                                   ("missing_layer", True, "layer")):
        with ctx.session(ctx.program("observation", mode, tracing=tracing), ready=False) as driver:
            detail = until(lambda: (d if (d := driver.session.detail())["state"] == "Failed" else None),
                           "Ungültiger App-Start wurde nicht abgelehnt", timeout=120)
            assert detail["error"]["code"] == "launch", detail
            assert message in detail["error"]["message"], detail
            assert not any(e["kind"] == "report" for e in driver.session.poll())
            pid = int((driver.session.artifacts / "fixture-pid").read_text())
            until(lambda: process_gone(pid), "Fehlgeschlagener Start hinterlässt App-Prozess")
    with ctx.session(ctx.program("observation", "missing_layer", tracing=False)) as driver:
        assert driver.session.detail()["state"] == "Ready"
        assert driver.command("tick.warp.start", {"ticks": 1})["executed_ticks"] == 1
