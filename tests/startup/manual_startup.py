"""Manual startup runs no simulation ticks, even while the client waits."""
import os
import time
import unittest

from tests._support import ROOT, start_server


class ManualStartupTest(unittest.TestCase):
    def test_manual_startup_stays_at_zero_ticks_and_time(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window. Explicit permission for this run "
            "is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "manual-startup")

        # The shared launch config explicitly selects manual startup: no warm-up.
        session_id = server.cli(
            "session", "create", "--config", str(ROOT / "tests/app.toml"),
        )["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not reach SessionReady; see {server.artifacts}")
            time.sleep(0.05)

        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        before_ticks = server.read_resource(session_id, ticks_type)
        before_time = server.read_resource(session_id, time_type)
        self.assertEqual(before_ticks, {"ticks": 0})
        self.assertEqual(before_time["elapsed_secs_f64"], 0.0)
        self.assertEqual(before_time["delta_secs_f64"], 0.0)

        # This is a real client pause, not a tick or an implicit readiness loop.
        server.record(["client", "pause"], seconds=0.25)
        time.sleep(0.25)

        after_ticks = server.read_resource(session_id, ticks_type)
        after_time = server.read_resource(session_id, time_type)
        self.assertEqual(after_ticks, before_ticks)
        self.assertEqual(after_time, before_time)

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
