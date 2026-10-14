"""Syntax, scripts and terminal behavior have interface-specific contracts."""
import json

from tests.catalog import scenario


@scenario("json", "invalid_script_has_no_valid_prefix", interfaces=("json",))
def invalid_script_has_no_valid_prefix(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        path = ctx.server.directory / "invalid-script.json"
        path.write_text(json.dumps({"version": 1, "commands": [
            {"command": "tick.warp.start", "arguments": {"ticks": 10}},
            {"command": "inspect.query", "arguments": {"source": "unknown"}},
        ]}))
        ctx.server.cli("session", "script", driver.session.id, "--file", str(path), success=False)
        assert ctx.probe(driver) == before, "Gültiger Präfix eines ungültigen Scripts wurde ausgeführt"


@scenario("json", "script_pipelines_warp_and_stop", interfaces=("json",))
def script_pipelines_warp_and_stop(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        path = ctx.server.directory / "pipeline.json"
        path.write_text(json.dumps({"version": 1, "commands": [
            {"command": "tick.warp.start", "arguments": {
                "ticks": 100000, "pace": {"kind": "ticks_per_second", "target": 10}}},
            {"command": "tick.warp.stop", "arguments": {}},
        ]}))
        reply = ctx.server.cli("session", "script", driver.session.id, "--file", str(path))
        assert reply["kind"] == "passed", reply
        results = reply["completed"]
        assert [r["command_index"] for r in results] == [0, 1], reply
        assert len({r["request_id"] for r in results}) == 2, reply
        assert results[0]["output"]["outcome"] == "stopped", reply
        assert results[1]["output"] == {"was_running": True}, reply
        assert ctx.probe(driver)["ticks"] == before["ticks"] + results[0]["output"]["executed_ticks"]


@scenario("repl", "parse_errors_do_not_submit_or_advance", interfaces=("repl",))
def parse_errors_do_not_submit_or_advance(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        # Short forms validate locally; generic commands validate in the app.
        for line in ("tick warp -1", "tick warp pace 0", "inspect entity 1:2:3", "command {"):
            start = len(driver.lines)
            driver.send(line)
            driver.wait(lambda: any(l.startswith("input error:") for l in driver.lines[start:]),
                        f"REPL akzeptierte ungültige Eingabe: {line}")
            assert not any(" pending" in l for l in driver.lines[start:]), driver.lines[start:]
        assert ctx.probe(driver) == before


@scenario("repl", "quit_disconnects_without_stopping_session", interfaces=("repl",))
def quit_disconnects_without_stopping_session(ctx):
    with ctx.session() as driver:
        ctx.probe(driver)
        driver.send("quit")
        driver.process.wait(timeout=5)
        assert driver.process.returncode == 0
        assert driver.session.detail()["state"] == "SessionReady"


@scenario("repl", "terminal_editing_survives_external_activity", interfaces=("repl",))
def terminal_editing_survives_external_activity(ctx):
    from tests.interfaces.json import Json
    from tests.interfaces.terminal import Terminal
    with ctx.session() as driver:
        before = ctx.probe(driver)
        terminal = Terminal(driver.session)
        try:
            terminal.wait(f"connected {driver.session.id}")
            terminal.send("inspect resour")
            # An event from another client must not destroy partially edited input.
            observer = Json(driver.session)
            request = observer.submit("tick.warp.stop", {})
            terminal.wait(f'"request_id":{request}')
            terminal.send("ces\r")
            terminal.wait('"command":"inspect.query"')
            terminal.send(b"\x03")
            terminal.finish()
            assert driver.session.detail()["state"] == "SessionReady"
            assert ctx.probe(driver) == before
        finally:
            terminal.close()
