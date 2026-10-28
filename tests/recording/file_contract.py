"""A real recording preserves acceptance order, typed outcomes and only recordable work."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.recording import read_jsonl


class RecordingFileContractTest(unittest.TestCase):
    def test_jsonl_has_exact_boundaries_acceptance_order_and_typed_uncut_outcomes(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window with tracing diagnostics. F12 press/one tick/"
                         "release intentionally produces a local nonfatal error report. A 60-tick "
                         "Warp at 2/s is stopped early (maximum 61 ticks total); ShiftLeft inputs, "
                         "two expected game rejections. No OS input or retries. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "recording-file-contract", env={"RUST_LOG": "error"})
        configuration = load_config()
        configuration["launch"]["arguments"].insert(0, "--woodpecker-diagnostics")
        configuration["report"]["tracing_errors"] = True
        config = server.artifacts / "diagnostics.toml"
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
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
        seed = json.loads((ROOT / "tests/settings.json").read_text())
        relative = "recordings/contract.jsonl"
        self.assertEqual(server.call(session_id, {"command": "recording.start", "arguments": {"path": relative}}),
                         {"path": relative})
        recording_cursor = server.cli("session", "poll", session_id)["cursor"]
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f12"}}))
        self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}}),
                         {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "f12"}}))
        cursor = recording_cursor
        events = []
        deadline = time.monotonic() + 10
        while True:
            args = ["session", "poll", session_id, "--wait-ms", "100"]
            if cursor is not None:
                args += ["--cursor", json.dumps(cursor)]
            activity = server.cli(*args)
            self.assertEqual(activity["kind"], "activity", activity)
            cursor = activity["cursor"]
            events.extend(entry["event"] for entry in activity["entries"])
            if any(event["kind"] == "report" for event in events):
                break
            if time.monotonic() >= deadline:
                self.fail("The deliberate tracing event did not produce a local report")
        self.assertTrue(any(event["kind"] == "event" for event in events))
        warp_args = {"ticks": 60, "pace": {"kind": "ticks_per_second", "target": 2.0}}
        warp = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": warp_args,
        }))
        settings_type = "test_app::app::settings::Settings"
        query_args = {"source": "resources", "selector": {"kind": "type", "type_path": settings_type},
                      "projection": {"kind": "value"}}
        query = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "inspect.query", "arguments": query_args,
        }))
        query_result = server.result(session_id, query)
        expected_query = {"items": [{"kind": "resource", "result": {"kind": "value", "type_path": settings_type,
                          "value": {"status": "readable", "value": seed}}}]}
        self.assertEqual(query_result["kind"], "completed", query_result)
        self.assertEqual(query_result["output"], expected_query)
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "shift_left"},
        }))
        duplicate = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "input.keyboard.press", "arguments": {"key": "shift_left"},
        }))
        duplicate_result = server.result(session_id, duplicate)
        self.assertEqual(duplicate_result["kind"], "rejected", duplicate_result)
        self.assertEqual(duplicate_result["error"]["code"], "key_already_pressed")
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "shift_left"},
        }))
        screenshot = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "screenshot.capture", "arguments": {"path": "not-image.txt"},
        }))
        screenshot_result = server.result(session_id, screenshot)
        self.assertEqual(screenshot_result["kind"], "rejected", screenshot_result)
        self.assertEqual(screenshot_result["error"]["code"], "invalid_screenshot_path")
        activity = server.cli("session", "poll", session_id)
        terminals = [entry["event"] for entry in activity["entries"]
                     if entry["event"]["kind"] in ("completed", "rejected", "failed")]
        self.assertTrue(any(event["request_id"] == query["request_id"] for event in terminals))
        self.assertFalse(any(event["request_id"] == warp["request_id"] for event in terminals))
        self.assertEqual(server.call(session_id, {"command": "tick.warp.stop", "arguments": {}}), {"was_running": True})
        warp_result = server.result(session_id, warp)
        self.assertEqual(warp_result["kind"], "completed", warp_result)
        self.assertEqual(warp_result["output"]["requested_ticks"], 60)
        self.assertEqual(warp_result["output"]["outcome"], "stopped")
        self.assertLess(warp_result["output"]["executed_ticks"], 60)
        # A successful host-side replay control is excluded too, even while recording.
        self.assertEqual(server.call(session_id, {"command": "replay.stop", "arguments": {}}), {"was_running": False})
        self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                         {"path": relative, "recorded_commands": 10})
        path = root / relative
        rows = read_jsonl(path)
        self.assertTrue(path.read_bytes().endswith(b"\n"))
        self.assertEqual(len(rows), 12)
        self.assertEqual(rows[0], {"type": "recording_started", "format_version": 2,
                                  "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}})
        self.assertEqual(rows[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 10})
        expected = [
            ("input.keyboard.press", {"key": "f12"}, {"status": "completed", "output": None}),
            ("tick.warp.start", {"ticks": 1}, {"status": "completed", "output": {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}}),
            ("input.keyboard.release", {"key": "f12"}, {"status": "completed", "output": None}),
            ("tick.warp.start", warp_args, {"status": "completed", "output": warp_result["output"]}),
            ("inspect.query", query_args, {"status": "completed", "output": expected_query}),
            ("input.keyboard.press", {"key": "shift_left"}, {"status": "completed", "output": None}),
            ("input.keyboard.press", {"key": "shift_left"}, {"status": "rejected", "error": duplicate_result["error"]}),
            ("input.keyboard.release", {"key": "shift_left"}, {"status": "completed", "output": None}),
            ("screenshot.capture", {"path": "not-image.txt"}, {"status": "rejected", "error": screenshot_result["error"]}),
            ("tick.warp.stop", {}, {"status": "completed", "output": {"was_running": True}}),
        ]
        for entry, (command, arguments, outcome) in zip(rows[1:-1], expected):
            self.assertEqual(entry, {"type": "command", "command": command,
                                     "arguments": arguments, "outcome": outcome})
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"),
                         {"ticks": 1 + warp_result["output"]["executed_ticks"]})
        original = path.read_bytes()
        server.record(["observation", "recording-file-contract"], recorded_commands=10,
                      earlier_query_completed_before_warp=True, events_and_reports_observed=True)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertEqual(path.read_bytes(), original)
