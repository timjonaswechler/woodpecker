"""One original server deadline covers sessions and provider, even after another Stop."""
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
from tests.shutdown import owned_game, progress, require_observation_budget, thaw, wait_state


class ShutdownDeadlineTest(unittest.TestCase):
    def test_one_deadline_kills_only_remaining_owned_work_without_claiming_complete_output(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Two manual game windows, each 120-tick Warp at 2/s stopped early, plus "
                         "B F12/one trigger tick (maximum 241 ticks). SIGSTOP verified owned games; "
                         "server Stop has one 12s deadline. Resume A halfway, repeat Stop, leave B/"
                         "fake gh held for product-forced cleanup and declared server exit 1. "
                         "No real issues. Permission required: WOODPECKER_TEST_RENDERED=1")
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="deadline-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        gh = fixture / "gh"
        environment = gh_fixture(gh)
        environment["RUST_LOG"] = "error"
        (gh / "mode.json").write_text('{"mode":"hold"}\n')
        server = start_server(self, "server-deadline", env=environment, expected_exit_code=1, shutdown_seconds=12)
        self.addCleanup((gh / "release").write_text, "cleanup release after assertions\n")
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
        server.call(b, {"command": "input.keyboard.press", "arguments": {"key": "f12"}})
        server.call(b, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        server.call(b, {"command": "input.keyboard.release", "arguments": {"key": "f12"}})
        deadline = time.monotonic() + 15
        while not (gh / "entered.json").exists():
            if time.monotonic() >= deadline:
                self.fail("Fake gh did not enter hold")
            time.sleep(0.02)
        holders = []
        pids = []
        for session_id, root in sessions:
            server.cli("session", "submit", session_id, "--command", json.dumps({"command": "tick.warp.start", "arguments": {
                "ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}}}))
            self.assertLess(progress(root, at_least=2 if session_id == b else 1), 120 if session_id == a else 121)
            pid = owned_game(server, root)
            holder = {"server": server, "root": root}
            holders.append(holder)
            self.addCleanup(thaw, holder)
            pids.append(pid)
            os.kill(pid, signal.SIGSTOP)
        gh_pid = json.loads((gh / "entered.json").read_text())["pid"]
        provider = {"pid": gh_pid, "group": os.getpgid(gh_pid)}
        groups = [os.getpgid(pid) for pid in [*pids, gh_pid]]
        self.assertEqual(len(set(groups)), 3, "Games/provider require separate owned process groups")
        self.assertNotIn(os.getpgrp(), groups)
        budget = 12.0
        stop_before = time.monotonic()
        self.assertEqual(server.cli("server", "stop"), {"state": "Stopping"})
        stop_after = time.monotonic()
        stop_interval = (stop_before, stop_after)
        require_observation_budget(self, server, stop_interval, budget, provider=provider)
        # The first deadline began somewhere inside this CLI interval, not
        # necessarily at its send time. Confirm both held sessions are stopping.
        wait_state(server, a, "Stopping")
        require_observation_budget(self, server, stop_interval, budget, provider=provider)
        wait_state(server, b, "Stopping")
        time.sleep(max(0, stop_before + budget / 2 - time.monotonic()))
        require_observation_budget(self, server, stop_interval, budget, provider=provider)
        self.assertIsNone(server.process.poll())
        verified = owned_game(server, root_a)
        release_before = time.monotonic()
        os.kill(verified, signal.SIGCONT)
        release_after = time.monotonic()
        holders[0].clear()
        remaining = stop_before + budget - time.monotonic()
        require_observation_budget(self, server, stop_interval, budget, provider=provider)
        self.assertGreater(remaining, 0, "First-Stop interval overlaps remaining original-deadline budget")
        self.assertEqual(wait_state(server, a, "Ended", timeout=remaining)["error"], None)
        self.assertEqual(server.cli("session", "inspect", b)["state"], "Stopping")
        self.assertFalse((gh / "release").exists())
        require_observation_budget(self, server, stop_interval, budget, provider=provider)
        repeat_before = time.monotonic()
        self.assertEqual(server.cli("server", "stop"), {"state": "Stopping"})
        repeat_after = time.monotonic()
        original_earliest = stop_before + budget
        original_latest = stop_after + budget
        # Restarting on repeated Stop, or starting B's allowance only after A,
        # cannot expire before this counterfactual lower bound. Split the two
        # hypotheses, rather than widening an arbitrary elapsed-time band.
        wrong_earliest = min(repeat_before, release_before) + budget
        cutoff = (original_latest + wrong_earliest) / 2 if original_latest < wrong_earliest else None
        last_alive = repeat_before
        observation_deadline = repeat_after + budget + 5
        while True:
            poll_before = time.monotonic()
            status = server.process.poll()
            poll_after = time.monotonic()
            if status is not None:
                exit_interval = (last_alive, poll_after)
                break
            last_alive = poll_before
            if poll_after >= observation_deadline:
                self.fail("Server did not finish within the bounded cleanup observation")
            time.sleep(0.01)
        self.assertEqual(server.process.wait(timeout=1), 1)
        self.assertGreaterEqual(exit_interval[1], original_earliest, "Server ended before its original deadline")
        if cutoff is not None:
            self.assertLess(exit_interval[0], cutoff, "A restarted/serial allowance outlived the original deadline")
        ambiguous = cutoff is None or exit_interval[1] >= cutoff
        elapsed = exit_interval[1] - stop_before
        intervals = {"first_stop": [stop_before, stop_after], "resume_a": [release_before, release_after],
                     "repeat_stop": [repeat_before, repeat_after], "exit": exit_interval,
                     "original_latest": original_latest, "wrong_earliest": wrong_earliest, "cutoff": cutoff}
        server.record(["observation", "shutdown-deadline-intervals"], **intervals)
        for pid in [*pids, gh_pid]:
            with self.assertRaises(ProcessLookupError):
                os.kill(pid, 0)
        for group in groups:
            with self.assertRaises(ProcessLookupError):
                os.killpg(group, 0)
        holders[1].clear()  # Product, not this harness, killed the still-stopped owned game.
        self.assertIn("shutdown_incomplete", (server.artifacts / "server.stderr.log").read_text(errors="replace"))
        self.assertEqual(read_jsonl(root_a / "recordings/active.jsonl")[-1]["outcome"], "stopped")
        # Cancellation may leave no footer or a session-ended/unanswered prefix, never a clean stopped footer.
        data = (root_b / "recordings/active.jsonl").read_bytes()
        complete = [json.loads(line) for line in data.split(b"\n")[:-1] if line]
        self.assertTrue(complete and complete[0]["type"] == "recording_started")
        self.assertFalse(any(row.get("type") == "recording_ended" and row.get("outcome") == "stopped" for row in complete))
        self.assertFalse((root_b / "reports").exists())
        calls = [json.loads(line) for line in (gh / "calls.jsonl").read_text().splitlines()]
        self.assertEqual(len(calls), 1, "Held search must not be reported as a successful publication")
        self.assertIsNone(calls[0]["body"])
        (server.artifacts / "deadline-observation.json").write_text(json.dumps({
            "configured_seconds": 12, "elapsed": elapsed, "intervals": intervals,
            "repeat_did_not_reset": not ambiguous, "measurement_ambiguous": ambiguous,
            "product_forced_cleanup": True, "harness_forced_cleanup": False}, indent=2) + "\n")
        if ambiguous:
            self.skipTest("Unvollständig: observer delay overlapped original and restarted deadline hypotheses")
