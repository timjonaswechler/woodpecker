"""Rejected keyboard transitions preserve queued and applied virtual key state."""
import json
import os
import unittest

from tests._shared import game


class InvalidTransitionsTest(unittest.TestCase):
    def test_invalid_tokens_and_transitions_preserve_queued_held_and_released_state(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "Prepared shared-or-standalone Signalstation fixture: up to 120 readiness ticks/10 s, "
            "then F9 and three reset ticks. This case uses 4 action ticks. "
            "Virtual inputs only, no OS input. Permission required: WOODPECKER_TEST_RENDERED=1",
        )
        server, session_id = game(self, "keyboard-invalid-transitions")
        ticks_type = "test_app::woodpecker::SimulationTicks"
        keys_type = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
        before_ticks = server.read_resource(session_id, ticks_type)["ticks"]
        self.assertEqual(server.read_resource(session_id, keys_type)["pressed"], [])
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "shift_left"},
        }))
        rejections = []
        for phase in ("queued-press", "applied-held"):
            before_keys = server.read_resource(session_id, keys_type)
            before_phase_ticks = server.read_resource(session_id, ticks_type)
            for command, key, code in (
                ("input.keyboard.press", "shift_left", "key_already_pressed"),
                ("input.keyboard.release", "control_left", "key_not_pressed"),
                ("input.keyboard.press", "woodpecker_invalid_key", "invalid_key"),
                ("input.keyboard.release", "woodpecker_invalid_key", "invalid_key"),
            ):
                pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                    "command": command, "arguments": {"key": key},
                }))
                outcome = server.result(session_id, pending)
                self.assertEqual(outcome["kind"], "rejected", outcome)
                self.assertEqual(outcome["error"]["code"], code, outcome)
                rejections.append({"phase": phase, "command": command, "key": key, "outcome": outcome})
            self.assertEqual(server.read_resource(session_id, keys_type), before_keys)
            self.assertEqual(server.read_resource(session_id, ticks_type), before_phase_ticks)
            # A valid queued press must survive the errors; later errors must
            # neither release the held key nor create a new press/release edge.
            self.assertEqual(server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
            applied = server.read_resource(session_id, keys_type)
            self.assertEqual(applied["pressed"], ["ShiftLeft"])
            self.assertEqual(applied["just_pressed"], ["ShiftLeft"] if phase == "queued-press" else [])
            self.assertEqual(applied["just_released"], [])
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "shift_left"},
        }))
        # Accepted state is already released, even though Bevy still holds the
        # key until tick three. A second release must not corrupt that pending edit.
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "input.keyboard.release", "arguments": {"key": "shift_left"},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "key_not_pressed")
        rejections.append({"phase": "queued-release", "outcome": outcome})
        self.assertEqual(server.read_resource(session_id, keys_type), applied)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        released = server.read_resource(session_id, keys_type)
        self.assertEqual(released["pressed"], [])
        self.assertEqual(released["just_pressed"], [])
        self.assertEqual(released["just_released"], ["ShiftLeft"])
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        idle = server.read_resource(session_id, keys_type)
        self.assertEqual(idle, {"pressed": [], "just_pressed": [], "just_released": []})
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - before_ticks, 4)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
        server.record(["observation", "invalid-keyboard-transitions"], rejections=rejections,
                      held=applied, released=released, final=idle, action_ticks=4)
