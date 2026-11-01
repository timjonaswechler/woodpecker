"""Scroll reaches Bevy in signed line units, accumulates once and clears on idle."""
import os
import time
import unittest

from tests._support import ROOT, start_server
from tests.pointer import MOUSE, TICKS, pointer_states


class PointerScrollTest(unittest.TestCase):
    def test_signed_horizontal_vertical_zero_and_batched_scroll_are_line_deltas(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, exactly six ticks. Pointer at [720,850], "
                         "positive/negative horizontal/vertical and zero line scrolling, "
                         "two queued scrolls accumulated, then idle. No OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "pointer-scroll")
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
        scroll_type = "bevy_input::mouse::AccumulatedMouseScroll"
        zero = {"unit": "Line", "delta": [0.0, 0.0]}
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        self.assertEqual(server.read_resource(session_id, scroll_type), zero)
        initial_pointers = pointer_states(server, session_id)
        self.assertTrue(initial_pointers)
        initial_mouse = server.read_resource(session_id, MOUSE)
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": [720.0, 850.0]},
        }))
        observations = []
        # Explicit worked sums, not sums computed from the response under test.
        for deltas, expected in (
            ([[1.5, 0.0]], [1.5, 0.0]),
            ([[0.0, -2.25]], [0.0, -2.25]),
            ([[-3.0, 4.5]], [-3.0, 4.5]),
            ([[0.0, 0.0]], [0.0, 0.0]),
            ([[1.0, 2.0], [-0.25, -3.0]], [0.75, -1.0]),
        ):
            before = server.read_resource(session_id, scroll_type)
            before_phase_ticks = server.read_resource(session_id, TICKS)
            before_pointers = pointer_states(server, session_id)
            for delta in deltas:
                self.assertIsNone(server.call(session_id, {
                    "command": "input.pointer.scroll", "arguments": {"delta": delta},
                }))
            self.assertEqual(server.read_resource(session_id, scroll_type), before)
            self.assertEqual(server.read_resource(session_id, TICKS), before_phase_ticks)
            self.assertEqual(pointer_states(server, session_id), before_pointers)
            server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            actual = server.read_resource(session_id, scroll_type)
            self.assertEqual(actual, {"unit": "Line", "delta": expected})
            pointer, = [item for item in pointer_states(server, session_id) if item["location"] is not None]
            self.assertEqual(pointer["location"]["position"], [720.0, 850.0])
            self.assertEqual(pointer["buttons"], {"primary": False, "secondary": False, "middle": False})
            self.assertEqual(server.read_resource(session_id, MOUSE), initial_mouse)
            observations.append({"deltas": deltas, "applied": actual})
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, scroll_type), zero)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 6)
        server.record(["observation", "pointer-scroll"], phases=observations, idle=zero, action_ticks=6)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
