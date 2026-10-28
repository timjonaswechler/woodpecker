"""A capture before any explicit tick is rejected without touching existing files."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server
from tests.screenshot import snapshot_tree


class ScreenshotBeforeFirstTickTest(unittest.TestCase):
    def test_capture_before_first_tick_preserves_files_time_and_simulation(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, zero ticks/inputs. Exactly one capture into "
                         "a test-owned existing sentinel; expected screenshot_window_unavailable. "
                         "No bootstrap, retries or OS input. Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "screenshot-before-first-tick")
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
        self.assertTrue(root.resolve().is_relative_to((server.artifacts / "sessions").resolve()))
        shots = root / "screenshots"
        shots.mkdir()
        sentinel = shots / "before-first.png"
        sentinel.write_bytes(b"Test-owned existing bytes: a rejected capture must preserve them.\n")
        before_files = snapshot_tree(shots)
        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        player_type = "test_app::player::Player"
        before_ticks = server.read_resource(session_id, ticks_type)
        before_time = server.read_resource(session_id, time_type)
        before_player = server.read_resource(session_id, player_type)
        self.assertEqual(before_ticks, {"ticks": 0})
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "screenshot.capture", "arguments": {"path": "screenshots/before-first.png"},
        }))
        outcome = server.result(session_id, pending, timeout=10)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "screenshot_window_unavailable")
        self.assertEqual(snapshot_tree(shots), before_files)
        self.assertEqual(server.read_resource(session_id, ticks_type), before_ticks)
        self.assertEqual(server.read_resource(session_id, time_type), before_time)
        self.assertEqual(server.read_resource(session_id, player_type), before_player)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
        server.record(["observation", "capture-before-first-tick"], outcome=outcome,
                      files_unchanged=before_files, application_ticks=before_ticks)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertEqual(snapshot_tree(shots), before_files)
