"""Owned server file-size faults distinguish Start I/O, active failure and end finalization."""
import json
import os
from pathlib import Path
import resource
import sys
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture
from tests.recording import read_jsonl


class RecordingIoFailureTest(unittest.TestCase):
    def test_start_active_write_and_unexpected_end_failures_keep_their_own_outcomes(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One real manual-startup window, zero ticks/accepted inputs; four intentional "
                         "text_focus_unavailable rejections with 16384-byte text payloads. "
                         "Only owned servers have a 65536-byte file soft limit; Cargo/game limits restored. "
                         "Two additional windowless peers end with pending work, one during footer I/O. "
                         "No panic, foreign files or retries. Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        parent_limit = resource.getrlimit(resource.RLIMIT_FSIZE)
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        controls = Path(tempfile.mkdtemp(prefix="recording-io-controls-", dir=runs))
        self.artifact_dirs = [str(controls)]
        limit = 65536
        launcher = ROOT / "tests/faults/file_limit.py"
        real_limit = controls / "real-limit"
        real_limit.mkdir()
        server = start_server(self, "recording-active-io", activity_bytes=16 * 1024 * 1024,
                              command_prefix=[sys.executable, launcher, "limit", str(limit), real_limit])
        session_id = server.cli("session", "create", "--config", str(ROOT / "tests/app.toml"))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail("Real app did not become ready")
            time.sleep(0.05)
        root = Path(detail["artifact_dir"])
        # A component beyond NAME_MAX yields ordinary start I/O, not a lexical path error.
        too_long = "recordings/" + "x" * 300 + ".jsonl"
        cursor = server.cli("session", "poll", session_id)["cursor"]
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.start", "arguments": {"path": too_long},
        }))
        failed_start = server.result(session_id, pending)
        self.assertEqual(failed_start["kind"], "failed", failed_start)
        self.assertEqual(failed_start["error"]["code"], "io")
        activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
        self.assertFalse(any(entry["event"]["kind"] == "event" and
                             entry["event"]["event"]["kind"] == "recording_failed" for entry in activity["entries"]))
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/active.jsonl"}})
        query = {"command": "inspect.query", "arguments": {
            "source": "resources", "selector": {"kind": "all"}, "projection": {"kind": "metadata"}}}
        # Fixed recordable arguments fill this file independently of resource
        # inventory. Without a first tick there is no editable target; the
        # recording failure must not rewrite these ordinary game rejections.
        payload = "p" * 16384
        self.assertEqual(len(payload.encode("utf-8")), 16384)
        for _ in range(4):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "input.text.input", "arguments": {"text": payload},
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], "text_focus_unavailable")
        events = []
        deadline = time.monotonic() + 10
        while True:
            activity = server.cli("session", "poll", session_id, "--wait-ms", "100", "--cursor", json.dumps(cursor))
            self.assertEqual(activity["kind"], "activity", activity)
            cursor = activity["cursor"]
            events.extend(entry["event"] for entry in activity["entries"])
            failures = [event["event"] for event in events if event["kind"] == "event" and
                        event["event"]["kind"] == "recording_failed"]
            if failures:
                break
            if time.monotonic() >= deadline:
                self.fail("Four fixed 16-KiB recording arguments did not trigger the declared file limit")
        self.assertEqual(len(failures), 1, failures)
        self.assertEqual(failures[0]["path"], "recordings/active.jsonl")
        self.assertTrue(failures[0]["message"])
        self.assertLessEqual((root / "recordings/active.jsonl").stat().st_size, limit)
        self.assertGreater((root / "recordings/active.jsonl").stat().st_size, limit - len(payload))
        for log in ("server.stdout.log", "server.stderr.log"):
            self.assertLess((server.artifacts / log).stat().st_size, limit,
                            "The declared fault must be recording I/O, not a full server log")
        self.assertNotIn(b'"type":"recording_ended"', (root / "recordings/active.jsonl").read_bytes())
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "recording.stop", "arguments": {},
        }))
        stopped = server.result(session_id, pending)
        self.assertEqual(stopped["kind"], "rejected", stopped)
        self.assertEqual(stopped["error"]["code"], "recording_not_active")
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/recovered.jsonl"}})
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 0})
        self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                         {"path": "recordings/recovered.jsonl", "recorded_commands": 1})
        self.assertEqual(read_jsonl(root / "recordings/recovered.jsonl")[-1],
                         {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 1})
        restored = [json.loads(line) for line in (real_limit / "cargo-limit-restored.jsonl").read_text().splitlines()]
        saved = json.loads((real_limit / "server-file-limit.json").read_text())
        self.assertTrue(restored)
        self.assertTrue(all(entry["limit"] == saved["saved"]["limit"] for entry in restored))
        self.assertEqual(resource.getrlimit(resource.RLIMIT_FSIZE), parent_limit)
        server.record(["observation", "recording-active-io"], start_failure=failed_start,
                      recording_failure=failures[0], rejected_game_commands=4, payload_bytes=16384, application_ticks=0,
                      file_limit=limit, cargo_and_game_limit_restored=True)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)

        # Two external peers, not games: a clean end finalization and a deterministic
        # append/footer failure with only 64 bytes left in the recording's allowance.
        manifest, environment = protocol_fixture(controls / "peer-package")
        for label, limited in (("healthy-end", False), ("failed-end", True)):
            holder = {}
            self.addCleanup(cleanup_peer, holder)
            prefix = []
            if limited:
                limit_root = controls / "end-limit"
                limit_root.mkdir()
                prefix = [sys.executable, launcher, "limit", str(limit), limit_root]
            server = start_server(self, f"recording-{label}", expected_exit_code=1,
                                  env=environment, command_prefix=prefix)
            configuration = load_config()
            configuration["launch"].update(
                manifest_path=str(manifest), package="woodpecker-protocol-fixture",
                target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=["recording-end"],
            )
            config = server.artifacts / "peer.toml"
            write_config(config, configuration)
            created = server.cli("session", "create", "--config", str(config))
            session_id = created["id"]
            root = Path(created["artifact_dir"])
            holder["path"] = root / "protocol-peer.json"
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "SessionReady":
                    break
                self.assertEqual(detail["state"], "Starting", detail)
                if time.monotonic() >= deadline:
                    self.fail("End fixture did not become ready")
                time.sleep(0.02)
            path = root / "recordings/end.jsonl"
            server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/end.jsonl"}})
            prefix_bytes = path.read_bytes()
            if limited:
                padding_entry = {"type": "command", "command": "input.text.input", "arguments": {"text": ""},
                                 "outcome": {"status": "completed", "output": None}}
                overhead = len((json.dumps(padding_entry, separators=(",", ":")) + "\n").encode())
                padding = "p" * (limit - 64 - len(prefix_bytes) - overhead)
                # This is an intentionally synthetic protocol peer, not a text-limit game test.
                self.assertIsNone(server.call(session_id, {
                    "command": "input.text.input", "arguments": {"text": padding},
                }))
                deadline = time.monotonic() + 10
                while path.stat().st_size != limit - 64:
                    if time.monotonic() >= deadline:
                        self.fail("Peer padding entry did not reach the independently calculated byte budget")
                    time.sleep(0.01)
                prefix_bytes = path.read_bytes()
            cursor = server.cli("session", "poll", session_id)["cursor"]
            warp = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "tick.warp.start", "arguments": {"ticks": 5},
            }))
            ending = server.cli("session", "submit", session_id, "--command", json.dumps(query))
            for pending in (warp, ending):
                outcome = server.result(session_id, pending)
                self.assertEqual(outcome["kind"], "failed", outcome)
                self.assertEqual(outcome["error"]["code"], "ended", outcome)
            deadline = time.monotonic() + 15
            events = []
            while True:
                activity = server.cli("session", "poll", session_id, "--wait-ms", "100", "--cursor", json.dumps(cursor))
                self.assertEqual(activity["kind"], "activity", activity)
                cursor = activity["cursor"]
                events.extend(entry["event"] for entry in activity["entries"])
                normal = [event["event"] for event in events if event["kind"] == "event"]
                if any(event["kind"] == "ended" for event in normal):
                    break
                if time.monotonic() >= deadline:
                    self.fail("Recording end did not publish its terminal event")
            kinds = [event["kind"] for event in normal]
            if limited:
                self.assertEqual(kinds.count("recording_failed"), 1, normal)
                self.assertLess(kinds.index("recording_failed"), kinds.index("ended"))
                self.assertTrue(path.read_bytes().startswith(prefix_bytes))
                self.assertLessEqual(path.stat().st_size, limit)
                self.assertNotIn(b'"type":"recording_ended"', path.read_bytes())
            else:
                self.assertNotIn("recording_failed", kinds)
                rows = read_jsonl(path)
                self.assertEqual([row["command"] for row in rows[1:-1]], ["tick.warp.start", "inspect.query"])
                self.assertTrue(all(row["outcome"] == {"status": "unanswered"} for row in rows[1:-1]))
                self.assertEqual(rows[-1], {"type": "recording_ended", "outcome": "session_ended", "recorded_commands": 2})
            server.record(["observation", "recording-end"], variant=label, events=kinds,
                          recording_bytes=path.stat().st_size)
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 1)
        self.assertEqual(resource.getrlimit(resource.RLIMIT_FSIZE), parent_limit)
