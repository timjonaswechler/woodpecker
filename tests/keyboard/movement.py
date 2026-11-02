"""Virtual press, hold and release cause real movement only on explicit ticks."""
import os
import unittest

from tests._shared import game


class KeyboardMovementTest(unittest.TestCase):
    def test_press_hold_and_release_control_player_movement(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "Prepared shared-or-standalone Signalstation fixture: up to 120 readiness ticks/10 s, "
            "then F9 and three reset ticks. This case uses 8 action ticks. "
            "Virtual inputs only, no OS input. Permission required: WOODPECKER_TEST_RENDERED=1",
        )
        server, session_id = game(self, "keyboard-movement")
        ticks_type = "test_app::woodpecker::SimulationTicks"
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
