"""Virtual press, hold and release cause real movement only on explicit ticks."""
import os
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class KeyboardMovementTest(unittest.TestCase):
    def test_press_hold_and_release_control_player_movement(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window, allows up to 3600 engine-startup "
            "and 120 game-readiness ticks, then sends arrow_right and eight explicit ticks. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "keyboard-movement")
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
                self.fail(f"Engine did not become ready; see {server.artifacts}")
            time.sleep(0.05)
        ticks_type = "test_app::woodpecker::SimulationTicks"
        startup_ticks = server.read_resource(session_id, ticks_type)["ticks"]
        server.record(["preparation", "engine-ready"], ticks=startup_ticks)

        # Game readiness is checked separately. Every additional tick stays visible.
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

        player_type = "test_app::player::Player"
        initial = server.read_resource(session_id, player_type)["position"]
        before_press_ticks = server.read_resource(session_id, ticks_type)
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "arrow_right"},
        }))
        self.assertEqual(server.read_resource(session_id, ticks_type), before_press_ticks)
        self.assertEqual(server.read_resource(session_id, player_type)["position"], initial)

        output = server.call(session_id, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        })
        self.assertEqual(output, {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - before_press_ticks["ticks"], 1)
        after_one = server.read_resource(session_id, player_type)["position"]
        self.assertGreater(after_one[0], initial[0])
        self.assertAlmostEqual(after_one[1], initial[1], delta=1e-6)

        # No second press. Observe every held tick, not just the net displacement:
        # a dropped middle tick must not hide behind movement on another tick.
        after_hold = after_one
        for action_tick in range(2, 5):
            output = server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            })
            self.assertEqual(output, {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
            self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - before_press_ticks["ticks"],
                             action_tick)
            position = server.read_resource(session_id, player_type)["position"]
            self.assertGreater(position[0], after_hold[0], f"Held key lost on action tick {action_tick}")
            self.assertAlmostEqual(position[1], initial[1], delta=1e-6)
            server.record(["observation", "held-key"], action_tick=action_tick,
                          before=after_hold, after=position)
            after_hold = position

        before_release_ticks = server.read_resource(session_id, ticks_type)
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "arrow_right"},
        }))
        self.assertEqual(server.read_resource(session_id, ticks_type), before_release_ticks)
        self.assertEqual(server.read_resource(session_id, player_type)["position"], after_hold)
        output = server.call(session_id, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        })
        self.assertEqual(output, {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - before_press_ticks["ticks"], 5)
        self.assertEqual(server.read_resource(session_id, player_type)["position"], after_hold)
        output = server.call(session_id, {
            "command": "tick.warp.start", "arguments": {"ticks": 3},
        })
        self.assertEqual(output, {"requested_ticks": 3, "executed_ticks": 3, "outcome": "completed"})
        self.assertEqual(server.read_resource(session_id, player_type)["position"], after_hold)
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - before_press_ticks["ticks"], 8)
        server.record(["observation", "movement-ticks"], action_ticks=8)

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
