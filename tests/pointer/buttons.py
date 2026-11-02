"""All three pointer buttons reach both picking and Bevy button state across ticks."""
import os
import unittest

from tests._shared import game
from tests.pointer import MOUSE, TICKS, pointer_states


class PointerButtonsTest(unittest.TestCase):
    def test_three_buttons_hold_and_release_independently_on_the_virtual_pointer(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Prepared shared-or-standalone Signalstation fixture: up to 120 readiness ticks/10 s, "
                         "then F9 and three reset ticks. This case uses 8 action ticks. "
                         "Virtual inputs only, no OS input. Permission required: WOODPECKER_TEST_RENDERED=1")
        server, session_id = game(self, "pointer-buttons")
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        before = pointer_states(server, session_id)
        self.assertTrue(before)
        self.assertTrue(all(item["location"] is None for item in before))
        empty_mouse = {"pressed": [], "just_pressed": [], "just_released": []}
        self.assertEqual(server.read_resource(session_id, MOUSE), empty_mouse)
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": [720.0, 850.0]},
        }))
        self.assertEqual(pointer_states(server, session_id), before)
        self.assertEqual(server.read_resource(session_id, MOUSE), empty_mouse)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks)
        presses = []
        pointer_id = None
        # Observe each Press separately: swapping Left/Right only at Press must
        # not disappear inside a simultaneous three-button aggregate.
        for button, bevy_button, expected_pressed, expected_picking in (
            ("left", "Left", ["Left"], {"primary": True, "secondary": False, "middle": False}),
            ("right", "Right", ["Left", "Right"], {"primary": True, "secondary": True, "middle": False}),
            ("middle", "Middle", ["Left", "Middle", "Right"], {"primary": True, "secondary": True, "middle": True}),
        ):
            before = pointer_states(server, session_id)
            before_mouse = server.read_resource(session_id, MOUSE)
            before_phase_ticks = server.read_resource(session_id, TICKS)
            self.assertIsNone(server.call(session_id, {
                "command": "input.pointer.press", "arguments": {"button": button},
            }))
            self.assertEqual(pointer_states(server, session_id), before)
            self.assertEqual(server.read_resource(session_id, MOUSE), before_mouse)
            self.assertEqual(server.read_resource(session_id, TICKS), before_phase_ticks)
            self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}}),
                             {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
            active, = [item for item in pointer_states(server, session_id) if item["location"] is not None]
            if pointer_id is None:
                pointer_id = active["entity"]
            self.assertEqual(active["entity"], pointer_id)
            self.assertEqual(active["location"]["position"], [720.0, 850.0])
            self.assertEqual(active["buttons"], expected_picking)
            self.assertEqual(server.read_resource(session_id, MOUSE), {
                "pressed": expected_pressed, "just_pressed": [bevy_button], "just_released": [],
            })
            presses.append({"button": button, "pointer": active})
        all_buttons = ["Left", "Middle", "Right"]
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        held, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
        self.assertEqual(held, active)
        self.assertEqual(server.read_resource(session_id, MOUSE),
                         {"pressed": all_buttons, "just_pressed": [], "just_released": []})
        releases = []
        for button, bevy_button, expected_pressed, expected_picking in (
            ("left", "Left", ["Middle", "Right"], {"primary": False, "secondary": True, "middle": True}),
            ("right", "Right", ["Middle"], {"primary": False, "secondary": False, "middle": True}),
            ("middle", "Middle", [], {"primary": False, "secondary": False, "middle": False}),
        ):
            before = pointer_states(server, session_id)
            before_mouse = server.read_resource(session_id, MOUSE)
            before_phase_ticks = server.read_resource(session_id, TICKS)
            self.assertIsNone(server.call(session_id, {
                "command": "input.pointer.release", "arguments": {"button": button},
            }))
            self.assertEqual(pointer_states(server, session_id), before)
            self.assertEqual(server.read_resource(session_id, MOUSE), before_mouse)
            self.assertEqual(server.read_resource(session_id, TICKS), before_phase_ticks)
            server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            pointer, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
            self.assertEqual(pointer["buttons"], expected_picking)
            self.assertEqual(pointer["location"]["position"], [720.0, 850.0])
            self.assertEqual(server.read_resource(session_id, MOUSE), {
                "pressed": expected_pressed, "just_pressed": [], "just_released": [bevy_button],
            })
            releases.append({"button": button, "pointer": pointer})
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, MOUSE), empty_mouse)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 8)
        server.record(["observation", "pointer-buttons"], presses=presses, releases=releases, action_ticks=8)
