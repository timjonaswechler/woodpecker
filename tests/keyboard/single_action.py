"""A real map-toggle action fires once per press edge, not once per held tick."""
import os
import unittest

from tests._shared import game


class SingleActionTest(unittest.TestCase):
    def test_map_toggle_runs_once_while_held_and_again_only_after_repress(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "Prepared shared-or-standalone Signalstation fixture: up to 120 readiness ticks/10 s, "
            "then F9 and three reset ticks. This case uses 7 action ticks. "
            "Virtual inputs only, no OS input. Permission required: WOODPECKER_TEST_RENDERED=1",
        )
        server, session_id = game(self, "keyboard-single-action")
        ticks_type = "test_app::woodpecker::SimulationTicks"
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
