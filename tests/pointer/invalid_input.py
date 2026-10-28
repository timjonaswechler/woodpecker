"""Pointer rejections preserve accepted queued work and the applied pointer state."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server
from tests.pointer import MOUSE, TICKS, pointer_states


class InvalidPointerInputTest(unittest.TestCase):
    def test_bad_positions_buttons_and_transitions_do_not_damage_queued_or_applied_state(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, exactly four ticks. Nineteen intentional "
                         "pointer rejections: missing position, four window-bound violations, "
                         "unknown buttons and duplicate/unmatched transitions while queued/held. "
                         "Valid middle press/release and two absolute positions. No OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "pointer-invalid-input")
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
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        before = pointer_states(server, session_id)
        self.assertTrue(before)
        self.assertTrue(all(item["location"] is None for item in before))
        empty_mouse = {"pressed": [], "just_pressed": [], "just_released": []}
        self.assertEqual(server.read_resource(session_id, MOUSE), empty_mouse)
        rejections = []
        for command, arguments in (
            ("input.pointer.press", {"button": "left"}),
            ("input.pointer.scroll", {"delta": [0.0, 0.0]}),
        ):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": command, "arguments": arguments,
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], "pointer_location_unavailable")
            rejections.append({"phase": "no-location", "command": command, "outcome": outcome})
        self.assertEqual(pointer_states(server, session_id), before)
        self.assertEqual(server.read_resource(session_id, MOUSE), empty_mouse)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks)
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": [100.0, 200.0]},
        }))
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.press", "arguments": {"button": "middle"},
        }))
        for phase in ("queued-press", "applied-held"):
            before = pointer_states(server, session_id)
            before_mouse = server.read_resource(session_id, MOUSE)
            before_phase_ticks = server.read_resource(session_id, TICKS)
            for command, arguments, code in (
                ("input.pointer.move_to", {"position": [-1.0, 20.0]}, "pointer_position_out_of_bounds"),
                ("input.pointer.move_to", {"position": [20.0, -1.0]}, "pointer_position_out_of_bounds"),
                ("input.pointer.move_to", {"position": [1440.0, 20.0]}, "pointer_position_out_of_bounds"),
                ("input.pointer.move_to", {"position": [20.0, 900.0]}, "pointer_position_out_of_bounds"),
                ("input.pointer.press", {"button": "woodpecker_invalid_button"}, "invalid_pointer_button"),
                ("input.pointer.release", {"button": "woodpecker_invalid_button"}, "invalid_pointer_button"),
                ("input.pointer.press", {"button": "middle"}, "pointer_button_already_pressed"),
                ("input.pointer.release", {"button": "left"}, "pointer_button_not_pressed"),
            ):
                pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                    "command": command, "arguments": arguments,
                }))
                outcome = server.result(session_id, pending)
                self.assertEqual(outcome["kind"], "rejected", outcome)
                self.assertEqual(outcome["error"]["code"], code, outcome)
                rejections.append({"phase": phase, "command": command, "arguments": arguments, "outcome": outcome})
            self.assertEqual(pointer_states(server, session_id), before)
            self.assertEqual(server.read_resource(session_id, MOUSE), before_mouse)
            self.assertEqual(server.read_resource(session_id, TICKS), before_phase_ticks)
            server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            active, = [item for item in pointer_states(server, session_id) if item["location"] is not None]
            self.assertEqual(active["location"]["position"], [100.0, 200.0])
            self.assertEqual(active["buttons"], {"primary": False, "secondary": False, "middle": True})
            self.assertEqual(server.read_resource(session_id, MOUSE), {
                "pressed": ["Middle"], "just_pressed": ["Middle"] if phase == "queued-press" else [],
                "just_released": [],
            })
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.release", "arguments": {"button": "middle"},
        }))
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "input.pointer.release", "arguments": {"button": "middle"},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "pointer_button_not_pressed")
        rejections.append({"phase": "queued-release", "outcome": outcome})
        self.assertEqual(len(rejections), 19)
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], ["Middle"])
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": [300.0, 400.0]},
        }))
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 2)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, MOUSE),
                         {"pressed": [], "just_pressed": [], "just_released": ["Middle"]})
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        final, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
        self.assertEqual(final["location"]["position"], [300.0, 400.0])
        self.assertEqual(final["buttons"], {"primary": False, "secondary": False, "middle": False})
        self.assertEqual(server.read_resource(session_id, MOUSE), empty_mouse)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 4)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
        server.record(["observation", "invalid-pointer"], rejections=rejections, final=final, action_ticks=4)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
