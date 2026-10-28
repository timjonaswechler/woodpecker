"""Active replay rejects external game/recording work and a competing replay."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.pointer import pointer_states
from tests.recording import read_jsonl, write_jsonl


class ReplayExclusivityTest(unittest.TestCase):
    def test_external_work_cannot_enter_the_plan_or_change_inputs_during_replay(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window. A replay Warp requests 120 ticks at 2/s, "
                         "stopped early. Nine external requests deliberately rejected while busy; "
                         "later g press/release without ticks checks unpoisoned input state. "
                         "Maximum 120 ticks, no OS input. Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "replay-exclusivity")
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
        settings_type = "test_app::app::settings::Settings"
        keys_type = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
        before_settings = server.read_resource(session_id, settings_type)
        before_keys = server.read_resource(session_id, keys_type)
        before_pointer = pointer_states(server, session_id)
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": 0})
        rows = [
            {"type": "recording_started", "format_version": 2,
             "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}},
            {"type": "command", "command": "tick.warp.start", "arguments": {
                "ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}},
             "outcome": {"status": "completed", "output": {
                 "requested_ticks": 120, "executed_ticks": 120, "outcome": "completed"}}},
            {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 1},
        ]
        write_jsonl(root / "recordings/plan.jsonl", rows)
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/observed.jsonl"}})
        replay = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "replay.start", "arguments": {"path": "recordings/plan.jsonl"},
        }))
        deadline = time.monotonic() + 15
        while True:
            probe = root / "simulation-ticks.json"
            reached = json.loads(probe.read_text())["ticks"] if probe.exists() else 0
            if reached >= 1:
                break
            if time.monotonic() >= deadline:
                self.fail("Replay Warp did not advance the independent app counter")
            time.sleep(0.03)
        self.assertLess(reached, 120)
        rejected = []
        for command, arguments, code in (
            ("inspect.query", {"source": "resources", "selector": {"kind": "type", "type_path": settings_type},
                               "projection": {"kind": "value"}}, "replay_in_progress"),
            ("input.keyboard.press", {"key": "g"}, "replay_in_progress"),
            ("input.pointer.move_to", {"position": [720.0, 450.0]}, "replay_in_progress"),
            ("tick.warp.start", {"ticks": 1}, "replay_in_progress"),
            ("tick.warp.stop", {}, "replay_in_progress"),
            ("recording.start", {"path": "recordings/forbidden.jsonl"}, "replay_in_progress"),
            ("recording.stop", {}, "replay_in_progress"),
            ("screenshot.capture", {"path": "screenshots/forbidden.png"}, "replay_in_progress"),
            ("replay.start", {"path": "recordings/plan.jsonl"}, "replay_already_running"),
        ):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": command, "arguments": arguments,
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], code, outcome)
            rejected.append(outcome)
        self.assertEqual(server.call(session_id, {"command": "replay.stop", "arguments": {}}), {"was_running": True})
        outcome = server.result(session_id, replay)
        self.assertEqual(outcome["kind"], "completed", outcome)
        self.assertEqual(outcome["output"], {"outcome": {"kind": "stopped"}})
        self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                         {"path": "recordings/observed.jsonl", "recorded_commands": 2})
        observed = read_jsonl(root / "recordings/observed.jsonl")
        self.assertEqual([row["command"] for row in observed[1:-1]], ["tick.warp.start", "tick.warp.stop"])
        long = observed[1]["outcome"]["output"]
        self.assertEqual(long["requested_ticks"], 120)
        self.assertEqual(long["outcome"], "stopped")
        self.assertGreaterEqual(long["executed_ticks"], reached)
        self.assertLess(long["executed_ticks"], 120)
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": long["executed_ticks"]})
        self.assertEqual(server.read_resource(session_id, settings_type), before_settings)
        self.assertEqual(server.read_resource(session_id, keys_type), before_keys)
        self.assertEqual(pointer_states(server, session_id), before_pointer)
        self.assertFalse((root / "recordings/forbidden.jsonl").exists())
        self.assertFalse((root / "screenshots/forbidden.png").exists())
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "g"}}))
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "g"}}))
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": long["executed_ticks"]})
        server.record(["observation", "replay-exclusivity"], rejections=rejected, replay=outcome,
                      observed_commands=2, executed_ticks=long["executed_ticks"])
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
