"""A bad recording is rejected in full before any apparently valid prefix is submitted."""
import copy
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server
from tests.recording import write_jsonl


class InvalidRecordingTest(unittest.TestCase):
    def _state(self, server, session_id):
        return {name: server.read_resource(session_id, path) for name, path in (
            ("ticks", "test_app::woodpecker::SimulationTicks"),
            ("time", "bevy_time::time::Time<()>"),
            ("keys", "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"),
            ("view", "test_app::player::camera::ViewState"),
            ("settings", "test_app::app::settings::Settings"),
        )}

    def test_invalid_files_paths_and_symlinks_never_execute_a_valid_looking_prefix(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, zero ticks. Invalid version/JSONL/commands/"
                         "outcomes/paths/symlinks must not submit the valid g-press/one-tick prefix. "
                         "Two empty valid plans and final g press/release without ticks; no OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "replay-invalid-recording")
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
        before = self._state(server, session_id)
        self.assertEqual(before["ticks"], {"ticks": 0})
        header = {"type": "recording_started", "format_version": 2,
                  "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}}
        prefix = [
            header,
            {"type": "command", "command": "input.keyboard.press", "arguments": {"key": "g"},
             "outcome": {"status": "completed", "output": None}},
            {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 1},
             "outcome": {"status": "completed", "output": {
                 "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}}},
        ]
        footer = {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 2}
        valid = [*prefix, footer]
        encode = lambda rows: "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows).encode("utf-8")
        variants = []
        variants.append(("missing-header", encode(valid[1:]), "invalid_recording"))
        variants.append(("missing-footer", encode(prefix), "invalid_recording"))
        variants.append(("broken-end", encode(prefix) + b"{broken\n", "invalid_recording"))
        variants.append(("blank-line", encode(prefix) + b"\n" + encode([footer]), "invalid_recording"))
        variants.append(("duplicate-header", encode([header, *valid]), "invalid_recording"))
        variants.append(("after-footer", encode(valid) + encode([footer]), "invalid_recording"))
        for label, index, update, code in (
            ("unsupported-version", 0, {"format_version": 999}, "unsupported_recording_version"),
            ("bad-version-type", 0, {"format_version": "2"}, "invalid_recording"),
            ("header-field", 0, {"extra": True}, "invalid_recording"),
            ("footer-count", -1, {"recorded_commands": 1}, "invalid_recording"),
            ("footer-field", -1, {"extra": True}, "invalid_recording"),
            ("unknown-command", 1, {"command": "invented.command"}, "invalid_recording"),
            ("control-command", 1, {"command": "shutdown", "arguments": {}}, "invalid_recording"),
            ("command-arguments", 2, {"arguments": {"ticks": "one"}}, "invalid_recording"),
            ("wrong-output", 1, {"outcome": {"status": "completed", "output": {"was_running": False}}}, "invalid_recording"),
            ("missing-output", 1, {"outcome": {"status": "completed"}}, "invalid_recording"),
            ("unanswered-stopped", 1, {"outcome": {"status": "unanswered"}}, "invalid_recording"),
            ("bad-outcome", 1, {"outcome": {"status": "invented"}}, "invalid_recording"),
            ("warp-count", 2, {"outcome": {"status": "completed", "output": {
                "requested_ticks": 1, "executed_ticks": 2, "outcome": "completed"}}}, "invalid_recording"),
            ("clock-mismatch", 0, {"clock": {"kind": "fixed", "seconds_per_tick": 0.02}}, "replay_clock_mismatch"),
        ):
            rows = copy.deepcopy(valid)
            rows[index].update(update)
            variants.append((label, encode(rows), code))
        v1 = [{"type": "recording_started", "format_version": 1}, *valid[1:]]
        variants.append(("version-one-clock", encode(v1), "replay_clock_mismatch"))
        future = {**header, "format_version": 999}
        variants.append(("unsupported-before-broken-tail", encode([future]) + b"{broken\n",
                         "unsupported_recording_version"))
        duplicate = ('{"type":"command","command":"input.keyboard.press",'
                     '"arguments":{"key":"g","key":"g"},"outcome":{"status":"completed","output":null}}\n')
        variants.append(("duplicate-nested-field", encode([header]) + duplicate.encode() +
                         encode(valid[2:]), "invalid_recording"))
        variants.append(("invalid-utf8", encode(prefix) + b"\xff\n", "io"))
        paths = []
        for label, data, code in variants:
            path = root / "recordings" / (label + ".jsonl")
            path.parent.mkdir(exist_ok=True)
            path.write_bytes(data)
            paths.append((label, str(path.relative_to(root)), code))
        outside = server.artifacts / "outside-replay"
        outside.mkdir()
        write_jsonl(outside / "escape.jsonl", valid)
        (root / "recordings/link.jsonl").symlink_to(os.path.relpath(outside / "escape.jsonl", root / "recordings"))
        (root / "recordings/link-dir").symlink_to(os.path.relpath(outside, root / "recordings"),
                                                 target_is_directory=True)
        (root / "recordings/directory.jsonl").mkdir()
        paths.extend([
            ("empty-path", "", "invalid_recording_path"),
            ("absolute-path", str(outside / "escape.jsonl"), "invalid_recording_path"),
            ("traversal", "../escape.jsonl", "invalid_recording_path"),
            ("backslash", "recordings\\escape.jsonl", "invalid_recording_path"),
            ("extension", "recordings/escape.txt", "invalid_recording_path"),
            ("directory", "recordings/directory.jsonl", "invalid_recording_path"),
            ("file-symlink", "recordings/link.jsonl", "invalid_recording_path"),
            ("parent-symlink", "recordings/link-dir/escape.jsonl", "invalid_recording_path"),
            ("missing-file", "recordings/missing.jsonl", "io"),
        ])
        observations = []
        for label, path, code in paths:
            cursor = server.cli("session", "poll", session_id)["cursor"]
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "replay.start", "arguments": {"path": path},
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "failed" if code == "io" else "rejected", (label, outcome))
            self.assertEqual(outcome["error"]["code"], code, (label, outcome))
            activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
            self.assertEqual(activity["kind"], "activity", activity)
            submitted = [entry["event"]["command"] for entry in activity["entries"]
                         if entry["event"]["kind"] == "pending"]
            self.assertEqual(submitted, ["replay.start"], (label, activity))
            self.assertFalse(any(entry["event"]["kind"] == "event" for entry in activity["entries"]))
            self.assertEqual(self._state(server, session_id), before, label)
            observations.append({"variant": label, "path": path, "outcome": outcome})
        # Legal empty files (with and without final LF) prove recovery after load errors.
        empty = [header, {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 0}]
        for final_lf in (True, False):
            data = encode(empty)
            (root / "recordings/empty.jsonl").write_bytes(data if final_lf else data.rstrip(b"\n"))
            self.assertEqual(server.call(session_id, {
                "command": "replay.start", "arguments": {"path": "recordings/empty.jsonl"},
            }), {"outcome": {"kind": "completed"}})
        # Even a queued, not-yet-applied leaked prefix must not poison accepted key state.
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "g"},
        }))
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "g"},
        }))
        self.assertEqual(self._state(server, session_id), before)
        server.record(["observation", "invalid-recording"], variants=observations, application_ticks=0)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
