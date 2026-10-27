"""An intentional app panic fails its request and leaves one durable local report."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import (ROOT, start_server, load_config, owned_process,
                            wait_process_gone, write_config)
from tests.reports import audit_local_files


class PanicExitTest(unittest.TestCase):
    def test_panic_fails_the_tick_and_reports_once_before_process_cleanup(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window and intentionally crashes it with "
            "F11 and one explicit tick. Only local reports are used. Permission for this "
            "run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        # A retained Failed session makes server shutdown unsuccessful, not forced.
        server = start_server(self, "panic-exit", expected_exit_code=1)
        config = server.artifacts / "diagnostics.toml"
        configuration = load_config()
        configuration["launch"]["arguments"][:0] = ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
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

        process = owned_process(server, detail["artifact_dir"])
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "f11"},
        }))
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        }))
        self.assertEqual(pending["kind"], "pending", pending)
        self.assertEqual(pending["command"], "tick.warp.start", pending)
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome, {
            "kind": "failed",
            "request_id": pending["request_id"],
            "command": "tick.warp.start",
            "error": {"code": "ended", "message": "session has ended"},
        })
        # No release, extra tick or successful shutdown command into the dead app.
        # Observe from the beginning again: result() consumed no server activity.
        events = []
        cursor = None
        deadline = time.monotonic() + 15
        while True:
            arguments = ["session", "poll", session_id, "--wait-ms", "100"]
            if cursor is not None:
                arguments.extend(["--cursor", json.dumps(cursor)])
            activity = server.cli(*arguments)
            self.assertEqual(activity["kind"], "activity", activity)
            self.assertEqual(activity["cursor"]["session_id"], session_id, activity)
            if cursor is not None:
                self.assertGreaterEqual(activity["cursor"]["position"], cursor["position"])
            for entry in activity["entries"]:
                self.assertEqual(entry["cursor"]["session_id"], session_id, entry)
                events.append(entry["event"])
            reports = [event for event in events if event["kind"] == "report"]
            terminal = [event for event in events
                        if event["kind"] == "lifecycle" and event["state"] == "Failed"]
            if terminal and reports:
                break
            if time.monotonic() >= deadline:
                self.fail(f"Panic did not reach Failed with a local report; see {server.artifacts}")
            cursor = activity["cursor"]

        # Failed is published only after the session worker has drained its events.
        failures = [event["event"]["failure"] for event in events
                    if event["kind"] == "event" and event["event"]["kind"] == "failure"]
        self.assertEqual(len(failures), 1, failures)
        failure = failures[0]
        message = "Signalstation diagnostic panic"
        self.assertEqual(failure["message"], message)
        self.assertEqual(failure["origin"]["kind"], "panic", failure)
        location = failure["origin"]["location"]
        self.assertTrue(location["file"].endswith("src/diagnostics.rs"), location)
        self.assertGreater(location["line"], 0)
        ended = [event["event"] for event in events
                 if event["kind"] == "event" and event["event"]["kind"] == "ended"]
        self.assertEqual(len(ended), 1, ended)
        self.assertEqual(ended[0]["reason"], {
            "kind": "process_exit", "status": "exit status: 101",
        })
        outcomes = [event for event in events
                    if event.get("request_id") == pending["request_id"]
                    and event["kind"] in ("completed", "rejected", "failed")]
        self.assertEqual(outcomes, [outcome])
        self.assertEqual(len(reports), 1, reports)
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

        detail = server.cli("session", "inspect", session_id)
        self.assertEqual(detail["state"], "Failed", detail)
        self.assertEqual(detail["error"]["code"], "ended", detail)
        self.assertIsNone(server.process.poll(), "The app panic stopped the server")
        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 1)
        self.assertIn(
            "shutdown_incomplete",
            (server.artifacts / "server.stderr.log").read_text(encoding="utf-8"),
        )
        wait_process_gone(process)
        audit_local_files(server, detail["artifact_dir"], events)
        self.assertEqual(path.read_text(encoding="utf-8"), markdown)
