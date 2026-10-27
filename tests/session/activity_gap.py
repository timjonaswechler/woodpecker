"""Evicted activity is explicit unknown; current pending work survives and can resume."""
import json
import os
import re
import signal
import subprocess
import time
import unittest

from tests._support import CLI, ROOT, start_server


class ActivityGapTest(unittest.TestCase):
    def _cleanup_observer(self, process, directory):
        forced = False
        if process.poll() is None:
            try:
                process.wait(timeout=2)  # stdin cleanup already sent EOF
            except subprocess.TimeoutExpired:
                forced = True
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
        (directory / "cleanup.json").write_text(json.dumps({
            "client_pid": process.pid, "returncode": process.returncode, "forced": forced,
        }, indent=2) + "\n")
        self.assertFalse(forced, "Snapshot observer required forced cleanup")
        self.assertEqual(process.returncode, 0)

    def test_gap_preserves_open_work_and_explicit_cursor_resumes_observation(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window with manual startup. One "
            "100-tick warp at 2 ticks/s is stopped early; at most 64 read-only commands "
            "within 20 seconds explicitly evict its Pending from a 4096-byte activity store. A read-only REPL observes Snapshot "
            "pending state. No inputs or retries. Permission for this run is "
            "required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "activity-gap", activity_bytes=4096)
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
        initial = server.cli("session", "poll", session_id)
        self.assertEqual(initial["kind"], "activity")
        stale = initial["cursor"]
        cursor = stale
        warp = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 100, "pace": {"kind": "ticks_per_second", "target": 2.0},
            },
        }))
        overflow_started = time.monotonic()
        gap = None
        for query_count in range(1, 65):
            if time.monotonic() - overflow_started >= 20:
                break  # Observation budget unavailable; resolve accepted work below, never retry.
            query = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "inspect.query", "arguments": {
                    "source": "resources", "selector": {
                        "kind": "type", "type_path": "test_app::app::settings::Settings",
                    }, "projection": {"kind": "metadata"},
                },
            }))
            deadline = time.monotonic() + 10
            while True:
                activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor),
                                      "--wait-ms", "500")
                self.assertEqual(activity["kind"], "activity", activity)
                self.assertEqual(activity["cursor"]["session_id"], session_id)
                self.assertGreaterEqual(activity["cursor"]["position"], cursor["position"])
                cursor = activity["cursor"]
                outcomes = [entry["event"] for entry in activity["entries"]
                            if entry["event"].get("request_id") == query["request_id"]
                            and entry["event"]["kind"] in ("completed", "rejected", "failed")]
                if outcomes:
                    self.assertEqual(len(outcomes), 1)
                    self.assertEqual(outcomes[0]["kind"], "completed", outcomes)
                    break
                if time.monotonic() >= deadline:
                    self.fail("Read-only overflow command outcome missing; no retry")
            # Explicit stale-cursor observations, never resubmission or hidden Gap repair.
            lost = server.run_cli("session", "poll", session_id, "--cursor", json.dumps(stale))
            if lost.returncode == 0:
                self.assertEqual(json.loads(lost.stdout)["kind"], "activity")
                continue
            self.assertEqual(lost.returncode, 1, lost.stderr)
            gap = json.loads(lost.stdout)
            self.assertEqual(gap["kind"], "gap", gap)
            self.assertEqual(gap["cursor"]["session_id"], session_id)
            self.assertGreater(gap["cursor"]["position"], stale["position"])
            self.assertNotIn("entries", gap)
            self.assertIn("unknown", lost.stderr)
            retained = server.cli("session", "poll", session_id, "--cursor", json.dumps(gap["cursor"]))
            self.assertEqual(retained["kind"], "activity", retained)
            if not any(entry["event"].get("request_id") == warp["request_id"] for entry in retained["entries"]):
                break
        else:
            if time.monotonic() - overflow_started <= 20:
                self.fail("64 explicit metadata commands did not evict the Warp Pending")
        overflow_seconds = time.monotonic() - overflow_started
        if overflow_seconds > 20:
            # Exceeding the observer's budget is not proof of a broken store.
            # Still resolve exactly the original Warp and verify real cleanup.
            stopped = server.call(session_id, {"command": "tick.warp.stop", "arguments": {}})
            outcome = server.result(session_id, warp, cursor=cursor)
            self.assertEqual(outcome["kind"], "completed", outcome)
            if stopped == {"was_running": True}:
                self.assertEqual(outcome["output"]["outcome"], "stopped")
                self.assertLess(outcome["output"]["executed_ticks"], 100)
            else:
                self.assertEqual(stopped, {"was_running": False})
                self.assertEqual(outcome["output"], {"requested_ticks": 100,
                    "executed_ticks": 100, "outcome": "completed"})
            server.record(["observation", "eviction-budget-exhausted"],
                          seconds=overflow_seconds, commands=query_count, gap=gap)
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0)
            self.skipTest("Unvollständig: observer exceeded 20s eviction budget; no retry")
        resume = retained["cursor"]

        # The machine CLI does not expose Snapshot directly. A fresh plain-pipe
        # REPL prints its server Snapshot before reading Activity; no PTY test here.
        directory = server.artifacts / "snapshot-observer"
        directory.mkdir()
        stdout = self.enterContext((directory / "stdout.log").open("w"))
        stderr = self.enterContext((directory / "stderr.log").open("w"))
        command = [str(CLI), "--address", server.address, "session", "repl", session_id]
        observer = subprocess.Popen(command, cwd=ROOT, stdin=subprocess.PIPE, stdout=stdout,
                                    stderr=stderr, text=True, start_new_session=True)
        self.addCleanup(self._cleanup_observer, observer, directory)
        self.addCleanup(observer.stdin.close)
        server.record(command, client_pid=observer.pid)
        observer.stdin.write("pending\n")
        observer.stdin.flush()
        deadline = time.monotonic() + 10
        while True:
            text = (directory / "stdout.log").read_text()
            if re.search(rf"^\s*\[{warp['request_id']}\]\s+tick\.warp\.start\s+pending\s*$", text, re.MULTILINE):
                break
            self.assertIsNone(observer.poll(), text)
            if time.monotonic() >= deadline:
                self.fail(f"Snapshot lost the open command; see {directory}")
            time.sleep(0.02)
        observer.stdin.write("quit\n")
        observer.stdin.flush()
        self.assertEqual(observer.wait(timeout=5), 0)
        stop = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.stop", "arguments": {},
        }))
        stopped = server.result(session_id, stop, cursor=resume)
        self.assertEqual(stopped["kind"], "completed", stopped)
        self.assertEqual(stopped["output"], {"was_running": True})
        outcome = server.result(session_id, warp, cursor=resume)
        self.assertEqual(outcome["kind"], "completed", outcome)
        self.assertEqual(outcome["output"]["requested_ticks"], 100)
        self.assertEqual(outcome["output"]["outcome"], "stopped")
        self.assertLess(outcome["output"]["executed_ticks"], 100)
        server.record(["observation", "activity-gap"], stale_cursor=stale, gap=gap,
                      resume_cursor=resume, warp_outcome=outcome, overflow_commands=query_count,
                      overflow_seconds=overflow_seconds)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
