"""An opt-in tracing error produces a local report without ending the session."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config
from tests.reports import audit_local_files


class TracingReportTest(unittest.TestCase):
    def test_tracing_error_writes_a_report_and_session_remains_usable(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window, opts into diagnostics and tracing "
            "observation, and sends F12 plus two explicit ticks. Only local reports are used. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        # Configure the actual server/app filter without changing the parent environment.
        server = start_server(self, "tracing-report", env={"RUST_LOG": "error"})
        server.record(["process", "environment"], overrides={"RUST_LOG": "error"})
        config = server.artifacts / "diagnostics.toml"
        configuration = load_config()
        configuration["launch"]["arguments"].insert(0, "--woodpecker-diagnostics")
        configuration["report"]["tracing_errors"] = True
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not become ready; see {server.artifacts}")
            time.sleep(0.05)

        # Manual startup needs no engine/game warm-up for this application action.
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "f12"},
        }))
        output = server.call(session_id, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        })
        self.assertEqual(output["outcome"], "completed", output)
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "f12"},
        }))
        output = server.call(session_id, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        })
        self.assertEqual(output["outcome"], "completed", output)

        # Failure and provider outcome are independent session activity, not a
        # fabricated command response. Publishing may finish after the tick result.
        events = []
        cursor = None
        deadline = time.monotonic() + 10
        while True:
            arguments = ["session", "poll", session_id, "--wait-ms", "100"]
            if cursor is not None:
                arguments.extend(["--cursor", json.dumps(cursor)])
            activity = server.cli(*arguments)
            self.assertEqual(activity["kind"], "activity", activity)
            events.extend(entry["event"] for entry in activity["entries"])
            cursor = activity["cursor"]
            reports = [event for event in events if event["kind"] == "report"]
            if reports:
                break
            if time.monotonic() >= deadline:
                self.fail(f"Local report missing; see {server.artifacts}")
        failures = [event["event"]["failure"] for event in events
                    if event["kind"] == "event" and event["event"]["kind"] == "failure"]
        self.assertEqual(len(failures), 1, failures)
        self.assertEqual(len(reports), 1, reports)
        failure = failures[0]
        message = "Signalstation diagnostic tracing error"
        self.assertEqual(failure["message"], message)
        self.assertEqual(failure["origin"]["kind"], "tracing_error")
        self.assertEqual(failure["origin"]["target"], "signalstation::diagnostics")
        report = reports[0]["report"]
        self.assertEqual(report["failure"], failure)
        result = reports[0]["result"]
        self.assertEqual(result["status"], "submitted", result)
        self.assertEqual(result["outcome"]["kind"], "created", result)
        reference = result["outcome"]["reference"]
        self.assertEqual(reference["kind"], "file", reference)
        relative = Path(reference["reference"]["path"])
        self.assertFalse(relative.is_absolute())
        self.assertNotIn("..", relative.parts)
        path = Path(detail["artifact_dir"]) / relative
        markdown = path.read_text(encoding="utf-8")
        self.assertIn(message, markdown)
        self.assertIn(
            f"<!-- bug_hunter-signature: {report['signature']['value']} -->", markdown,
        )

        # Prove the session is still controllable after the observed error/report.
        expected = json.loads((ROOT / "tests/settings.json").read_text(encoding="utf-8"))
        self.assertEqual(
            server.read_resource(session_id, "test_app::app::settings::Settings"), expected,
        )
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
        # End the app while the server still exposes Activity. Late duplicate
        # observations are included; Ended itself is not a provider completion barrier.
        server.cli("session", "stop", session_id)
        deadline = time.monotonic() + 10
        while not any(event["kind"] == "lifecycle" and event["state"] == "Ended" for event in events):
            activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor), "--wait-ms", "100")
            self.assertEqual(activity["kind"], "activity", activity)
            events.extend(entry["event"] for entry in activity["entries"])
            cursor = activity["cursor"]
            if time.monotonic() >= deadline:
                self.fail(f"Session did not end after reporting; see {server.artifacts}")
        self.assertEqual([event["event"]["failure"] for event in events
                          if event["kind"] == "event" and event["event"]["kind"] == "failure"], [failure])
        self.assertEqual([event for event in events if event["kind"] == "report"], reports)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "Ended")
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        audit_local_files(server, detail["artifact_dir"], events)
        self.assertEqual(path.read_text(encoding="utf-8"), markdown)
