"""Cancellation of an owned pre-Ready process is a normal managed stop, not a failure report."""
import json
import os
from pathlib import Path
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture
from tests.shutdown import no_failures, wait_state


class StartingStopTest(unittest.TestCase):
    def test_stopping_a_proven_live_start_reaps_it_without_late_ready_or_report(self):
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="stop-starting-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        manifest, environment = protocol_fixture(fixture / "peer-package")
        holder = {}
        self.addCleanup(cleanup_peer, holder)
        server = start_server(self, "stop-starting", env=environment)
        configuration = load_config()
        configuration["launch"].update(manifest_path=str(manifest), package="woodpecker-protocol-fixture",
            target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=["shutdown-starting"])
        config = server.artifacts / "config.toml"
        write_config(config, configuration)
        self.assertEqual(load_config(config)["launch"], configuration["launch"])
        created = server.cli("session", "create", "--config", str(config))
        session_id = created["id"]
        root = Path(created["artifact_dir"])
        holder["path"] = root / "protocol-peer.json"
        deadline = time.monotonic() + 15
        while not (root / "before-ready").exists():
            self.assertEqual(server.cli("session", "inspect", session_id)["state"], "Starting")
            if time.monotonic() >= deadline:
                self.fail("External process never reached the pre-Ready gate")
            time.sleep(0.02)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "Starting")
        rejected = server.run_cli("session", "submit", session_id, "--command", '{"command":"tick.warp.stop","arguments":{}}')
        self.assertEqual(rejected.returncode, 1)
        self.assertEqual(json.loads(rejected.stdout)["code"], "session_not_ready")
        self.assertNotIn("request_id", json.loads(rejected.stdout))
        server.cli("session", "stop", session_id)
        detail = wait_state(server, session_id, "Ended")
        self.assertIsNone(detail["error"])
        self.assertEqual(server.cli("session", "stop", session_id)["state"], "Ended")
        pid = json.loads((root / "protocol-peer.json").read_text())["pid"]
        with self.assertRaises(ProcessLookupError):
            os.kill(pid, 0)
        (root / "release-ready").write_text("release after cancellation must not resurrect process\n")
        activity = server.cli("session", "poll", session_id)
        events = [row["event"] for row in activity["entries"]]
        self.assertTrue(no_failures(events), events)
        self.assertEqual([e["state"] for e in events if e["kind"] == "lifecycle"], ["Stopping", "Ended"])
        self.assertFalse(any(e["kind"] in ("pending", "completed", "failed") for e in events))
        self.assertFalse((root / "reports").exists())
        self.assertEqual(server.cli("session", "ls")[0]["state"], "Ended")
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
