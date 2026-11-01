"""Caught system/task/worker/unknown-payload panics are observations, not forced exits."""
import os
import time
import unittest

from tests._support import load_config, start_server, write_config
from tests.reports import audit_local_files, failures, observe, reports


class CaughtPanicTest(unittest.TestCase):
    def test_caught_panics_keep_payload_location_and_backtrace_separate_and_process_alive(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Four serial manual game windows, caught system/task/worker/panic_any "
                         "panics via F9 and two ticks each (eight total). Local reports only. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        for mode in ("caught-system", "caught-task", "caught-worker", "caught-any"):
            server = start_server(self, mode)
            configuration = load_config()
            configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-report-action", mode]
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
            self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})["executed_ticks"], 1)
            self.assertIsNone(server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "f9"}}))
            self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})["executed_ticks"], 1)
            self.assertEqual(server.read_resource(session_id, "test_app::diagnostics::reports::ReportProbe"),
                             {"mode": mode, "actions": 1, "caught": 1, "handled_errors": 0})
            self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 2})
            server.cli("session", "stop", session_id)
            events = observe(server, session_id, lambda e: len(reports(e)) >= 1 and
                             any(v["kind"] == "lifecycle" and v["state"] == "Ended" for v in e))
            self.assertEqual(len(failures(events)), 1, events)
            self.assertEqual(len(reports(events)), 1, events)
            failure = failures(events)[0]
            self.assertEqual(failure["message"], None if mode == "caught-any" else
                             "Signalstation " + mode.replace("-", " ") + " panic")
            origin = failure["origin"]
            self.assertEqual(origin["kind"], "panic")
            self.assertTrue(origin["location"]["file"].endswith("diagnostics/reports.rs"))
            self.assertGreater(origin["location"]["line"], 0)
            self.assertEqual(origin["backtrace_status"], "captured")
            self.assertIsInstance(origin["backtrace"], str)
            self.assertTrue(origin["backtrace"].strip())
            self.assertEqual(reports(events)[0]["report"]["failure"], failure)
            self.assertEqual(reports(events)[0]["result"]["status"], "submitted")
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0)
            audit_local_files(server, detail["artifact_dir"], events)
