"""Before successful Ready, failure markers and exit remain launch diagnostics, not reports."""
import json
from pathlib import Path
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture


class BeforeReadyTest(unittest.TestCase):
    def test_markers_bad_handshake_and_exit_before_ready_have_no_report_or_command_context(self):
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="report-before-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        manifest, environment = protocol_fixture(fixture / "peer-package")
        for variant in ("panic", "tracing", "invalid", "exit-0", "exit-17"):
            holder = {}
            self.addCleanup(cleanup_peer, holder)
            server = start_server(self, "before-ready-" + variant, env=environment, expected_exit_code=1)
            configuration = load_config()
            configuration["report"]["tracing_errors"] = variant == "tracing"
            configuration["launch"].update(manifest_path=str(manifest), package="woodpecker-protocol-fixture",
                target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=["report-before-" + variant])
            config = server.artifacts / "config.toml"
            write_config(config, configuration)
            self.assertEqual(load_config(config)["launch"], configuration["launch"])
            created = server.cli("session", "create", "--config", str(config))
            session_id = created["id"]
            root = Path(created["artifact_dir"])
            holder["path"] = root / "protocol-peer.json"
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                if detail["state"] == "Failed":
                    break
                self.assertEqual(detail["state"], "Starting", detail)
                if time.monotonic() >= deadline:
                    self.fail("Before-Ready failure deadline")
                time.sleep(0.02)
            self.assertTrue(holder["path"].is_file(), "The selected peer must actually have started")
            peer = json.loads(holder["path"].read_text())
            self.assertEqual(peer["mode"], "report-before-" + variant)
            self.assertGreater(peer["pid"], 0)
            self.assertIsNotNone(detail["error"])
            self.assertEqual(detail["error"]["code"], "invalid_ready" if variant == "invalid" else "launch")
            message = detail["error"]["message"]
            if variant == "invalid":
                self.assertIn("not a Ready response", message)
            else:
                self.assertIn("game ended before Ready", message)
                self.assertIn(f"exit status: {17 if variant == 'exit-17' else 0}", message)
            if variant in ("panic", "tracing"):
                self.assertIn("external " + ("report panic" if variant == "panic" else "early tracing"),
                              detail["error"]["message"])
            activity = server.cli("session", "poll", session_id)
            self.assertEqual(activity["kind"], "activity")
            events = [entry["event"] for entry in activity["entries"]]
            self.assertFalse(any(v["kind"] in ("report", "pending", "completed", "rejected") for v in events), events)
            self.assertFalse(any(v["kind"] == "event" and v["event"]["kind"] == "failure" for v in events), events)
            self.assertFalse(any(v["kind"] == "lifecycle" and v["state"] in ("SessionReady", "EngineReady") for v in events))
            self.assertFalse((root / "reports").exists())
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 1)
