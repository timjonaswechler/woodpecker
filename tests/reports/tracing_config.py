"""Tracing is opt-in, filtered before observation, and requires a confirmed layer."""
import os
import time
import unittest

from tests._support import load_config, start_server, write_config
from tests.reports import audit_local_files, failures, observe, reports


class TracingConfigTest(unittest.TestCase):
    def test_opt_in_filter_missing_layer_and_handled_error_have_distinct_observation(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Five serial manual game starts/windows; four one-tick variants with "
                         "F12/F9, one declared failed start without tracing layer. Only local reports. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        for label, enabled, log, key in (
            ("disabled", False, "error", "f12"), ("filtered", True, "off", "f12"),
            ("enabled", True, "error", "f12"), ("handled", True, "error", "f9"),
            ("missing-layer", True, "error", None),
        ):
            server = start_server(self, "tracing-" + label, env={"RUST_LOG": log},
                                  expected_exit_code=1 if key is None else 0)
            configuration = load_config()
            configuration["launch"]["arguments"].insert(0, "--woodpecker-diagnostics")
            configuration["report"]["tracing_errors"] = enabled
            if label == "handled":
                configuration["launch"]["arguments"] += ["--woodpecker-report-action", "handled-error"]
            if key is None:
                configuration["launch"]["arguments"] += ["--woodpecker-no-tracing-layer"]
            path = server.artifacts / "config.toml"
            write_config(path, configuration)
            created = server.cli("session", "create", "--config", str(path))
            session_id = created["id"]
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                if detail["state"] in ("SessionReady", "Failed"):
                    break
                self.assertEqual(detail["state"], "Starting", detail)
                if time.monotonic() >= deadline:
                    self.fail("Readiness deadline exceeded")
                time.sleep(0.03)
            if key is None:
                self.assertEqual(detail["state"], "Failed")
                self.assertEqual(detail["error"]["code"], "launch")
                self.assertIn("layer", detail["error"]["message"])
                events = server.cli("session", "poll", session_id)["entries"]
                self.assertFalse(any(row["event"]["kind"] == "report" for row in events))
            else:
                self.assertEqual(detail["state"], "SessionReady", detail)
                self.assertIsNone(server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": key}}))
                self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})["executed_ticks"], 1)
                self.assertIsNone(server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": key}}))
                if label == "handled":
                    probe = server.read_resource(session_id, "test_app::diagnostics::reports::ReportProbe")
                    self.assertEqual(probe, {"mode": "handled-error", "actions": 1, "caught": 0, "handled_errors": 1})
                server.cli("session", "stop", session_id)
                events = observe(server, session_id, lambda values:
                    any(v["kind"] == "lifecycle" and v["state"] == "Ended" for v in values)
                    and (label != "enabled" or len(reports(values)) >= 1))
                self.assertEqual(len(failures(events)), int(label == "enabled"), (label, events))
                self.assertEqual(len(reports(events)), int(label == "enabled"), (label, events))
                if label == "enabled":
                    self.assertEqual(failures(events)[0]["origin"]["kind"], "tracing_error")
                    self.assertEqual(failures(events)[0]["message"], "Signalstation diagnostic tracing error")
                    self.assertEqual(reports(events)[0]["result"]["status"], "submitted")
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 1 if key is None else 0)
            audit_local_files(server, detail["artifact_dir"],
                              [row["event"] for row in events] if key is None else events)
            server.record(["observation", label], tracing=enabled, filter=log)
