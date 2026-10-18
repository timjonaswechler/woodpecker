"""Scripts and terminal behavior use real clients against the Bevy fixture."""
import json

from tests.catalog import scenario
from tests.connection.clock import counter
from tests.interfaces.json import Json
from tests.interfaces.terminal import Terminal


@scenario("json", "scripts_validate_and_pipeline", interfaces=("json",),
          programs=("activity",), independent=True)
def scripts_validate_and_pipeline(ctx):
    with ctx.session(ctx.program("activity")) as driver:
        def invalid_script():
            before = counter(driver)
            path = ctx.server.directory / "invalid-script.json"
            path.write_text(json.dumps({"version": 1, "commands": [
                {"command": "tick.warp.start", "arguments": {"ticks": 10}},
                {"command": "inspect.query", "arguments": {"source": "unknown"}},
            ]}))
            ctx.server.cli("session", "script", driver.session.id, "--file", str(path), success=False)
            assert counter(driver) == before, "Gültiger Präfix eines ungültigen Scripts wurde ausgeführt"

        ctx.steps.run("7.invalid_script", invalid_script)

        def pipeline():
            before = counter(driver)
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
            assert counter(driver)["ticks"] == before["ticks"] + results[0]["output"]["executed_ticks"]

        ctx.steps.run("7.script_pipeline", pipeline)


@scenario("repl", "parsing_terminal_and_disconnect", interfaces=("repl",),
          programs=("activity",), independent=True)
def parsing_terminal_and_disconnect(ctx):
    with ctx.session(ctx.program("activity")) as driver:
        before = counter(driver)

        def parse_errors():
            for line in ("tick warp -1", "tick warp pace 0", "inspect entity 1:2:3", "command {"):
                start = len(driver.lines)
                driver.send(line)
                driver.wait(lambda: any(l.startswith("input error:") for l in driver.lines[start:]),
                            f"REPL akzeptierte ungültige Eingabe: {line}")
                assert not any(" pending" in l for l in driver.lines[start:]), driver.lines[start:]
            assert counter(driver) == before

        ctx.steps.run("7.repl_parse_errors", parse_errors)

        def editing():
            terminal = Terminal(driver.session)
            try:
                terminal.wait(f"connected {driver.session.id}")
                terminal.send("inspect resour")
                observer = Json(driver.session)
                request = observer.submit("tick.warp.stop", {})
                terminal.wait(f'"request_id":{request}')
                terminal.send("ces\r")
                terminal.wait('"command":"inspect.query"')
                terminal.send(b"\x03")
                terminal.finish()
                assert driver.session.detail()["state"] == "SessionReady"
                assert counter(driver) == before
            finally:
                terminal.close()

        ctx.steps.run("7.terminal_editing", editing)

        def quit_repl():
            driver.send("quit")
            driver.process.wait(timeout=5)
            assert driver.process.returncode == 0
            assert driver.session.detail()["state"] == "SessionReady"
            assert counter(Json(driver.session)) == before

        ctx.steps.run("7.repl_quit", quit_repl)
