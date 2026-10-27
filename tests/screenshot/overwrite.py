"""Two real captures replace a test-owned sentinel without advancing simulation."""
import hashlib
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config

TICKS = "test_app::woodpecker::SimulationTicks"


class ScreenshotOverwriteTest(unittest.TestCase):
    def test_two_captures_report_overwrite_and_replace_existing_bytes_with_valid_png(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One window, up to 3600 engine ticks/60 s and 120 readiness ticks/10 s. "
                         "Exactly two captures under one name, zero later ticks/inputs. The first "
                         "test-owned PNG is deliberately replaced with a sentinel between captures; "
                         "the second must replace it. No retry. Permission required: WOODPECKER_TEST_RENDERED=1")
        try:
            from PIL import Image
        except ImportError:
            self.fail("Screenshot checks require Pillow: python3 -m pip install Pillow")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "screenshot-overwrite")
        config = server.artifacts / "engine.toml"
        configuration = load_config()
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
        deadline = time.monotonic() + 90
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "EngineReady":
                break
            self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
            if time.monotonic() >= deadline:
                self.fail("EngineReady deadline exceeded")
            time.sleep(0.05)
        startup_ticks = server.read_resource(session_id, TICKS)["ticks"]
        deadline = time.monotonic() + 10
        for additional in range(121):
            load = server.read_resource(session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            self.assertEqual(load, "Loading")
            if additional == 120 or time.monotonic() >= deadline:
                self.fail("Game readiness budget exceeded")
            self.assertEqual(server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        server.record(["preparation", "game-ready"], startup_ticks=startup_ticks, additional_ticks=additional)
        root = Path(detail["artifact_dir"])
        self.assertTrue(root.resolve().is_relative_to((server.artifacts / "sessions").resolve()))
        relative = "screenshots/repeated.png"
        destination = root / relative
        self.assertFalse(destination.exists())
        time_type = "bevy_time::time::Time<()>"
        player_type = "test_app::player::Player"
        before_ticks = server.read_resource(session_id, TICKS)
        before_time = server.read_resource(session_id, time_type)
        before_player = server.read_resource(session_id, player_type)
        sentinel = b"Test-owned sentinel: capture two must replace these non-PNG bytes.\n"
        captures = []
        for index, overwritten in ((1, False), (2, True)):
            output = server.call(session_id, {
                "command": "screenshot.capture", "arguments": {"path": relative},
            }, timeout=45)
            self.assertEqual(output, {"path": relative, "width": 1440, "height": 900,
                                      "overwritten": overwritten})
            with Image.open(destination) as image:
                self.assertEqual(image.format, "PNG")
                self.assertEqual(image.size, (1440, 900))
                image.verify()
            with Image.open(destination) as image:
                image.load()  # Fully decode, not just the signature/IHDR.
            data = destination.read_bytes()
            self.assertNotEqual(data, sentinel)
            self.assertEqual(server.read_resource(session_id, TICKS), before_ticks)
            self.assertEqual(server.read_resource(session_id, time_type), before_time)
            self.assertEqual(server.read_resource(session_id, player_type), before_player)
            captures.append({"index": index, "output": output, "sha256": hashlib.sha256(data).hexdigest()})
            if index == 1:
                # Prevent a stale first PNG plus a fabricated overwritten=true
                # response from satisfying the second-capture assertions.
                destination.write_bytes(sentinel)
                self.assertEqual(destination.read_bytes(), sentinel)
        self.assertEqual(list((root / "screenshots").iterdir()), [destination])
        server.record(["observation", "screenshot-overwrite"], captures=captures, action_ticks=0,
                      test_owned_sentinel_replaced=True)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
