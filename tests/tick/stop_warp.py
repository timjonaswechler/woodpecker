"""Explicit stop completes a paced or maximum-speed warp without further ticks."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class StopWarpTest(unittest.TestCase):
    def test_stop_finishes_active_warps_and_freezes_simulation(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window with manual startup. It requests "
            "600 ticks at 10 ticks/s and then 50000 maximum-speed ticks, stopping each "
            "early; no inputs or automatic retries. Permission for this run is "
            "required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "stop-warp")
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
        inconclusive = []
        for label, requested, pace in (
            ("paced", 600, {"kind": "ticks_per_second", "target": 10.0}),
            ("maximum-speed", 50000, {"kind": "as_fast_as_possible"}),
        ):
            before = server.read_resource(session_id, ticks_type)["ticks"]
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "tick.warp.start", "arguments": {"ticks": requested, "pace": pace},
            }))
            self.assertEqual(pending["kind"], "pending", pending)
            # Both variants must execute work before stop is submitted. Otherwise
            # accepting start+stop in one intake batch would not prove yielding.
            minimum = 2 if label == "paced" else 1
            deadline = time.monotonic() + 10
            while server.read_resource(session_id, ticks_type)["ticks"] - before < minimum:
                if time.monotonic() >= deadline:
                    self.fail(f"{label} warp made no observable progress; see {server.artifacts}")
                time.sleep(0.02)
            # Natural completion before this stop is not proof of interruptibility;
            # it must not be retried automatically with a bigger tick budget.
            stop = server.call(session_id, {
                "command": "tick.warp.stop", "arguments": {},
            })
            self.assertIn(stop, ({"was_running": True}, {"was_running": False}), label)
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "completed", outcome)
            output = outcome["output"]
            self.assertEqual(output["requested_ticks"], requested, label)
            if stop["was_running"]:
                self.assertEqual(output["outcome"], "stopped", label)
                self.assertGreaterEqual(output["executed_ticks"], minimum, label)
                self.assertLess(output["executed_ticks"], requested, label)
            else:
                # Only a fully accounted natural completion is an unavailable
                # stop precondition. Lost/ignored Stop or partial work still fails.
                self.assertEqual(output, {"requested_ticks": requested,
                    "executed_ticks": requested, "outcome": "completed"}, label)
                inconclusive.append(label)
            stopped_ticks = server.read_resource(session_id, ticks_type)
            stopped_time = server.read_resource(session_id, time_type)
            self.assertEqual(stopped_ticks["ticks"] - before, output["executed_ticks"], label)
            server.record(["observation", "stopped"], variant=label, output=output,
                          application_ticks=stopped_ticks, time=stopped_time)
            server.record(["client", "pause"], seconds=0.25, variant=label)
            time.sleep(0.25)
            self.assertEqual(server.read_resource(session_id, ticks_type), stopped_ticks, label)
            self.assertEqual(server.read_resource(session_id, time_type), stopped_time, label)

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        if inconclusive:
            self.skipTest(f"Unvollständig: Warp naturally completed before Stop: {inconclusive}; no retry")
