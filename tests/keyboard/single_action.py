"""A real map-toggle action fires once per press edge, not once per held tick."""
import os
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class SingleActionTest(unittest.TestCase):
    def test_map_toggle_runs_once_while_held_and_again_only_after_repress(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window. Up to 3600 engine-startup "
            "ticks/60 seconds and 120 readiness ticks/ten seconds, then seven action "
            "ticks with virtual g press/hold/release/repress/release. No OS input. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "keyboard-single-action")
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
        ticks_type = "test_app::woodpecker::SimulationTicks"
        startup_ticks = server.read_resource(session_id, ticks_type)["ticks"]
        deadline = time.monotonic() + 10
        for additional in range(121):
            load = server.read_resource(session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            self.assertEqual(load, "Loading", load)
            if additional == 120 or time.monotonic() >= deadline:
                self.fail("Game readiness budget exceeded")
            self.assertEqual(server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        server.record(["preparation", "game-ready"], startup_ticks=startup_ticks, additional_ticks=additional)
        view_type = "test_app::player::camera::ViewState"
        keys_type = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
        before = server.read_resource(session_id, ticks_type)["ticks"]
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertEqual(server.read_resource(session_id, keys_type)["pressed"], [])
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "g"},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Map")
        first_keys = server.read_resource(session_id, keys_type)
        self.assertEqual(first_keys["pressed"], ["KeyG"])
        self.assertEqual(first_keys["just_pressed"], ["KeyG"])
        held_modes = []
        # Observe every held tick separately: an even number of unintended
        # toggles must not hide a repeated action in a single final snapshot.
        for _ in range(3):
            server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            mode = server.read_resource(session_id, view_type)["mode"]
            held_modes.append(mode)
            self.assertEqual(mode, "Map")
            keys = server.read_resource(session_id, keys_type)
            self.assertEqual(keys["pressed"], ["KeyG"])
            self.assertEqual(keys["just_pressed"], [])
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "g"},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Map")
        self.assertEqual(server.read_resource(session_id, keys_type)["pressed"], [])
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "g"},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "g"},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertEqual(server.read_resource(session_id, keys_type)["pressed"], [])
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - before, 7)
        server.record(["observation", "single-action"], key="g", first_mode="Map", held_modes=held_modes,
                      release_mode="Map", repress_mode="Overview", action_ticks=7)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
