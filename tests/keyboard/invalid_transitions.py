"""Rejected keyboard transitions preserve queued and applied virtual key state."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class InvalidTransitionsTest(unittest.TestCase):
    def test_invalid_tokens_and_transitions_preserve_queued_held_and_released_state(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window with manual startup and exactly "
            "four ticks. Virtual ShiftLeft press/release; duplicate press, unmatched "
            "ControlLeft release and unknown-token press/release are deliberately "
            "rejected while queued and held; duplicate queued release is rejected too. "
            "Nine expected rejections, no OS input. Permission for this run is "
            "required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "keyboard-invalid-transitions")
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
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
