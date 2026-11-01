"""New protocol failures block a replay; historical outcomes are never its oracle."""
import json
from pathlib import Path
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture
from tests.recording import read_jsonl, write_jsonl


class BlockedReplayTest(unittest.TestCase):
    def test_new_protocol_failure_blocks_tail_but_historical_failure_does_not_predict_new_outcomes(self):
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="replay-blocked-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        manifest, environment = protocol_fixture(fixture / "peer-package")
        for mode, expected_kind, count in (("bad-output", "blocked", 1), ("replay-ok", "completed", 5)):
            holder = {}
            self.addCleanup(cleanup_peer, holder)
            server = start_server(self, f"replay-blocked-{mode}", env=environment)
            configuration = load_config()
            configuration["launch"].update(
                manifest_path=str(manifest), package="woodpecker-protocol-fixture",
                target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=[mode],
            )
            config = server.artifacts / "peer.toml"
            write_config(config, configuration)
            self.assertEqual(load_config(config)["launch"], configuration["launch"])
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
                    self.fail("External peer did not become ready")
                time.sleep(0.02)
            rows = [
                {"type": "recording_started", "format_version": 2,
                 "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}},
                {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 1},
                 "outcome": {"status": "protocol_failed", "error": {
                     "code": "old_protocol_error", "message": "Historical failure is not an expectation"}}},
                {"type": "command", "command": "input.keyboard.press", "arguments": {"key": "shift_left"},
                 "outcome": {"status": "rejected", "error": {
                     "code": "old_rejection", "message": "Historical rejection is not an expectation"}}},
                {"type": "command", "command": "input.keyboard.release", "arguments": {"key": "shift_left"},
                 "outcome": {"status": "io_failed", "error": {"message": "Historical I/O is not an expectation"}}},
                {"type": "command", "command": "input.pointer.move_to", "arguments": {"position": [100.0, 200.0]},
                 "outcome": {"status": "unanswered"}},
                {"type": "command", "command": "input.pointer.press", "arguments": {"button": "left"},
                 "outcome": {"status": "completed", "output": None}},
                {"type": "recording_ended", "outcome": "session_ended", "recorded_commands": 5},
            ]
            write_jsonl(root / "recordings/plan.jsonl", rows)
            server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/current.jsonl"}})
            output = server.call(session_id, {"command": "replay.start", "arguments": {"path": "recordings/plan.jsonl"}})
            self.assertEqual(output["outcome"]["kind"], expected_kind, output)
            if mode == "bad-output":
                self.assertEqual(output["outcome"]["code"], "command_protocol_failed")
                self.assertTrue(output["outcome"]["message"])
            else:
                self.assertEqual(output, {"outcome": {"kind": "completed"}})
            self.assertEqual(server.call(session_id, {"command": "recording.stop", "arguments": {}}),
                             {"path": "recordings/current.jsonl", "recorded_commands": count})
            current = read_jsonl(root / "recordings/current.jsonl")
            self.assertEqual([row["command"] for row in current[1:-1]],
                             [row["command"] for row in rows[1:1 + count]])
            if mode == "bad-output":
                self.assertEqual(current[1]["outcome"]["status"], "protocol_failed")
                self.assertEqual(current[1]["outcome"]["error"]["code"], "invalid_response")
            else:
                self.assertEqual(current[1]["outcome"], {"status": "completed", "output": {
                    "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}})
                self.assertTrue(all(row["outcome"] == {"status": "completed", "output": None}
                                    for row in current[2:-1]))
            self.assertEqual(server.call(session_id, {"command": "replay.stop", "arguments": {}}),
                             {"was_running": False})
            self.assertEqual(read_jsonl(root / "recordings/plan.jsonl"), rows)
            journal = [json.loads(line) for line in (root / "protocol-peer.jsonl").read_text().splitlines()]
            plan_commands = [entry["value"]["command"] for entry in journal if entry["kind"] == "stdin"]
            self.assertEqual(plan_commands, [row["command"] for row in rows[1:1 + count]])
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0)
            journal = [json.loads(line) for line in (root / "protocol-peer.jsonl").read_text().splitlines()]
            sent = [entry["value"]["command"] for entry in journal if entry["kind"] == "stdin"]
            self.assertEqual(sent[:count], plan_commands)
            self.assertEqual(sent[-1], "shutdown")
            self.assertTrue(all(command in ("tick.warp.stop", "shutdown") for command in sent[count:]))
            server.record(["observation", "replay-blocked"], mode=mode, replay=output,
                          actual_recorded_commands=count, peer_commands=sent)
