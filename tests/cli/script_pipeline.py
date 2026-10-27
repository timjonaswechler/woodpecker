"""A script pipelines ordinary work, respects boundary drains and preserves original indices."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server
from tests.recording import read_jsonl


class ScriptPipelineTest(unittest.TestCase):
    def test_pipeline_boundaries_rejection_and_shutdown_keep_full_indexed_results(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window. Read-only passing script, then mixed script: "
                         "two ticks at 2/s before recording, "
                         "60-tick Warp at 2/s stopped early, Inspect, one intentional invalid-key "
                         "rejection, one final tick, Recording Stop and normal Shutdown. Maximum "
                         "63 ticks; no accepted input or OS input. Permission required: "
                         "WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "cli-script-pipeline")
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
        root = Path(detail["artifact_dir"])
        ticks_type = "test_app::woodpecker::SimulationTicks"
        settings_type = "test_app::app::settings::Settings"
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": 0})
        seed = json.loads((ROOT / "tests/settings.json").read_text())
        relative = "recordings/script-pipeline.jsonl"
        commands = [
            {"command": "tick.warp.start", "arguments": {
                "ticks": 2, "pace": {"kind": "ticks_per_second", "target": 2.0}}},
            {"command": "recording.start", "arguments": {"path": relative}},
            {"command": "tick.warp.start", "arguments": {
                "ticks": 60, "pace": {"kind": "ticks_per_second", "target": 2.0}}},
            {"command": "inspect.query", "arguments": {"source": "resources",
                "selector": {"kind": "type", "type_path": settings_type}, "projection": {"kind": "value"}}},
            {"command": "input.keyboard.press", "arguments": {"key": "woodpecker_invalid_key"}},
            {"command": "tick.warp.stop", "arguments": {}},
            {"command": "tick.warp.start", "arguments": {"ticks": 1}},
            {"command": "recording.stop", "arguments": {}},
            {"command": "inspect.query", "arguments": {"source": "resources",
                "selector": {"kind": "type", "type_path": ticks_type}, "projection": {"kind": "value"}}},
            {"command": "shutdown", "arguments": {}},
        ]
        settings_output = {"items": [{"kind": "resource", "result": {"kind": "value",
            "type_path": settings_type, "value": {"status": "readable", "value": seed}}}]}
        passing = server.artifacts / "passing.json"
        passing.write_text(json.dumps({"version": 1, "commands": [commands[3]]}) + "\n")
        positive = server.run_cli("session", "script", session_id, "--file", str(passing))
        self.assertEqual(positive.returncode, 0, (positive.stdout, positive.stderr))
        self.assertEqual(positive.stderr, "")
        passed = json.loads(positive.stdout)
        self.assertEqual(set(passed), {"kind", "completed"})
        self.assertEqual(passed["kind"], "passed")
        self.assertEqual(len(passed["completed"]), 1)
        retained = passed["completed"][0]
        self.assertEqual(set(retained), {"command_index", "request_id", "command", "output"})
        self.assertEqual(retained["command_index"], 0)
        self.assertIsInstance(retained["request_id"], int)
        self.assertGreater(retained["request_id"], 0)
        self.assertEqual(retained["command"], commands[3])
        self.assertEqual(retained["output"], settings_output)
        path = server.artifacts / "pipeline.json"
        path.write_text(json.dumps({"version": 1, "commands": commands}) + "\n")
        cursor = server.cli("session", "poll", session_id)["cursor"]
        # A missing ordinary pipeline would wait 30s for the long Warp; never retry it.
        result = server.run_cli("session", "script", session_id, "--file", str(path), timeout=20)
        self.assertEqual(result.returncode, 1, (result.stdout, result.stderr))
        self.assertIn("script failed", json.loads(result.stderr)["error"])
        outcome = json.loads(result.stdout)
        self.assertEqual(set(outcome), {"kind", "completed", "failures"})
        self.assertEqual(outcome["kind"], "failed")
        self.assertEqual([row["command_index"] for row in outcome["completed"]], [0, 1, 2, 3, 5, 6, 7, 8, 9])
        self.assertEqual([row["command_index"] for row in outcome["failures"]], [4])
        completed = {row["command_index"]: row for row in outcome["completed"]}
        failure = outcome["failures"][0]
        self.assertEqual(set(failure), {"command_index", "request_id", "command", "reason"})
        self.assertEqual(failure["command"], commands[4])
        self.assertEqual(set(failure["reason"]), {"kind", "code", "message"})
        self.assertEqual(failure["reason"]["kind"], "rejected")
        self.assertEqual(failure["reason"]["code"], "invalid_key")
        self.assertTrue(failure["reason"]["message"].strip())
        for index, row in completed.items():
            self.assertEqual(set(row), {"command_index", "request_id", "command", "output"})
            self.assertEqual(row["command"], commands[index])
        rows_by_index = {**completed, 4: failure}
        ids = [rows_by_index[index]["request_id"] for index in range(len(commands))]
        self.assertTrue(all(isinstance(value, int) and value > 0 for value in ids))
        self.assertEqual(ids[:-1], sorted(set(ids[:-1])))
        self.assertEqual(ids[-1], 18446744073709551615)
        self.assertEqual(completed[0]["output"], {
            "requested_ticks": 2, "executed_ticks": 2, "outcome": "completed"})
        self.assertEqual(completed[1]["output"], {"path": relative})
        stopped = completed[2]["output"]
        self.assertEqual(set(stopped), {"requested_ticks", "executed_ticks", "outcome"})
        self.assertEqual(stopped["requested_ticks"], 60)
        self.assertEqual(stopped["outcome"], "stopped")
        self.assertGreaterEqual(stopped["executed_ticks"], 0)
        self.assertLess(stopped["executed_ticks"], 60)
        self.assertEqual(completed[3]["output"], settings_output)
        self.assertEqual(completed[5]["output"], {"was_running": True})
        self.assertEqual(completed[6]["output"], {
            "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertEqual(completed[7]["output"], {"path": relative, "recorded_commands": 5})
        self.assertEqual(completed[8]["output"], {"items": [{"kind": "resource", "result": {
            "kind": "value", "type_path": ticks_type, "value": {"status": "readable",
            "value": {"ticks": 3 + stopped["executed_ticks"]}}}}]})
        self.assertIsNone(completed[9]["output"])
        activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
        self.assertEqual(activity["kind"], "activity")
        events = [entry["event"] for entry in activity["entries"]]
        accepted = [event["request_id"] for event in events if event["kind"] == "pending"]
        self.assertEqual(accepted, ids)
        terminal_ids = [event["request_id"] for event in events
                        if event["kind"] in ("completed", "rejected", "failed")]
        self.assertEqual(sorted(terminal_ids), sorted(ids))
        # Outcomes already available in the same server pass may be published
        # in acceptance order. Pipeline means no ordinary-command drain before
        # submitting Inspect/Stop, not a guaranteed order of their Completions.
        warp_terminal = next(i for i, event in enumerate(events)
                             if event.get("request_id") == ids[2]
                             and event["kind"] in ("completed", "rejected", "failed"))
        for index in (3, 5):
            submitted = next(i for i, event in enumerate(events)
                             if event.get("request_id") == ids[index] and event["kind"] == "pending")
            self.assertLess(submitted, warp_terminal,
                            f"Command {index} must be accepted before the Warp outcome, not after a hidden drain")
        # Public Activity positions, not response-list sorting, prove boundary order.
        for earlier, later in ((0, 1), (2, 7), (3, 7), (4, 7), (5, 7), (6, 7), (7, 9), (8, 9)):
            terminal = next(i for i, event in enumerate(events)
                            if event.get("request_id") == ids[earlier]
                            and event["kind"] in ("completed", "rejected", "failed"))
            pending = next(i for i, event in enumerate(events)
                           if event.get("request_id") == ids[later] and event["kind"] == "pending")
            self.assertLess(terminal, pending, (earlier, later))
        recording = read_jsonl(root / relative)
        self.assertEqual(recording[0], {"type": "recording_started", "format_version": 2,
                                       "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}})
        self.assertEqual(recording[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 5})
        expected = []
        for index in range(2, 7):
            recorded_outcome = ({"status": "rejected", "error": {
                "code": failure["reason"]["code"], "message": failure["reason"]["message"]}}
                if index == 4 else {"status": "completed", "output": completed[index]["output"]})
            expected.append({"type": "command", **commands[index], "outcome": recorded_outcome})
        self.assertEqual(recording[1:-1], expected)
        deadline = time.monotonic() + 10
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "Ended":
                break
            self.assertEqual(detail["state"], "Stopping", detail)
            if time.monotonic() >= deadline:
                self.fail("Script shutdown did not end the session")
            time.sleep(0.05)
        self.assertIsNone(server.process.poll(), "Script shutdown must not stop the server")
