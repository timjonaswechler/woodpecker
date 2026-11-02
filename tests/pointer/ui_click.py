"""Absolute virtual positioning activates an actual laid-out UI button only on ticks."""
import os
import unittest

from tests._shared import game
from tests.pointer import MOUSE, TICKS, layout_target, pointer_states, read_components


class UiClickTest(unittest.TestCase):
    def test_absolute_pointer_picks_the_real_map_button_without_early_effects(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Prepared shared-or-standalone Signalstation fixture: up to 120 readiness ticks/10 s, "
                         "then F9 and three reset ticks. This case uses 3 action ticks. "
                         "Virtual inputs only, no OS input. Permission required: WOODPECKER_TEST_RENDERED=1")
        server, session_id = game(self, "pointer-ui-click")
        view_type = "test_app::player::camera::ViewState"
        button_type = "bevy_ui_widgets::button::Button"
        pressed_type = "bevy_ui::interaction_states::Pressed"
        target = layout_target(server, session_id, button_type, label="Gesamtkarte [G]")
        center = target["position"]
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        before_pointers = pointer_states(server, session_id)
        self.assertTrue(before_pointers)
        self.assertTrue(all(item["location"] is None for item in before_pointers))
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": center},
        }))
        self.assertEqual(pointer_states(server, session_id), before_pointers)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        active, = [item for item in pointer_states(server, session_id) if item["location"] is not None]
        self.assertEqual(active["location"]["position"], center)
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.press", "arguments": {"button": "left"},
        }))
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], [])
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks + 1)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Map")
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], ["Left"])
        pressed = read_components(server, session_id, [button_type, pressed_type], [])
        self.assertEqual([item["entity"] for item in pressed], [target["entity"]])
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.release", "arguments": {"button": "left"},
        }))
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], ["Left"])
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], [])
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Map")
        pressed = read_components(server, session_id, [button_type, pressed_type], [])
        self.assertEqual(pressed, [])
        final, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
        self.assertEqual(final["location"]["position"], center)
        self.assertFalse(final["buttons"]["primary"])
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 3)
        server.record(["observation", "ui-click"], target=target, position=center, action_ticks=3,
                      mode="Map", pointer=final)
