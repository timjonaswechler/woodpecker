"""Stopping one managed session must leave the other session ready."""
import os
import time
import unittest

from tests._support import (ROOT, WINDOW_POSITIONS, start_server, load_config,
                            owned_process, wait_process_gone, write_config)


class SessionIsolationTest(unittest.TestCase):
    def test_stopping_one_session_leaves_the_other_ready(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens two Signalstation windows. Explicit permission for this run "
            "is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "session-isolation")
        configs = []
        for label, (x, y) in zip(("A", "B"), WINDOW_POSITIONS):
            config = server.artifacts / f"session-{label}.toml"
            configuration = load_config()
            configuration["launch"]["arguments"][:0] = ["--window-position", f"{x},{y}",
                "--woodpecker-diagnostics", "--woodpecker-process-probe"]
            write_config(config, configuration)
            configs.append(str(config))

        # Both sessions use manual startup: no inputs or tick commands.
        session_a = server.cli("session", "create", "--config", configs[0])["id"]
        session_b = server.cli("session", "create", "--config", configs[1])["id"]
        self.assertNotEqual(session_a, session_b)
        processes = {}
        for session_id in (session_a, session_b):
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertEqual(detail["id"], session_id)
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "SessionReady":
                    break
                self.assertEqual(detail["state"], "Starting", detail)
                if time.monotonic() >= deadline:
                    self.fail(f"Session {session_id} did not reach SessionReady; see {server.artifacts}")
                time.sleep(0.05)

            processes[session_id] = owned_process(server, detail["artifact_dir"])

        # Only A is stopped. B must survive that operation on the same server.
        server.cli("session", "stop", session_a)
        deadline = time.monotonic() + 10
        while True:
            detail = server.cli("session", "inspect", session_a)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "Ended":
                break
            self.assertEqual(detail["state"], "Stopping", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session A did not reach Ended; see {server.artifacts}")
            time.sleep(0.05)

        wait_process_gone(processes[session_a])
        surviving = server.cli("session", "inspect", session_b)
        self.assertEqual(surviving["id"], session_b)
        self.assertEqual(surviving["state"], "SessionReady", surviving)
        self.assertIsNone(surviving["error"], surviving)
        sessions = server.cli("session", "ls")
        self.assertCountEqual(
            [(item["id"], item["state"]) for item in sessions],
            [(session_a, "Ended"), (session_b, "SessionReady")],
        )

        # A fresh bridge request proves B is alive, not merely cached as Ready.
        self.assertEqual(
            server.read_resource(session_b, "test_app::woodpecker::SimulationTicks"),
            {"ticks": 0},
        )

        # B is stopped separately for cleanup, not by stopping the server first.
        server.cli("session", "stop", session_b)
        deadline = time.monotonic() + 10
        while True:
            detail = server.cli("session", "inspect", session_b)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "Ended":
                break
            self.assertEqual(detail["state"], "Stopping", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session B did not reach Ended; see {server.artifacts}")
            time.sleep(0.05)

        wait_process_gone(processes[session_b])
        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
