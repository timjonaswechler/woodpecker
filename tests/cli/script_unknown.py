"""External wire faults preserve known script results without retry or hidden continuation."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from tests._support import CLI, ROOT, Server


class ScriptUnknownTest(unittest.TestCase):
    def _cleanup_fixture(self, process, artifacts):
        (artifacts / "stop").write_text("CLI finished\n")
        forced = False
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            forced = True
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
        (artifacts / "fixture-cleanup.json").write_text(json.dumps({
            "pid": process.pid, "returncode": process.returncode, "forced": forced}) + "\n")
        self.assertFalse(forced, "External wire peer required forced cleanup")
        self.assertEqual(process.returncode, 0, f"Wire fixture failed; see {artifacts}")

    def test_connection_correlation_and_gap_abort_without_losing_known_results_or_retrying(self):
        self.assertTrue(CLI.is_file(), "Build the CLI first")
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        session_id = "1" * 32
        commands = [
            {"command": "tick.warp.stop", "arguments": {}},
            {"command": "tick.warp.start", "arguments": {"ticks": 60}},
            {"command": "input.keyboard.press", "arguments": {"key": "shift_left"}},
            {"command": "input.keyboard.release", "arguments": {"key": "shift_left"}},
            {"command": "tick.warp.start", "arguments": {"ticks": 1}},
        ]
        for mode, unconfirmed, diagnostic in (
            ("drop-submit", True, "SubmissionUnknown"),
            ("wrong-submit-call", True, "uncorrelated response"),
            ("wrong-poll-call", False, "uncorrelated response"),
            ("gap", False, "Activity gap"),
            ("wrong-command", False, "uncorrelated command"),
        ):
            artifacts = Path(tempfile.mkdtemp(prefix=f"script-{mode}-", dir=runs))
            self.artifact_dirs = [*getattr(self, "artifact_dirs", []), str(artifacts)]
            print(f"\nArtefakte: {artifacts}", flush=True)
            path = artifacts / "script.json"
            path.write_text(json.dumps({"version": 1, "commands": commands}) + "\n")
            stdout = self.enterContext((artifacts / "fixture.stdout.log").open("wb"))
            stderr = self.enterContext((artifacts / "fixture.stderr.log").open("wb"))
            # Explicit external peer start; this never creates a real server session or game.
            process = subprocess.Popen(
                [sys.executable, str(ROOT / "tests/faults/script_server.py"), mode, str(artifacts), session_id],
                cwd=ROOT, stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr, start_new_session=True)
            self.addCleanup(self._cleanup_fixture, process, artifacts)
            deadline = time.monotonic() + 5
            while True:
                with (artifacts / "fixture.stdout.log").open() as output:
                    line = output.readline(65537)
                self.assertLessEqual(len(line), 65536)
                if line.endswith("\n"):
                    break
                self.assertIsNone(process.poll(), f"Peer exited before Ready; see {artifacts}")
                if time.monotonic() >= deadline:
                    self.fail("Wire peer did not become ready")
                time.sleep(0.02)
            ready = json.loads(line)
            self.assertEqual(ready["pid"], process.pid)
            self.assertEqual(ready["mode"], mode)
            client = Server(process, artifacts)  # Transport/log helper, no product objects mocked.
            client.address = ready["address"]
            result = client.run_cli("session", "script", session_id, "--file", str(path))
            (artifacts / "stop").write_text("CLI finished\n")
            self.assertEqual(process.wait(timeout=5), 0, f"Peer conversation failed; see {artifacts}")
            self.assertEqual(result.returncode, 1, (mode, result.stdout, result.stderr))
            self.assertIn("script failed", json.loads(result.stderr)["error"])
            outcome = json.loads(result.stdout)
            self.assertEqual(set(outcome), {"kind", "completed", "failures"})
            self.assertEqual(outcome["kind"], "failed")
            self.assertEqual(outcome["completed"], [{
                "command_index": 0, "request_id": 1, "command": commands[0],
                "output": {"was_running": False}}])
            self.assertEqual([row["command_index"] for row in outcome["failures"]], [1, 2, 3, 4])
            for row in outcome["failures"]:
                index = row["command_index"]
                self.assertEqual(set(row), {"command_index", "request_id", "command", "reason"})
                self.assertEqual(row["command"], commands[index])
                self.assertEqual(row["request_id"], 2 if index == 1 else None)
                expected = "unknown" if index == 1 or (unconfirmed and index == 2) else "not_submitted"
                self.assertEqual(row["reason"]["kind"], expected, (mode, row))
                self.assertEqual(set(row["reason"]), {"kind", "message"})
                self.assertIn(diagnostic, row["reason"]["message"], (mode, row))
            journal = [json.loads(line) for line in (artifacts / "wire.jsonl").read_text().splitlines()]
            self.assertEqual(sum(row["kind"] == "connection" for row in journal), 1)
            self.assertFalse(any(row["kind"] == "unexpected_connection" for row in journal),
                             "No reconnect/retry after uncertain submission or Activity loss")
            self.assertEqual([row["mode"] for row in journal if row["kind"] == "fault_injected"], [mode])
            self.assertEqual(journal[-1]["kind"], "finished")
            operations = [row["value"]["operation"]["kind"] for row in journal if row["kind"] == "request"]
            self.assertEqual(operations, ["snapshot", "submit", "poll", "submit", "poll"] +
                             (["submit"] if unconfirmed else []), "No implicit Gap/correlation recovery RPC")
            self.assertFalse(any(row["kind"] == "unexpected_request" for row in journal))
            submitted = [row["value"]["operation"]["command"] for row in journal
                         if row["kind"] == "request" and row["value"]["operation"]["kind"] == "submit"]
            self.assertEqual(submitted, commands[:3 if unconfirmed else 2],
                             "Each accepted prefix once; no tail, resubmission, Stop or Shutdown")
            self.assertEqual(sum(row["kind"] == "unconfirmed_receipt" for row in journal), int(unconfirmed))
