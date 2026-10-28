"""Relative motion retains full deltas while the virtual cursor clamps independently."""
import os
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.pointer import TICKS, pointer_states


class RelativeMotionTest(unittest.TestCase):
    def test_relative_motion_moves_fpv_and_clamps_only_the_virtual_position(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One window, up to 3600 engine ticks/60 s and 120 readiness ticks/10 s, "
                         "then nine action ticks. Virtual v press/release, relative deltas including "
                         "zero and both window edges, one absolute move; no OS cursor input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "pointer-relative-motion")
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
        motion_type = "bevy_input::mouse::AccumulatedMouseMotion"
        view_type = "test_app::player::camera::ViewState"
        player_type = "test_app::player::Player"
        settings = server.read_resource(session_id, "test_app::app::settings::Settings")
        self.assertEqual(settings["sensitivity"], 1.0)
        self.assertFalse(settings["invert_y"])
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        pointers = pointer_states(server, session_id)
        self.assertTrue(pointers)
        self.assertTrue(all(item["location"] is None for item in pointers))
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_by", "arguments": {"delta": [10.0, -20.0]},
        }))
        self.assertEqual(pointer_states(server, session_id), pointers)
        self.assertEqual(server.read_resource(session_id, motion_type), {"delta": [0.0, 0.0]})
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        active, = [item for item in pointer_states(server, session_id) if item["location"] is not None]
        self.assertEqual(active["location"]["position"], [730.0, 430.0])  # center + first delta
        self.assertEqual(server.read_resource(session_id, motion_type), {"delta": [10.0, -20.0]})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "v"},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "v"},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "FirstPerson")
        self.assertTrue(server.read_resource(session_id, "test_app::input::PointerCapture")["active"])
        self.assertAlmostEqual(server.read_resource(session_id, player_type)["yaw"], 0.0, delta=1e-6)
        # Worked values use the seed sensitivity and the existing game's 0.0025
        # radians/pixel, not the observed motion or SDK cursor-clamping calculation.
        observations = []
        for deltas, expected_motion, expected_position, yaw, pitch in (
            ([[12.0, -8.0], [-4.0, 2.0]], [8.0, -6.0], [738.0, 424.0], -0.02, 0.015),
            ([[0.0, 0.0]], [0.0, 0.0], [738.0, 424.0], -0.02, 0.015),
            ([[2000.0, 1200.0]], [2000.0, 1200.0], [1440.0, 900.0], -5.02, -1.45),
            ([[-4000.0, -3000.0]], [-4000.0, -3000.0], [0.0, 0.0], 4.98, 1.45),
        ):
            before = pointer_states(server, session_id)
            before_motion = server.read_resource(session_id, motion_type)
            before_player = server.read_resource(session_id, player_type)
            before_view = server.read_resource(session_id, view_type)
            before_phase_ticks = server.read_resource(session_id, TICKS)
            for delta in deltas:
                self.assertIsNone(server.call(session_id, {
                    "command": "input.pointer.move_by", "arguments": {"delta": delta},
                }))
            self.assertEqual(pointer_states(server, session_id), before)
            self.assertEqual(server.read_resource(session_id, motion_type), before_motion)
            self.assertEqual(server.read_resource(session_id, player_type), before_player)
            self.assertEqual(server.read_resource(session_id, view_type), before_view)
            self.assertEqual(server.read_resource(session_id, TICKS), before_phase_ticks)
            server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            pointer, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
            position = pointer["location"]["position"]
            for actual, expected, upper in zip(position, expected_position, (1440.0, 900.0)):
                if expected == upper:
                    # Within 0.001 logical pixels of the exclusive edge, without
                    # prescribing the implementation\'s exact f32 predecessor.
                    self.assertGreaterEqual(actual, upper - 0.001)
                    self.assertLess(actual, upper)
                else:
                    self.assertAlmostEqual(actual, expected, delta=3e-5)
            self.assertTrue(0 <= position[0] < 1440 and 0 <= position[1] < 900, position)
            self.assertEqual(server.read_resource(session_id, motion_type), {"delta": expected_motion})
            self.assertAlmostEqual(server.read_resource(session_id, player_type)["yaw"], yaw, delta=1e-5)
            self.assertAlmostEqual(server.read_resource(session_id, view_type)["pitch"], pitch, delta=1e-6)
            observations.append({"deltas": deltas, "position": position, "expected_position_or_edge": expected_position,
                                 "yaw": yaw, "pitch": pitch})
        # Absolute placement must not invent relative MouseMotion or turn FPV.
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": [600.0, 400.0]},
        }))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, motion_type), {"delta": [0.0, 0.0]})
        self.assertAlmostEqual(server.read_resource(session_id, player_type)["yaw"], 4.98, delta=1e-5)
        self.assertAlmostEqual(server.read_resource(session_id, view_type)["pitch"], 1.45, delta=1e-6)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        pointer, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
        self.assertEqual(pointer["location"]["position"], [600.0, 400.0])
        self.assertEqual(server.read_resource(session_id, motion_type), {"delta": [0.0, 0.0]})
        self.assertAlmostEqual(server.read_resource(session_id, player_type)["yaw"], 4.98, delta=1e-5)
        self.assertAlmostEqual(server.read_resource(session_id, view_type)["pitch"], 1.45, delta=1e-6)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 9)
        server.record(["observation", "relative-motion"], first_position=[730.0, 430.0],
                      phases=observations, final_pointer=pointer, action_ticks=9)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
