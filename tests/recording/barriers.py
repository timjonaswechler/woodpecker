"""Recording controls delimit command ownership and reject invalid active/pending states."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server
from tests.recording import read_jsonl


class RecordingBarriersTest(unittest.TestCase):
    def test_controls_reject_open_work_and_delimit_commands_around_header_and_footer(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, two 60-tick Warps at 2/s stopped early "
                         "(maximum 120 ticks), no inputs. Recording controls and read-only queries "
                         "are pipelined around file boundaries. No forced transition timing or retries. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "recording-barriers")
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
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.stop", "arguments": {},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "recording_not_active")
        earlier = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 60, "pace": {"kind": "ticks_per_second", "target": 2.0}},
        }))
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.start", "arguments": {"path": "recordings/too-early.jsonl"},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "recording_commands_pending")
        self.assertFalse((root / "recordings/too-early.jsonl").exists())
        self.assertEqual(server.call(session_id, {"command": "tick.warp.stop", "arguments": {}}), {"was_running": True})
        self.assertEqual(server.result(session_id, earlier)["output"]["outcome"], "stopped")
        cursor = server.cli("session", "poll", session_id)["cursor"]
        started = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.start", "arguments": {"path": "recordings/boundary.jsonl"},
        }))
        query = {"command": "inspect.query", "arguments": {"source": "resources",
                 "selector": {"kind": "type", "type_path": "test_app::app::settings::Settings"},
                 "projection": {"kind": "value"}}}
        inside = server.cli("session", "submit", session_id, "--command", json.dumps(query))
        self.assertEqual(server.result(session_id, started)["output"], {"path": "recordings/boundary.jsonl"})
        header_bytes = (root / "recordings/boundary.jsonl").read_bytes().split(b"\n", 1)[0]
        self.assertEqual(json.loads(header_bytes),
                         {"type": "recording_started", "format_version": 2,
                          "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}})
        self.assertEqual(server.result(session_id, inside)["kind"], "completed")
        activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
        completions = [entry["event"]["request_id"] for entry in activity["entries"]
                       if entry["event"]["kind"] == "completed"]
        self.assertLess(completions.index(started["request_id"]), completions.index(inside["request_id"]))
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.start", "arguments": {"path": "recordings/duplicate.jsonl"},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "recording_already_active")
        self.assertFalse((root / "recordings/duplicate.jsonl").exists())
        warp = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 60, "pace": {"kind": "ticks_per_second", "target": 2.0}},
        }))
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.stop", "arguments": {},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "recording_commands_pending")
        self.assertEqual(server.call(session_id, {"command": "tick.warp.stop", "arguments": {}}), {"was_running": True})
        self.assertEqual(server.result(session_id, warp)["output"]["outcome"], "stopped")
        cursor = server.cli("session", "poll", session_id)["cursor"]
        stop = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.stop", "arguments": {},
        }))
        outside = server.cli("session", "submit", session_id, "--command", json.dumps(query))
        self.assertEqual(server.result(session_id, stop)["output"],
                         {"path": "recordings/boundary.jsonl", "recorded_commands": 3})
        self.assertEqual(read_jsonl(root / "recordings/boundary.jsonl")[-1],
                         {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 3})
        self.assertEqual(server.result(session_id, outside)["kind"], "completed")
        activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
        completions = [entry["event"]["request_id"] for entry in activity["entries"]
                       if entry["event"]["kind"] == "completed"]
        self.assertLess(completions.index(stop["request_id"]), completions.index(outside["request_id"]))
        path = root / "recordings/boundary.jsonl"
        recorded = read_jsonl(path)
        self.assertEqual([row["command"] for row in recorded[1:-1]],
                         ["inspect.query", "tick.warp.start", "tick.warp.stop"])
        self.assertEqual(recorded[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 3})
        original_bytes = path.read_bytes()
        (root / "recordings/link.jsonl").symlink_to("boundary.jsonl")
        (root / "recordings/link-dir").symlink_to(".", target_is_directory=True)
        rejections = []
        for label, candidate, code in (
            ("exists", "recordings/boundary.jsonl", "recording_path_exists"),
            ("empty", "", "invalid_recording_path"),
            ("absolute", str(root / "recordings/absolute.jsonl"), "invalid_recording_path"),
            ("traversal", "recordings/../traversal.jsonl", "invalid_recording_path"),
            ("backslash", "recordings\\backslash.jsonl", "invalid_recording_path"),
            ("extension", "recordings/wrong.png", "invalid_recording_path"),
            ("file-link", "recordings/link.jsonl", "invalid_recording_path"),
            ("parent-link", "recordings/link-dir/new.jsonl", "invalid_recording_path"),
        ):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "recording.start", "arguments": {"path": candidate},
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "rejected", (label, outcome))
            self.assertEqual(outcome["error"]["code"], code, (label, outcome))
            self.assertEqual(path.read_bytes(), original_bytes)
            rejections.append({"variant": label, "outcome": outcome})
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/recovered.jsonl"}})
        self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                         {"path": "recordings/recovered.jsonl", "recorded_commands": 0})
        self.assertEqual(len(read_jsonl(root / "recordings/recovered.jsonl")), 2)
        server.record(["observation", "recording-barriers"], recorded_commands=3, path_rejections=rejections,
                      transition_races="deferred to 4.78, not claimed by this test")
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertEqual(path.read_bytes(), original_bytes)
