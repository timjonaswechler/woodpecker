"""Faults cross a real server and the selected public command interface."""
from tests.catalog import scenario
from tests.interfaces.connection import Connection
from tests.session.runtime import until, process_gone


def session_events(session):
    return [e["event"] for e in session.poll() if e["kind"] == "event"]


def ended(session):
    until(lambda: session.detail()["state"] in ("Ended", "Failed"), "Fehlerprozess endet nicht")
    events = session_events(session)
    terminal = [e for e in events if e["kind"] == "ended"]
    assert len(terminal) == 1, events
    return events, terminal[0]


@scenario("connection", "protocol_failures_preserve_correlation", capabilities=(),
          programs=("protocol",), independent=True)
def protocol_failures_preserve_correlation(ctx):
    for mode, code in (("corrupt", "invalid_response"), ("wrong_name", "invalid_response"),
                       ("known_error", "fixture")):
        with ctx.session(ctx.program("protocol", mode)) as driver:
            request = driver.submit("tick.warp.stop", {})
            result = driver.outcome(request, "tick.warp.stop")
            assert result["kind"] == "failed" and result["error"]["code"] == code, result
            assert driver.session.detail()["state"] == "SessionReady"
            driver.session.stop()

    with ctx.session(ctx.program("protocol", "unsolicited")) as driver:
        assert driver.command("tick.warp.stop", {}) == {"was_running": False}
        errors = [e for e in session_events(driver.session) if e["kind"] == "protocol_error"]
        assert [e["code"] for e in errors] == ["fixture", "unexpected_ready"], errors
        assert driver.command("tick.warp.start", {"ticks": 1})["executed_ticks"] == 1


@scenario("connection", "full_pipes_and_exit_cleanup", capabilities=(),
          programs=("protocol",), independent=True)
def full_pipes_and_exit_cleanup(ctx):
    with ctx.session(ctx.program("protocol", "full_pipes")) as driver:
        pid = int((driver.session.artifacts / "pid").read_text())
        driver.session.pids.add(pid)
        assert driver.command("tick.warp.start", {"ticks": 1}) == {
            "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}

    with ctx.session(ctx.program("protocol", "exit")) as driver:
        root = driver.session.artifacts
        assert driver.command("tick.warp.stop", {}) == {"was_running": False}
        events, terminal = ended(driver.session)
        failures = [e for e in events if e["kind"] == "failure"]
        assert len(failures) == 1 and failures[0]["failure"]["origin"]["kind"] == "process_exit", events
        assert events.index(failures[0]) < events.index(terminal)
        for filename in ("pid", "descendant"):
            pid = int((root / filename).read_text())
            until(lambda: process_gone(pid), f"{filename} lebt nach Ende weiter")


@scenario("connection", "pipe_eof_has_explicit_terminal_reason", capabilities=(),
          programs=("protocol",), independent=True)
def pipe_eof_has_explicit_terminal_reason(ctx):
    for mode, channel in (("close_stderr", "stderr"), ("close_stdout", "stdout")):
        with ctx.session(ctx.program("protocol", mode)) as driver:
            root = driver.session.artifacts
            assert driver.command("tick.warp.stop", {}) == {"was_running": False}
            events, terminal = ended(driver.session)
            assert terminal["reason"] == {"kind": "transport_closed", "channel": channel}, events
            assert not any(e["kind"] == "failure" for e in events), events
            pid = int((root / "pid").read_text())
            until(lambda: process_gone(pid), "Prozess lebt trotz EOF weiter")


@scenario("connection", "launch_failure_cleans_up_process", capabilities=(), interfaces=("json",),
          programs=("protocol",), independent=True)
def launch_failure_cleans_up_process(ctx):
    with ctx.session(ctx.program("protocol", "wrong_version"), ready=False) as driver:
        detail = until(lambda: (d if (d := driver.session.detail())["state"] == "Failed" else None),
                       "Falsche Version wurde nicht abgelehnt", timeout=120)
        assert detail["error"]["code"] == "unsupported_protocol_version", detail
        pid = int((driver.session.artifacts / "pid").read_text())
        until(lambda: process_gone(pid), "Fehlgeschlagener Start hinterlässt Prozess")


@scenario("connection", "natural_exit_after_eof_keeps_exit_status", capabilities=(), interfaces=("json",),
          programs=("protocol",), independent=True)
def natural_exit_after_eof_keeps_exit_status(ctx):
    for mode in ("eof_then_exit", "stdout_eof_then_exit"):
        with ctx.session(ctx.program("protocol", mode)) as driver:
            root = driver.session.artifacts
            assert driver.command("tick.warp.stop", {}) == {"was_running": False}

            def refused():
                # EOF must have closed admission before the peer may exit naturally.
                with Connection(driver.session) as connection:
                    reply = connection.call({"kind": "submit", "command": {
                        "command": "tick.warp.stop", "arguments": {}}})
                if reply["kind"] == "pending":
                    return False
                assert reply["kind"] == "error" and reply["code"] == "ended", reply
                return True

            until(refused, "EOF hat die Annahme von Commands nicht beendet")
            (root / "allow-exit").write_text("exit now")
            events, terminal = ended(driver.session)
            failures = [e for e in events if e["kind"] == "failure"]
            assert len(failures) == 1, events
            assert failures[0]["failure"]["origin"] == {"kind": "process_exit", "status": "exit status: 0"}
            assert terminal["reason"] == {"kind": "process_exit", "status": "exit status: 0"}, events
