"""Public CLI diagnosis of an external faulty game-protocol peer; no renderer or game."""
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config


class ProtocolFaultsTest(unittest.TestCase):
    def _peer_cleanup(self, holder):
        path = holder.get("path")
        if path is None or not path.exists():
            return
        info = json.loads(path.read_text())
        pid = info["pid"]
        forced = False
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            pass
        else:
            command = subprocess.run(["ps", "-p", str(pid), "-o", "command="],
                                     capture_output=True, text=True, timeout=5).stdout
            self.assertIn("tests/faults/protocol_peer.py peer", command,
                          "PID reused; do not signal an unverified process")
            os.killpg(pid, signal.SIGKILL)
            forced = True
        (path.parent / "peer-cleanup.json").write_text(json.dumps({
            **info, "forced": forced,
        }, indent=2) + "\n")
        holder.clear()
        self.assertFalse(forced, "Server failed to reap its external fault peer")

    def test_bad_handshakes_and_responses_are_never_misreported_as_success(self):
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="protocol-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        cargo = shutil.which("cargo")
        self.assertIsNotNone(cargo, "Real Cargo is needed for the sentinel package's metadata")
        manifest = fixture / "Cargo.toml"
        manifest.write_text('[package]\nname = "woodpecker-protocol-fixture"\nversion = "0.0.0"\n'
                            'edition = "2024"\n[workspace]\n[[bin]]\nname = "protocol-peer"\n'
                            'path = "main.rs"\n')
        # Even if the external adapter is bypassed, Cargo can only run this
        # windowless sentinel, never test_app or another application.
        (fixture / "main.rs").write_text('fn main() { std::process::exit(86); }\n')
        adapter = fixture / "cargo-peer"
        peer = ROOT / "tests/faults/protocol_peer.py"
        adapter.write_text(f"#!/bin/sh\nexec {shlex.quote(sys.executable)} {shlex.quote(str(peer))} cargo \"$@\"\n")
        adapter.chmod(0o700)
        environment = {"CARGO": str(adapter), "WOODPECKER_TEST_REAL_CARGO": cargo,
                       "WOODPECKER_TEST_PEER_MANIFEST": str(manifest)}
        for mode, startup_error in (
            ("wrong-version", "unsupported_protocol_version"),
            ("invalid-capabilities", "invalid_ready"),
            ("missing-fixed-clock", "invalid_config"),
            ("missing-engine-startup", "invalid_config"),
            ("malformed-known", None), ("wrong-name", None), ("bad-output", None),
            ("malformed-unassigned", None), ("unknown-id", None), ("duplicate", None),
        ):
            holder = {}
            self.addCleanup(self._peer_cleanup, holder)
            server = start_server(self, f"protocol-{mode}", env=environment,
                                  expected_exit_code=1 if startup_error else 0)
            configuration = load_config()
            configuration["launch"].update(
                manifest_path=str(manifest), package="woodpecker-protocol-fixture",
                target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=[mode],
            )
            if mode == "missing-engine-startup":
                configuration["startup"] = {"kind": "engine", "timeout_seconds": 1, "max_ticks": 1}
            config = server.artifacts / "protocol.toml"
            write_config(config, configuration)
            # Check the launch barrier before submitting anything to the server.
            parsed = load_config(config)
            self.assertEqual(parsed["launch"]["manifest_path"], str(manifest))
            self.assertEqual(parsed["launch"]["package"], "woodpecker-protocol-fixture")
            self.assertEqual(parsed["launch"]["target"]["name"], "protocol-peer")
            self.assertEqual(parsed["launch"]["features"], [])
            created = server.cli("session", "create", "--config", str(config))
            session_id = created["id"]
            holder["path"] = Path(created["artifact_dir"]) / "protocol-peer.json"
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                if detail["state"] in ("SessionReady", "Failed"):
                    break
                self.assertEqual(detail["state"], "Starting", (mode, detail))
                if time.monotonic() >= deadline:
                    self.fail(f"{mode}: handshake result missing; see {server.artifacts}")
                time.sleep(0.02)
            self.assertTrue(holder["path"].is_file(), "Sentinel adapter did not launch the fault peer")
            if startup_error:
                self.assertEqual(detail["state"], "Failed", (mode, detail))
                self.assertEqual(detail["error"]["code"], startup_error, (mode, detail))
                activity = server.cli("session", "poll", session_id)
                self.assertEqual(activity["kind"], "activity")
                self.assertFalse(any(entry["event"].get("state") in ("SessionReady", "EngineReady")
                                     or entry["event"]["kind"] in ("pending", "completed")
                                     for entry in activity["entries"]), (mode, activity))
            else:
                self.assertEqual(detail["state"], "SessionReady", (mode, detail))
                ready_cursor = server.cli("session", "poll", session_id)["cursor"]
                warp = server.cli("session", "submit", session_id, "--command", json.dumps({
                    "command": "tick.warp.start", "arguments": {"ticks": 1},
                }))
                correlated = mode in ("malformed-known", "wrong-name", "bad-output")
                if correlated:
                    outcome = server.result(session_id, warp, cursor=ready_cursor)
                    self.assertEqual(outcome["kind"], "failed", (mode, outcome))
                    self.assertEqual(outcome["error"]["code"], "invalid_response", outcome)
                else:
                    deadline = time.monotonic() + 5
                    while True:
                        activity = server.cli("session", "poll", session_id, "--wait-ms", "100")
                        self.assertEqual(activity["kind"], "activity", activity)
                        events = [entry["event"] for entry in activity["entries"]]
                        protocol_events = [event["event"] for event in events
                                           if event["kind"] == "event"
                                           and event["event"]["kind"] == "protocol_error"]
                        if protocol_events:
                            break
                        if time.monotonic() >= deadline:
                            self.fail(f"{mode}: missing protocol event")
                        time.sleep(0.02)
                    self.assertEqual([event["code"] for event in protocol_events], ["invalid_response"])
                    terminal = [event for event in events if event.get("request_id") == warp["request_id"]
                                and event["kind"] in ("completed", "rejected", "failed")]
                    self.assertEqual(len(terminal), 1 if mode == "duplicate" else 0, (mode, terminal))
                stop = server.cli("session", "submit", session_id, "--command", json.dumps({
                    "command": "tick.warp.stop", "arguments": {},
                }))
                stopped = server.result(session_id, stop)
                self.assertEqual(stopped["kind"], "completed", stopped)
                was_running = mode in ("malformed-unassigned", "unknown-id")
                self.assertEqual(stopped["output"], {"was_running": was_running})
                outcome = server.result(session_id, warp)
                if not correlated:
                    self.assertEqual(outcome["kind"], "completed", outcome)
                    self.assertEqual(outcome["output"], {
                        "requested_ticks": 1, "executed_ticks": 0 if was_running else 1,
                        "outcome": "stopped" if was_running else "completed",
                    })
                activity = server.cli("session", "poll", session_id)
                terminal = [entry["event"] for entry in activity["entries"]
                            if entry["event"]["kind"] in ("completed", "rejected", "failed")]
                self.assertCountEqual([event["request_id"] for event in terminal],
                                      [warp["request_id"], stop["request_id"]])
                self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
            server.record(["observation", "protocol-fault"], mode=mode, expected_startup_error=startup_error)
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 1 if startup_error else 0)
            self._peer_cleanup(holder)
