"""Server stop drains independent sessions and an owned held report without new admission."""
import json
import os
from pathlib import Path
import signal
import tempfile
import time
import unittest

from tests._support import ROOT, WINDOW_POSITIONS, load_config, owned_process, start_server, write_config
from tests.providers import gh_fixture
from tests.recording import read_jsonl
from tests.reports import observe, reports
from tests.shutdown import owned_game, progress, require_observation_budget, thaw, wait_state


class ServerStopTest(unittest.TestCase):
    def test_stopped_slow_session_does_not_block_other_session_or_report_and_admission_closes(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Two manual game windows. A: recording/120-tick Warp at 2/s stopped early; "
                         "B: F12/one tick/local fake-gh report. SIGSTOP only verified owned A, "
                         "server stop with 20s deadline; B closes independently, release fake gh, "
                         "then SIGCONT A. Maximum 121 ticks; no desktop input or real issues. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="server-stop-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        gh = fixture / "gh"
        environment = gh_fixture(gh)
        environment["RUST_LOG"] = "error"
        (gh / "mode.json").write_text('{"mode":"hold"}\n')
        server = start_server(self, "server-stop-multiple", env=environment, shutdown_seconds=20)
        self.addCleanup((gh / "release").write_text, "cleanup release\n")
        sessions = []
        for index in range(2):
            configuration = load_config()
            configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe",
                "--window-position", f"{WINDOW_POSITIONS[index][0]},{WINDOW_POSITIONS[index][1]}"]
            configuration["report"].update(tracing_errors=index == 1, provider={"kind": "github" if index == 1 else "local"})
            config = server.artifacts / f"session-{index}.toml"
            write_config(config, configuration)
            created = server.cli("session", "create", "--config", str(config))
            detail = wait_state(server, created["id"], "SessionReady", timeout=30)
            sessions.append((created["id"], Path(detail["artifact_dir"])))
            owned_process(server, detail["artifact_dir"])
        (a, root_a), (b, root_b) = sessions
        for session_id, root in sessions:
            server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/active.jsonl"}})
        server.cli("session", "submit", a, "--command", json.dumps({"command": "tick.warp.start", "arguments": {
            "ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}}}))
        self.assertLess(progress(root_a), 120)
        server.call(b, {"command": "input.keyboard.press", "arguments": {"key": "f12"}})
        server.call(b, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        server.call(b, {"command": "input.keyboard.release", "arguments": {"key": "f12"}})
        deadline = time.monotonic() + 15
        while not (gh / "entered.json").exists():
            if time.monotonic() >= deadline:
                self.fail("Fake gh did not enter hold")
            time.sleep(0.02)
        gh_pid = json.loads((gh / "entered.json").read_text())["pid"]
        provider = {"pid": gh_pid, "group": os.getpgid(gh_pid)}
        holder = {"server": server, "root": root_a}
        self.addCleanup(thaw, holder)
        pid_a = owned_game(server, root_a)
        os.kill(pid_a, signal.SIGSTOP)
        before_stop = server.cli("session", "poll", b)["cursor"]
        stop_before = time.monotonic()
        self.assertEqual(server.cli("server", "stop"), {"state": "Stopping"})
        stop_interval = (stop_before, time.monotonic())
        require_observation_budget(self, server, stop_interval, 20, provider=provider)
        new_session = server.run_cli("session", "create", "--config", str(ROOT / "tests/app.toml"))
        self.assertEqual(new_session.returncode, 1)
        self.assertIn("server_stopping", new_session.stderr)
        for session_id in (a, b):
            require_observation_budget(self, server, stop_interval, 20, provider=provider)
            rejected = server.run_cli("session", "submit", session_id, "--command",
                                      '{"command":"tick.warp.stop","arguments":{}}')
            self.assertEqual(rejected.returncode, 1)
            reply = json.loads(rejected.stdout)
            self.assertEqual(reply["code"], "server_stopping")
            self.assertNotIn("request_id", reply)
        require_observation_budget(self, server, stop_interval, 20, provider=provider)
        self.assertEqual(wait_state(server, b, "Ended")["error"], None)
        require_observation_budget(self, server, stop_interval, 20, provider=provider)
        self.assertEqual(server.cli("session", "inspect", a)["state"], "Stopping")
        self.assertIsNone(server.process.poll(), "Slow A and pending provider must still be owned")
        self.assertFalse((gh / "release").exists())
        b_rows = read_jsonl(root_b / "recordings/active.jsonl")
        b_names = [row["command"] for row in b_rows[1:-1]]
        self.assertIn(b_names, [["input.keyboard.press", "tick.warp.start", "input.keyboard.release"],
                               ["input.keyboard.press", "tick.warp.start", "input.keyboard.release", "tick.warp.stop"]])
        if len(b_names) == 4:
            self.assertEqual(b_rows[4]["outcome"]["output"], {"was_running": False})
        self.assertEqual(b_rows[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": len(b_names)})
        # Keep A frozen while the provider finishes so its result remains observable.
        require_observation_budget(self, server, stop_interval, 20, provider=provider)
        (gh / "release").write_text("explicit provider release\n")
        events = observe(server, b, lambda e: len(reports(e)) >= 1, cursor=before_stop)
        self.assertEqual(len(reports(events)), 1)
        self.assertEqual(reports(events)[0]["result"]["status"], "submitted")
        self.assertEqual(reports(events)[0]["result"]["outcome"], {"kind": "created", "reference": {
            "kind": "issue", "reference": {"identifier": "12", "url": "https://example.invalid/issues/12"}}})
        self.assertEqual(server.cli("session", "inspect", a)["state"], "Stopping")
        require_observation_budget(self, server, stop_interval, 20, provider=provider)
        os.kill(owned_game(server, root_a), signal.SIGCONT)
        holder.clear()
        self.assertEqual(server.process.wait(timeout=15), 0)
        rows = read_jsonl(root_a / "recordings/active.jsonl")
        self.assertEqual([row["command"] for row in rows[1:-1]], ["tick.warp.start", "tick.warp.stop"])
        self.assertEqual(rows[1]["outcome"]["output"]["outcome"], "stopped")
        self.assertLess(rows[1]["outcome"]["output"]["executed_ticks"], 120)
        self.assertEqual(rows[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 2})
        self.assertEqual(json.loads((root_a / "simulation-ticks.json").read_text()),
                         {"ticks": rows[1]["outcome"]["output"]["executed_ticks"]})
        for session_id, root in sessions:
            with self.assertRaises(ProcessLookupError):
                os.kill(json.loads((root / "process-probe.json").read_text())["pid"], 0)
        calls = [json.loads(line) for line in (gh / "calls.jsonl").read_text().splitlines()]
        self.assertEqual(len(calls), 2)
        self.assertEqual(calls[1]["body"]["title"], "Signalstation diagnostic tracing error")
