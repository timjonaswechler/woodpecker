"""Direct shutdown rejects open work/recording without admission or side effects."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import (load_config, owned_process, start_server,
                            wait_process_gone, write_config)
from tests.recording import read_jsonl
from tests.shutdown import no_failures


class ShutdownCommandTest(unittest.TestCase):
    def test_preconditions_do_not_allocate_shutdown_id_and_final_shutdown_is_terminal(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual game window, one 120-tick Warp at 2/s stopped early. "
                         "Two intentionally blocked Shutdowns, read-only queries, Recording Stop "
                         "then normal Shutdown; no inputs. Permission required: WOODPECKER_TEST_RENDERED=1")
        server = start_server(self, "shutdown-command")
        configuration = load_config()
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
        config = server.artifacts / "config.toml"
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            if time.monotonic() >= deadline:
                self.fail("SessionReady deadline")
            time.sleep(0.03)
        root = Path(detail["artifact_dir"])
        process = owned_process(server, root)
        counter = "test_app::woodpecker::SimulationTicks"
        self.assertEqual(server.read_resource(session_id, counter), {"ticks": 0})
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/shutdown.jsonl"}})
        cursor = server.cli("session", "poll", session_id)["cursor"]
        warp = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {"ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}}}))
        rejected = server.run_cli("session", "submit", session_id, "--command", '{"command":"shutdown","arguments":{}}')
        self.assertEqual(rejected.returncode, 1)
        error = json.loads(rejected.stdout)
        self.assertEqual(error["kind"], "error")
        self.assertEqual(error["code"], "shutdown_commands_pending")
        self.assertNotIn("request_id", error)
        before_stop = server.read_resource(session_id, counter)["ticks"]
        self.assertLess(before_stop, 120)
        self.assertEqual(server.call(session_id, {"command": "tick.warp.stop", "arguments": {}}), {"was_running": True})
        stopped = server.result(session_id, warp)
        self.assertEqual(stopped["output"]["outcome"], "stopped")
        self.assertGreaterEqual(stopped["output"]["executed_ticks"], before_stop)
        self.assertLess(stopped["output"]["executed_ticks"], 120)
        rejected = server.run_cli("session", "submit", session_id, "--command", '{"command":"shutdown","arguments":{}}')
        self.assertEqual(rejected.returncode, 1)
        error = json.loads(rejected.stdout)
        self.assertEqual(error["code"], "shutdown_recording_active")
        self.assertEqual(error["kind"], "error")
        self.assertNotIn("request_id", error)
        self.assertEqual(server.read_resource(session_id, counter), {"ticks": stopped["output"]["executed_ticks"]})
        self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                         {"path": "recordings/shutdown.jsonl", "recorded_commands": 4})
        original = (root / "recordings/shutdown.jsonl").read_bytes()
        shutdown = server.cli("session", "submit", session_id, "--command", '{"command":"shutdown","arguments":{}}')
        self.assertEqual(shutdown["request_id"], 18446744073709551615)
        self.assertEqual(server.result(session_id, shutdown)["output"], None)
        deadline = time.monotonic() + 10
        while server.cli("session", "inspect", session_id)["state"] != "Ended":
            if time.monotonic() >= deadline:
                self.fail("Shutdown did not end the session")
            time.sleep(0.02)
        after = server.run_cli("session", "submit", session_id, "--command", '{"command":"tick.warp.stop","arguments":{}}')
        self.assertEqual(after.returncode, 1)
        self.assertEqual(json.loads(after.stdout)["code"], "session_not_ready")
        self.assertNotIn("request_id", json.loads(after.stdout))
        events = [row["event"] for row in server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))["entries"]]
        shutdown_events = [e for e in events if e.get("command") == "shutdown"]
        self.assertEqual([e["kind"] for e in shutdown_events], ["pending", "completed"])
        self.assertTrue(no_failures(events), events)
        rows = read_jsonl(root / "recordings/shutdown.jsonl")
        self.assertEqual([row["command"] for row in rows[1:-1]],
                         ["tick.warp.start", "inspect.query", "tick.warp.stop", "inspect.query"])
        self.assertEqual(rows[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 4})
        wait_process_gone(process)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertEqual((root / "recordings/shutdown.jsonl").read_bytes(), original)
