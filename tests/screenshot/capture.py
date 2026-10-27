"""A real PNG contains the prepared scene without advancing simulation."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


def _color_fraction(image, rect, rgb_min, rgb_max):
    """Measure a scene region using Pillow's decoded RGB pixels, not a PNG codec."""
    pixels = image.crop(rect).convert("RGB").getdata()
    matching = sum(all(rgb_min[channel] <= pixel[channel] <= rgb_max[channel]
                       for channel in range(3)) for pixel in pixels)
    return matching / len(pixels)


class ScreenshotCaptureTest(unittest.TestCase):
    def test_capture_contains_the_scene_without_advancing_simulation(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window, allows up to 3600 engine-startup "
            "and 120 game-readiness ticks, then captures once without input or further ticks. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        try:
            from PIL import Image
        except ImportError:
            self.fail("Screenshot checks require Pillow: python3 -m pip install Pillow")
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "screenshot-capture")
        config = server.artifacts / "engine.toml"
        configuration = load_config()
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
        arguments = configuration["launch"]["arguments"]
        self.assertIn("--fixed-window", arguments)
        self.assertEqual(arguments[arguments.index("--scale-factor") + 1], "1")
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
                self.fail(f"Engine did not become ready; see {server.artifacts}")
            time.sleep(0.05)
        startup_ticks = server.read_resource(session_id, "test_app::woodpecker::SimulationTicks")["ticks"]
        server.record(["preparation", "engine-ready"], ticks=startup_ticks)

        # Separate the game's load state from technical renderer readiness.
        deadline = time.monotonic() + 10
        for additional_ticks in range(121):
            load = server.read_resource(session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            self.assertEqual(load, "Loading", load)
            if additional_ticks == 120 or time.monotonic() >= deadline:
                self.fail(f"Game not ready after {additional_ticks} extra ticks; see {server.artifacts}")
            output = server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            })
            self.assertEqual(output["outcome"], "completed", output)
        server.record(["preparation", "game-ready"], additional_ticks=additional_ticks)

        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        player_type = "test_app::player::Player"
        before_ticks = server.read_resource(session_id, ticks_type)
        before_time = server.read_resource(session_id, time_type)
        before_player = server.read_resource(session_id, player_type)

        # Exactly one capture. Readback/encoding may wait, but no repair tick or retry.
        output = server.call(session_id, {
            "command": "screenshot.capture", "arguments": {"path": "screenshots/start.png"},
        }, timeout=45)
        self.assertEqual(output, {
            "path": "screenshots/start.png", "width": 1440, "height": 900, "overwritten": False,
        })
        self.assertEqual(server.read_resource(session_id, ticks_type), before_ticks)
        self.assertEqual(server.read_resource(session_id, time_type), before_time)
        self.assertEqual(server.read_resource(session_id, player_type), before_player)

        # Explicit fixture contract: fixed Signalstation start scene, 1440x900,
        # scale 1. Intentional visual changes require a reviewed fixture update,
        # never silently widened RGB bands or retries. Values remain unchanged.
        landmarks = (
            ("HUD background", (40, 30, 180, 80), (10, 15, 15), (65, 75, 75), 0.95),
            ("Entrance floor", (740, 330, 790, 370), (210, 205, 190), (255, 255, 245), 0.9),
            ("Teal start pad", (610, 425, 720, 510), (50, 120, 120), (140, 215, 215), 0.15),
        )
        server.record(["fixture", "screenshot-start-scene"], version=1, size=[1440, 900],
                      scale_factor=1, landmarks=landmarks)
        path = Path(detail["artifact_dir"]) / output["path"]
        measurements = {}
        with Image.open(path) as image:
            self.assertEqual(image.format, "PNG")
            self.assertEqual(image.size, (1440, 900))
            image.load()
            for name, rect, rgb_min, rgb_max, minimum in landmarks:
                measurements[name] = {
                    "fraction": _color_fraction(image, rect, rgb_min, rgb_max),
                    "minimum": minimum, "rect": rect, "rgb_min": rgb_min, "rgb_max": rgb_max,
                }
        (server.artifacts / "image-check.json").write_text(
            json.dumps(measurements, indent=2) + "\n", encoding="utf-8",
        )
        for name, measurement in measurements.items():
            self.assertGreaterEqual(
                measurement["fraction"], measurement["minimum"],
                f"Missing {name}: {measurement}; see {path} and {server.artifacts / 'image-check.json'}",
            )

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
