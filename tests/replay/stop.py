"""Replay Stop prevents the tail while retaining completed prefix work and queued input."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.recording import read_jsonl, write_jsonl


class StopReplayTest(unittest.TestCase):
    def test_stop_drains_the_active_warp_keeps_prefix_and_never_submits_tail(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window. Replay: two prefix ticks, ShiftLeft press, "
                         "120 ticks at 2/s stopped early; planned release/one-tick tail must not run. "
                         "Two explicit cleanup ticks after Stop; maximum 124 ticks, no OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "replay-stop")
        configuration = load_config()
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
        config_path = server.artifacts / "config.toml"
        write_config(config_path, configuration)
        session_id = server.cli("session", "create", "--config", str(config_path))["id"]
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
        root = Path(detail["artifact_dir"])
        ticks_type = "test_app::woodpecker::SimulationTicks"
        keys_type = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": 0})
        rows = [
            {"type": "recording_started", "format_version": 2,
             "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}},
            {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 2},
             "outcome": {"status": "completed", "output": {"requested_ticks": 2, "executed_ticks": 2, "outcome": "completed"}}},
            {"type": "command", "command": "input.keyboard.press", "arguments": {"key": "shift_left"},
             "outcome": {"status": "completed", "output": None}},
            {"type": "command", "command": "tick.warp.start", "arguments": {
                "ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}},
             "outcome": {"status": "completed", "output": {"requested_ticks": 120, "executed_ticks": 120, "outcome": "completed"}}},
            {"type": "command", "command": "input.keyboard.release", "arguments": {"key": "shift_left"},
             "outcome": {"status": "completed", "output": None}},
            {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 1},
             "outcome": {"status": "completed", "output": {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}}},
            {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 5},
        ]
        write_jsonl(root / "recordings/plan.jsonl", rows)
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/observed.jsonl"}})
        replay = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "replay.start", "arguments": {"path": "recordings/plan.jsonl"},
        }))
        # Internal replay commands are not public Pending Activity. Observe the
        # independent app counter without submitting forbidden in-replay queries.
        deadline = time.monotonic() + 15
        while True:
            probe = root / "simulation-ticks.json"
            reached = json.loads(probe.read_text())["ticks"] if probe.exists() else 0
            if reached >= 3:  # Two prefix ticks plus at least one long-Warp tick.
                break
            if time.monotonic() >= deadline:
                self.fail("Replay did not reach the long Warp")
            time.sleep(0.03)
        self.assertLess(reached, 122)
        stop = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "replay.stop", "arguments": {},
        }))
        stopped = server.result(session_id, stop)
        self.assertEqual(stopped["kind"], "completed", stopped)
        self.assertEqual(stopped["output"], {"was_running": True})
        outcome = server.result(session_id, replay)
        self.assertEqual(outcome["kind"], "completed", outcome)
        self.assertEqual(outcome["output"], {"outcome": {"kind": "stopped"}})
        self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                         {"path": "recordings/observed.jsonl", "recorded_commands": 4})
        observed = read_jsonl(root / "recordings/observed.jsonl")
        self.assertEqual([row["command"] for row in observed[1:-1]],
                         ["tick.warp.start", "input.keyboard.press", "tick.warp.start", "tick.warp.stop"])
        self.assertEqual(observed[1]["outcome"], {"status": "completed", "output": {
            "requested_ticks": 2, "executed_ticks": 2, "outcome": "completed"}})
        long = observed[3]["outcome"]["output"]
        self.assertEqual(long["requested_ticks"], 120)
        self.assertEqual(long["outcome"], "stopped")
        self.assertGreaterEqual(long["executed_ticks"], reached - 2)
        self.assertLess(long["executed_ticks"], 120)
        self.assertEqual(observed[4]["outcome"], {"status": "completed", "output": {"was_running": True}})
        executed = 2 + long["executed_ticks"]
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": executed})
        self.assertEqual(server.call(session_id, {"command": "replay.stop", "arguments": {}}), {"was_running": False})
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": executed})
        # A queued prefix Press survives Stop even if the long Warp executed zero
        # ticks; apply it explicitly, then release explicitly, outside the replay.
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, keys_type)["pressed"], ["ShiftLeft"])
        server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "shift_left"}})
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, keys_type)["pressed"], [])
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": executed + 2})
        self.assertEqual(read_jsonl(root / "recordings/plan.jsonl"), rows)
        server.record(["observation", "replay-stop"], replay=outcome, stop=stopped,
                      executed_before_cleanup=executed, recorded_commands=4, cleanup_ticks=2)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
