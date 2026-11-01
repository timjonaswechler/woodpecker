"""Unexpected successful and failed exits report ProcessExit after draining stderr."""
import os
import time
import unittest

from tests._support import load_config, start_server, write_config
from tests.reports import audit_local_files, failures, observe, reports


class ProcessExitTest(unittest.TestCase):
    def test_unexpected_exit_zero_and_nonzero_are_not_shutdown_or_panic(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Two serial manual game windows. F9 plus one tick each deliberately "
                         "exit(0)/exit(17); both servers have declared exit 1. Local reports. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        for code in (0, 17):
            server = start_server(self, f"process-exit-{code}", expected_exit_code=1)
            configuration = load_config()
            configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-report-action", f"exit-{code}"]
            path = server.artifacts / "config.toml"
            write_config(path, configuration)
            session_id = server.cli("session", "create", "--config", str(path))["id"]
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "SessionReady":
                    break
                if time.monotonic() >= deadline:
                    self.fail("Session readiness deadline")
                time.sleep(0.03)
            self.assertIsNone(server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f9"}}))
            pending = server.cli("session", "submit", session_id, "--command",
                                 '{"command":"tick.warp.start","arguments":{"ticks":1}}')
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "failed", outcome)
            self.assertEqual(outcome["error"]["code"], "ended")
            events = observe(server, session_id, lambda e: len(reports(e)) >= 1 and
                             any(v["kind"] == "lifecycle" and v["state"] == "Failed" for v in e))
            self.assertEqual(len(failures(events)), 1, events)
            self.assertEqual(len(reports(events)), 1, events)
            self.assertEqual(failures(events)[0], {"message": "process exited unexpectedly",
                             "origin": {"kind": "process_exit", "status": f"exit status: {code}"}})
            session_events = [v["event"] for v in events if v["kind"] == "event"]
            self.assertLess(next(i for i,v in enumerate(session_events) if v["kind"] == "failure"),
                            next(i for i,v in enumerate(session_events) if v["kind"] == "ended"))
            self.assertEqual(reports(events)[0]["result"]["status"], "submitted")
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 1)
            audit_local_files(server, detail["artifact_dir"], events)
            self.assertIn(f"Signalstation report probe exiting {code}".encode(),
                          (server.artifacts / "server.stderr.log").read_bytes())
