"""REPL syntax reaches the common commands; local errors never consume a request."""
import json
import os
from pathlib import Path
import subprocess
import time
import unittest

from tests._support import CLI, ROOT, start_server
from tests.recording import read_jsonl, write_jsonl
from tests.repl import ReplIO


class ReplCommandsTest(unittest.TestCase):
    def _completed(self, client, command, after):
        request_id = client.pending(command, after=after)
        event = client.terminal(request_id, after=after)
        self.assertEqual(event["kind"], "completed", event)
        return event["output"]

    def test_help_parse_errors_and_long_and_short_forms_share_the_public_command_contract(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, three explicit ticks, no game input. "
                         "Piped REPL help/pending, local malformed JSON/UTF-8, Inspect/Tick/"
                         "Recording/Replay forms and an empty replay. No OS input or retries. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "repl-commands")
        session_id = server.cli("session", "create", "--config", str(ROOT / "tests/app.toml"))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail("SessionReady deadline exceeded")
            time.sleep(0.05)
        root = Path(detail["artifact_dir"])
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 0})
        write_jsonl(root / "recordings/empty repl.jsonl", [
            {"type": "recording_started", "format_version": 2,
             "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}},
            {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 0},
        ])
        directory = server.artifacts / "client"
        directory.mkdir()
        stdout = self.enterContext((directory / "stdout.log").open("wb"))
        stderr = self.enterContext((directory / "stderr.log").open("wb"))
        process = subprocess.Popen([str(CLI), "--address", server.address, "session", "repl", session_id],
                                   cwd=ROOT, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr,
                                   start_new_session=True)
        client = ReplIO(process, directory)
        self.addCleanup(client.cleanup)
        client.wait(lambda text: f"connected {session_id} SessionReady" in text)
        local_cursor = server.cli("session", "poll", session_id)["cursor"]
        mark = client.mark()
        client.send(b" \nhelp\n")
        client.wait(lambda text: "quit, Ctrl+C and EOF disconnect only this client" in text, after=mark)
        help_text = client.text(mark)
        for spelling in ("command <command-json>", "tick warp <ticks>", "recording start <path>",
                         "replay start <path>", "inspect query <arguments-json>", "pending", "shutdown"):
            self.assertIn(spelling, help_text)
        mark = client.mark()
        client.send(b"help inspect\n")
        client.wait(lambda text: '"projection":{"kind":"value"}}\n' in text, after=mark)
        examples = [json.loads(line[len("inspect query "):]) for line in client.text(mark).splitlines()
                    if line.startswith("inspect query {")]
        self.assertEqual(len(examples), 6)
        self.assertEqual({example["projection"]["kind"] for example in examples},
                         {"summary", "component_names", "components", "hierarchy", "metadata", "value"})
        mark = client.mark()
        client.send(b"pending\n")
        client.wait(lambda text: "server pending: 0" in text, after=mark)
        local = server.cli("session", "poll", session_id, "--cursor", json.dumps(local_cursor))
        self.assertEqual(local["kind"], "activity")
        self.assertEqual(local["entries"], [], "Blank/help/pending must not allocate command IDs")
        mark = client.mark()
        client.send(b"recording start recordings/repl forms.jsonl\n")
        self.assertEqual(self._completed(client, "recording.start", mark), {"path": "recordings/repl forms.jsonl"})
        cursor = server.cli("session", "poll", session_id)["cursor"]
        invalid = ["invented", "command {", "command {\"command\":\"shutdown\",\"arguments\":{}} trailing",
                   "inspect query []", "inspect query {}", "inspect entity -1:0", "inspect entity 1:2:3",
                   "inspect entity 4294967296:0", "inspect resource", "tick warp -1", "tick warp 1 pace NaN",
                   "tick warp pace 0", "recording start", "replay start"]
        for line in invalid:
            mark = client.mark()
            client.send((line + "\n").encode())
            client.wait(lambda text: "input error:" in text, after=mark)
            self.assertIsNone(process.poll(), line)
        mark = client.mark()
        client.send(b"\xff\npending\n")
        client.wait(lambda text: "invalid UTF-8 input:" in text and "server pending: 0" in text, after=mark)
        activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
        self.assertEqual(activity["kind"], "activity")
        self.assertEqual(activity["entries"], [], "Local errors/help/pending must not allocate command IDs")

        settings_type = "test_app::app::settings::Settings"
        query = {"command": "inspect.query", "arguments": {"source": "resources",
                 "selector": {"kind": "type", "type_path": settings_type}, "projection": {"kind": "value"}}}
        unknown_type_path = "absent::Pair<Left, Right>"
        unknown_query = {"command": "inspect.query", "arguments": {"source": "resources",
                         "selector": {"kind": "type", "type_path": unknown_type_path},
                         "projection": {"kind": "value"}}}
        expected = []
        last_output = None
        for line, command in (
            ("command " + json.dumps(query), query),
            ("inspect query " + json.dumps(query["arguments"]), query),
            ("inspect resources", {"command": "inspect.query", "arguments": {"source": "resources",
                "selector": {"kind": "all"}, "projection": {"kind": "metadata"}}}),
            ("inspect resource " + settings_type, query),
            ("inspect resource   " + unknown_type_path + "  ", unknown_query),
            ("inspect entities", {"command": "inspect.query", "arguments": {"source": "entities",
                "entity": None, "with": [], "without": [], "projection": {"kind": "summary"}}}),
        ):
            mark = client.mark()
            client.send((line + "\n").encode())
            if command == unknown_query:
                # Valid REPL syntax preserves inner spaces, but the unknown type
                # must be rejected by Inspect rather than fabricate a value.
                request_id = client.pending("inspect.query", after=mark)
                outcome = client.terminal(request_id, after=mark)
                self.assertEqual(outcome, {"kind": "rejected", "request_id": request_id,
                    "command": "inspect.query", "error": {
                        "code": "unknown_type_path", "message": unknown_type_path}})
            else:
                last_output = self._completed(client, "inspect.query", mark)
                self.assertTrue(last_output["items"], line)
                if command == query:
                    self.assertEqual(last_output["items"][0]["result"]["value"]["value"],
                                     json.loads((ROOT / "tests/settings.json").read_text()))
            expected.append(command)
        handle = last_output["items"][0]["entity"]
        mark = client.mark()
        client.send(f"inspect entity {handle['index']}:{handle['generation']}\n".encode())
        self.assertEqual(self._completed(client, "inspect.query", mark)["items"][0]["entity"], handle)
        expected.append({"command": "inspect.query", "arguments": {"source": "entities", "entity": handle,
                         "with": [], "without": [], "projection": {"kind": "summary"}}})
        for line, arguments in (
            ("tick warp 1", {"ticks": 1}),
            ("tick warp 1 pace max", {"ticks": 1, "pace": {"kind": "as_fast_as_possible"}}),
            ("tick warp 1 pace 2", {"ticks": 1, "pace": {"kind": "ticks_per_second", "target": 2.0}}),
        ):
            mark = client.mark()
            client.send((line + "\n").encode())
            self.assertEqual(self._completed(client, "tick.warp.start", mark),
                             {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
            expected.append({"command": "tick.warp.start", "arguments": arguments})
        for line, pace in (("tick warp pace 2", {"kind": "ticks_per_second", "target": 2.0}),
                           ("tick warp pace max", {"kind": "as_fast_as_possible"})):
            mark = client.mark()
            client.send((line + "\n").encode())
            self.assertEqual(self._completed(client, "tick.warp.set_pace", mark), {"pace": pace})
            expected.append({"command": "tick.warp.set_pace", "arguments": {"pace": pace}})
        mark = client.mark()
        client.send(b"tick warp stop\n")
        self.assertEqual(self._completed(client, "tick.warp.stop", mark), {"was_running": False})
        expected.append({"command": "tick.warp.stop", "arguments": {}})
        mark = client.mark()
        client.send(b"recording stop\n")
        self.assertEqual(self._completed(client, "recording.stop", mark),
                         {"path": "recordings/repl forms.jsonl", "recorded_commands": 13})
        rows = read_jsonl(root / "recordings/repl forms.jsonl")
        self.assertEqual([{key: row[key] for key in ("command", "arguments")} for row in rows[1:-1]], expected)
        self.assertEqual(rows[1 + expected.index(unknown_query)]["outcome"], {
            "status": "rejected", "error": {
                "code": "unknown_type_path", "message": unknown_type_path}})
        mark = client.mark()
        client.send(b"replay start recordings/empty repl.jsonl\n")
        self.assertEqual(self._completed(client, "replay.start", mark), {"outcome": {"kind": "completed"}})
        mark = client.mark()
        client.send(b"replay stop\n")
        self.assertEqual(self._completed(client, "replay.stop", mark), {"was_running": False})
        client.send(b"quit\n")
        self.assertEqual(process.wait(timeout=5), 0)
        self.assertEqual((directory / "stderr.log").read_bytes(), b"")
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 3})
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
