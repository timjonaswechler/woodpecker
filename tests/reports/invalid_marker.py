"""External corrupt markers stay diagnostic bytes and cannot manufacture a panic."""
from pathlib import Path
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture
from tests.reports import audit_local_files, failures, observe, reports


class InvalidMarkerTest(unittest.TestCase):
    def test_fragmented_valid_marker_and_invalid_frames_preserve_nonmarker_bytes_through_eof(self):
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="report-marker-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        manifest, environment = protocol_fixture(fixture / "peer-package")
        for variant in ("valid", "json", "checksum", "base64", "duplicate", "missing", "truncated", "process-payload"):
            holder = {}
            self.addCleanup(cleanup_peer, holder)
            server = start_server(self, "marker-" + variant, env=environment,
                                  expected_exit_code=0 if variant == "valid" else 1)
            configuration = load_config()
            configuration["launch"].update(manifest_path=str(manifest), package="woodpecker-protocol-fixture",
                target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=["report-marker-" + variant])
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
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "SessionReady":
                    break
                if time.monotonic() >= deadline:
                    self.fail("Marker peer readiness deadline")
                time.sleep(0.02)
            pending = server.cli("session", "submit", session_id, "--command",
                                 '{"command":"tick.warp.start","arguments":{"ticks":1}}')
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "completed" if variant == "valid" else "failed", outcome)
            if variant == "valid":
                server.cli("session", "stop", session_id)
            events = observe(server, session_id, lambda e: len(reports(e)) >= 1 and any(
                v["kind"] == "lifecycle" and v["state"] in ("Ended", "Failed") for v in e))
            errors = [v["event"] for v in events if v["kind"] == "event" and v["event"]["kind"] == "observation_error"]
            self.assertEqual(len(failures(events)), 1, events)
            self.assertEqual(len(reports(events)), 1, events)
            self.assertEqual(failures(events)[0]["origin"]["kind"], "panic" if variant == "valid" else "process_exit")
            if variant == "valid":
                self.assertEqual(errors, [])
                self.assertEqual(failures(events)[0]["message"], "external report panic")
            else:
                self.assertTrue(errors, events)
                self.assertTrue(all(error["code"] == "invalid_report_marker" for error in errors))
            self.assertEqual(reports(events)[0]["result"]["status"], "submitted")
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0 if variant == "valid" else 1)
            audit_local_files(server, root, events)
            actual = (server.artifacts / "server.stderr.log").read_bytes()
            expected = (root / "expected-human-stderr.bin").read_bytes()
            self.assertIn(expected, actual, variant)
            if variant == "valid":
                self.assertNotIn(b"\x1eWOODPECKER_REPORT:", actual)
