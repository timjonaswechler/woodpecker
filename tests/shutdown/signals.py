"""Graceful first signals, forced second Ctrl+C, and ungraceful SIGKILL are distinct."""
import json
import os
from pathlib import Path
import signal
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture
from tests.recording import read_jsonl
from tests.shutdown import progress, wait_state


class ShutdownSignalsTest(unittest.TestCase):
    def test_first_interrupt_or_term_drains_second_interrupt_forces_and_kill_is_not_graceful(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Two serial manual game windows, one 120-tick Warp at 2/s each stopped "
                         "by first server SIGINT/SIGTERM (maximum 240 ticks). An external peer then "
                         "holds Shutdown for a second SIGINT; last, SIGKILL targets an empty owned "
                         "server with no children. No OS input; declared error/signal exits for faults. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        for signum in (signal.SIGINT, signal.SIGTERM):
            server = start_server(self, "shutdown-signal-" + str(int(signum)), shutdown_seconds=20)
            configuration = load_config()
            configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
            path = server.artifacts / "config.toml"
            write_config(path, configuration)
            created = server.cli("session", "create", "--config", str(path))
            session_id = created["id"]
            detail = wait_state(server, session_id, "SessionReady", timeout=30)
            root = Path(detail["artifact_dir"])
            server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/signal.jsonl"}})
            server.cli("session", "submit", session_id, "--command", json.dumps({"command": "tick.warp.start", "arguments": {
                "ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}}}))
            self.assertLess(progress(root), 120)
            server.process.send_signal(signum)
            self.assertEqual(server.process.wait(timeout=15), 0)
            rows = read_jsonl(root / "recordings/signal.jsonl")
            self.assertEqual([row["command"] for row in rows[1:-1]], ["tick.warp.start", "tick.warp.stop"])
            self.assertEqual(rows[1]["outcome"]["output"]["outcome"], "stopped")
            self.assertLess(rows[1]["outcome"]["output"]["executed_ticks"], 120)
            self.assertEqual(rows[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 2})
            self.assertFalse((root / "reports").exists())
            with self.assertRaises(ProcessLookupError):
                os.kill(json.loads((root / "process-probe.json").read_text())["pid"], 0)
            server.record(["observation", "first-signal"], signal=int(signum), returncode=0, graceful=True)

        # External second-interrupt phase: deliberately no game/window.
        fixture = Path(tempfile.mkdtemp(prefix="second-interrupt-fixture-", dir=ROOT / "target/test-runs"))
        self.artifact_dirs = [*self.artifact_dirs, str(fixture)]
        manifest, environment = protocol_fixture(fixture / "peer-package")
        holder = {}
        self.addCleanup(cleanup_peer, holder)
        held = start_server(self, "second-interrupt", env=environment, expected_exit_code=1, shutdown_seconds=20)
        configuration = load_config()
        configuration["launch"].update(manifest_path=str(manifest), package="woodpecker-protocol-fixture",
            target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=["shutdown-hold"])
        config = held.artifacts / "config.toml"
        write_config(config, configuration)
        self.assertEqual(load_config(config)["launch"], configuration["launch"])
        created = held.cli("session", "create", "--config", str(config))
        session_id = created["id"]
        root = Path(created["artifact_dir"])
        holder["path"] = root / "protocol-peer.json"
        wait_state(held, session_id, "SessionReady", timeout=30)
        held.cli("session", "submit", session_id, "--command", '{"command":"tick.warp.start","arguments":{"ticks":120}}')
        first_before = time.monotonic()
        held.process.send_signal(signal.SIGINT)
        first_sent_after = time.monotonic()
        deadline = time.monotonic() + 10
        while not (root / "shutdown-entered").exists():
            self.assertIsNone(held.process.poll())
            if time.monotonic() >= deadline:
                self.fail("First interrupt never reached graceful Shutdown")
            time.sleep(0.02)
        # The positive Shutdown gate proves the server handled the first signal.
        # send_signal returning alone is not an upper bound on signal handling.
        first_after = time.monotonic()
        # Separate forced exit from simply reaching the original 20s deadline.
        # Observer pauses widen the interval; they are not a product SLA of 3s.
        second_before = time.monotonic()
        second_sent = held.process.poll() is None
        if second_sent:
            held.process.send_signal(signal.SIGINT)
        second_after = time.monotonic()
        last_alive = second_before if second_sent else first_before
        observation_deadline = first_after + 25
        while True:
            before = time.monotonic()
            status = held.process.poll()
            after = time.monotonic()
            if status is not None:
                exit_interval = (last_alive, after)
                break
            last_alive = before
            if after >= observation_deadline:
                self.fail("Server did not exit after second interrupt or original deadline")
            time.sleep(0.01)
        self.assertEqual(held.process.wait(timeout=1), 1)
        original_earliest, original_latest = first_before + 20, first_after + 20
        ambiguous_signal = not second_sent or second_after >= original_earliest or exit_interval[1] >= original_earliest
        if not second_sent:
            self.assertGreaterEqual(second_after, original_earliest, "Server exited before deadline without second signal")
        if second_sent and second_after < original_earliest:
            self.assertLess(exit_interval[0], original_latest,
                            "Observed server alive past the original deadline; second interrupt did not finish earlier")
        elapsed = exit_interval[1] - second_before
        held.record(["observation", "signal-hypotheses"], first=[first_before, first_after],
                    first_send=[first_before, first_sent_after],
                    second=[second_before, second_after] if second_sent else None, exit=exit_interval,
                    original_deadline=[original_earliest, original_latest], ambiguous=ambiguous_signal)
        pid = json.loads((root / "protocol-peer.json").read_text())["pid"]
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        journal = [json.loads(line) for line in (root / "protocol-peer.jsonl").read_text().splitlines()]
        completed = [json.loads(row["value"]) for row in journal if row["kind"] == "stdout"]
        self.assertFalse(any(v.get("command") == "shutdown" and v.get("status") == "completed" for v in completed))
        self.assertIn("shutdown_incomplete", (held.artifacts / "server.stderr.log").read_text())
        held.record(["observation", "second-interrupt"], elapsed=elapsed, product_forced_cleanup=True)

        # SIGKILL is tested only on an empty server: it guarantees no child cleanup.
        killed = start_server(self, "server-sigkill", expected_exit_code=-int(signal.SIGKILL))
        self.assertEqual(killed.cli("session", "ls"), [])
        killed.process.send_signal(signal.SIGKILL)
        self.assertEqual(killed.process.wait(timeout=5), -int(signal.SIGKILL))
        killed.record(["observation", "sigkill"], graceful=False, sessions=0, returncode=killed.process.returncode)
        if ambiguous_signal:
            self.skipTest("Unvollständig: second-interrupt observation overlaps original deadline; cleanup verified")
