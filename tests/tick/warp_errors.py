"""Invalid warp work is rejected without corrupting the active warp or idle state."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class WarpErrorsTest(unittest.TestCase):
    def test_invalid_work_preserves_active_warp_and_idle_stop_is_noop(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window with manual startup. It requests "
            "100 ticks at 2 ticks/s, rejects zero ticks, two invalid start paces, "
            "a second active start and two invalid SetPace values; then explicitly "
            "stops, sends idle stop and completes one fresh tick (maximum 101 ticks). "
            "No inputs. Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "warp-errors")
        session_id = server.cli("session", "create", "--config", str(ROOT / "tests/app.toml"))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not become ready; see {server.artifacts}")
            time.sleep(0.05)

        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        before = server.read_resource(session_id, ticks_type)["ticks"]
        idle_time = server.read_resource(session_id, time_type)
        # Test bad start values in idle: combining them with an already-running
        # warp would assert an undocumented ordering of two rejection reasons.
        for arguments, code in (
            ({"ticks": 0}, "invalid_tick_count"),
            ({"ticks": 1, "pace": {"kind": "ticks_per_second", "target": 0.0}}, "invalid_pace"),
            ({"ticks": 1, "pace": {"kind": "ticks_per_second", "target": -1.0}}, "invalid_pace"),
        ):
            rejected = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "tick.warp.start", "arguments": arguments,
            }))
            outcome = server.result(session_id, rejected)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], code, outcome)
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"], before)
        self.assertEqual(server.read_resource(session_id, time_type), idle_time)

        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 100, "pace": {"kind": "ticks_per_second", "target": 2.0},
            },
        }))
        self.assertEqual(pending["kind"], "pending", pending)
        deadline = time.monotonic() + 10
        while server.read_resource(session_id, ticks_type)["ticks"] == before:
            if time.monotonic() >= deadline:
                self.fail(f"Initial warp made no progress; see {server.artifacts}")
            time.sleep(0.02)

        for command, arguments, code in (
            ("tick.warp.start", {"ticks": 1}, "warp_already_running"),
            ("tick.warp.set_pace", {
                "pace": {"kind": "ticks_per_second", "target": 0.0},
            }, "invalid_pace"),
            ("tick.warp.set_pace", {
                "pace": {"kind": "ticks_per_second", "target": -1.0},
            }, "invalid_pace"),
        ):
            rejected = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": command, "arguments": arguments,
            }))
            outcome = server.result(session_id, rejected)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], code, outcome)

        # A successful rejection alone could hide cancellation or a poisoned pace.
        # Observe fresh progress after the three active-state rejections.
        after_errors = server.read_resource(session_id, ticks_type)["ticks"]
        deadline = time.monotonic() + 10
        while server.read_resource(session_id, ticks_type)["ticks"] < after_errors + 2:
            if time.monotonic() >= deadline:
                self.fail(f"Rejected work damaged the active warp; see {server.artifacts}")
            time.sleep(0.02)
        self.assertEqual(server.call(session_id, {
            "command": "tick.warp.stop", "arguments": {},
        }), {"was_running": True})
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "completed", outcome)
        self.assertEqual(outcome["output"]["outcome"], "stopped", outcome)
        stopped_ticks = server.read_resource(session_id, ticks_type)
        self.assertEqual(stopped_ticks["ticks"] - before, outcome["output"]["executed_ticks"])
        stopped_time = server.read_resource(session_id, time_type)
        self.assertEqual(server.call(session_id, {
            "command": "tick.warp.stop", "arguments": {},
        }), {"was_running": False})
        self.assertEqual(server.read_resource(session_id, ticks_type), stopped_ticks)
        self.assertEqual(server.read_resource(session_id, time_type), stopped_time)

        # Valid work with no request pace must still be usable after the errors;
        # invalid SetPace must not poison the runtime default either.
        self.assertEqual(server.call(session_id, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"], stopped_ticks["ticks"] + 1)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
