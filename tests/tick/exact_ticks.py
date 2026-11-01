"""Explicit warps advance exact application ticks and fixed time, then stay idle."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class ExactTicksTest(unittest.TestCase):
    def test_exact_ticks_advance_fixed_time_only_while_warping(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window and explicitly runs ten ticks. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "exact-ticks")

        # Manual startup keeps preparation free of implicit simulation ticks.
        session_id = server.cli(
            "session", "create", "--config", str(ROOT / "tests/app.toml"),
        )["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not reach SessionReady; see {server.artifacts}")
            time.sleep(0.05)

        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        before_ticks = server.read_resource(session_id, ticks_type)
        before_time = server.read_resource(session_id, time_type)

        # The fixture specifies 16,666,667 ns per tick. These are worked examples,
        # independent of the reported executed_ticks or observed time values.
        for ticks, pace, expected_elapsed in (
            (3, {"kind": "as_fast_as_possible"}, 0.050000001),
            (7, {"kind": "ticks_per_second", "target": 20.0}, 0.116666669),
        ):
            command = {
                "command": "tick.warp.start",
                "arguments": {"ticks": ticks, "pace": pace},
            }
            pending = server.cli("session", "submit", session_id, "--command", json.dumps(command))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome, {
                "kind": "completed",
                "request_id": pending["request_id"],
                "command": "tick.warp.start",
                "output": {
                    "requested_ticks": ticks,
                    "executed_ticks": ticks,
                    "outcome": "completed",
                },
            })

            after_ticks = server.read_resource(session_id, ticks_type)
            after_time = server.read_resource(session_id, time_type)
            self.assertEqual(after_ticks["ticks"] - before_ticks["ticks"], ticks)
            self.assertAlmostEqual(
                after_time["elapsed_secs_f64"] - before_time["elapsed_secs_f64"],
                expected_elapsed, delta=1e-12, msg=f"{ticks} ticks at {pace}",
            )
            self.assertAlmostEqual(after_time["delta_secs_f64"], 0.016666667, delta=1e-12)

            # Rendering may continue, but neither thinking time nor pacing may
            # advance simulation after a completed warp. No stop command is sent.
            server.record(["client", "pause"], seconds=0.25, after_warp_ticks=ticks)
            time.sleep(0.25)
            self.assertEqual(server.read_resource(session_id, ticks_type), after_ticks)
            self.assertEqual(server.read_resource(session_id, time_type), after_time)
            before_ticks = after_ticks
            before_time = after_time

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
