"""A real Signalstation session can be created, inspected and stopped."""
import os
import time
import unittest

from tests._support import ROOT, load_config, owned_process, start_server, wait_process_gone, write_config


class SessionLifecycleTest(unittest.TestCase):
    def test_session_can_be_created_inspected_and_stopped(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens Signalstation. Explicit permission for this run is required: "
            "WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "session-lifecycle")

        # Manual startup: no engine warm-up, inputs, inspect.query or tick commands.
        configuration = load_config()
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
        config = server.artifacts / "config.toml"
        write_config(config, configuration)
        created = server.cli("session", "create", "--config", str(config))
        session_id = created["id"]
        self.assertEqual(created["state"], "Starting")

        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertEqual(detail["id"], session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not reach SessionReady; see {server.artifacts}")
            time.sleep(0.05)

        sessions = server.cli("session", "ls")
        self.assertEqual(
            [(item["id"], item["state"]) for item in sessions],
            [(session_id, "SessionReady")],
        )

        process = owned_process(server, detail["artifact_dir"])
        stopped = server.cli("session", "stop", session_id)
        self.assertEqual(stopped["id"], session_id)
        self.assertIn(stopped["state"], ("Stopping", "Ended"), stopped)
        deadline = time.monotonic() + 10
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "Ended":
                break
            self.assertEqual(detail["state"], "Stopping", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not reach Ended; see {server.artifacts}")
            time.sleep(0.05)

        wait_process_gone(process)
        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
