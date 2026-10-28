"""REPL quit/EOF/Ctrl+C leave accepted work alive; shutdown and technical end differ."""
import ast
import fcntl
import json
import os
from pathlib import Path
import pty
import struct
import subprocess
import sys
import tempfile
import termios
import time
import unittest

from tests._support import CLI, ROOT, load_config, start_server, write_config
from tests.faults import cleanup_peer, protocol_fixture
from tests.repl import ReplIO, tty_settings


class ReplDisconnectTest(unittest.TestCase):
    def test_client_exit_keeps_work_but_shutdown_ends_session_and_technical_end_errors(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup game window. quit, pipe EOF and owned-PTY Ctrl+C "
                         "each leave a 12-tick Warp at 2/s to finish: 36 ticks total, no game input. "
                         "A fourth REPL explicitly shuts down the game. One additional external "
                         "peer then ends unexpectedly (zero game ticks, declared server/REPL exits 1). "
                         "No desktop input or retries. Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "repl-disconnect")
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
        ticks_type = "test_app::woodpecker::SimulationTicks"
        self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": 0})
        for index, mode in enumerate(("quit", "eof", "ctrl-c")):
            directory = server.artifacts / mode
            directory.mkdir()
            stderr = self.enterContext((directory / "stderr.log").open("wb"))
            before_tty = None
            if mode == "ctrl-c":
                master, slave = pty.openpty()
                self.addCleanup(os.close, master)
                self.addCleanup(os.close, slave)
                fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
                before_tty = termios.tcgetattr(slave)
                process = subprocess.Popen([sys.executable, str(ROOT / "tests/repl/pty_client.py"), str(directory), str(CLI),
                                           "--address", server.address, "session", "repl", session_id],
                                          cwd=ROOT, stdin=slave, stdout=slave, stderr=stderr,
                                          start_new_session=True, env={**os.environ, "TERM": "xterm-256color"})
                client = ReplIO(process, directory, master=master)
            else:
                stdout = self.enterContext((directory / "stdout.log").open("wb"))
                process = subprocess.Popen([str(CLI), "--address", server.address, "session", "repl", session_id],
                                           cwd=ROOT, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr,
                                           start_new_session=True)
                client = ReplIO(process, directory)
            self.addCleanup(client.cleanup)
            client.wait(lambda text: f"connected {session_id} SessionReady" in text)
            cursor = server.cli("session", "poll", session_id)["cursor"]
            mark = client.mark()
            client.send(b"tick warp 12 pace 2\r" if mode == "ctrl-c" else b"tick warp 12 pace 2\n")
            warp_id = client.pending("tick.warp.start", after=mark)
            before_exit = server.read_resource(session_id, ticks_type)["ticks"]
            self.assertLess(before_exit, (index + 1) * 12)
            if mode == "quit":
                client.send(b"quit\n")
            elif mode == "eof":
                process.stdin.close()
            else:
                client.send(b"\x03")  # Real Ctrl+C byte in this client's raw PTY, not the desktop.
            self.assertEqual(client.wait_exit(), 0, mode)
            client.raw()
            if before_tty is not None:
                self.assertEqual(tty_settings(ast.literal_eval((directory / "tty-after.txt").read_text())), tty_settings(before_tty))
            self.assertEqual((directory / "stderr.log").read_bytes(), b"", mode)
            at_exit = server.read_resource(session_id, ticks_type)["ticks"]
            self.assertLess(at_exit, (index + 1) * 12, "Client exit must not wait for Warp completion")
            self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
            outcome = server.result(session_id, {"kind": "pending", "request_id": warp_id,
                                    "command": "tick.warp.start"}, cursor=cursor, timeout=15)
            self.assertEqual(outcome["kind"], "completed", outcome)
            self.assertEqual(outcome["output"], {"requested_ticks": 12, "executed_ticks": 12, "outcome": "completed"})
            self.assertEqual(server.read_resource(session_id, ticks_type), {"ticks": (index + 1) * 12})
            activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
            self.assertEqual(activity["kind"], "activity")
            events = [entry["event"] for entry in activity["entries"]]
            self.assertFalse(any(event.get("command") in ("tick.warp.stop", "shutdown", "replay.stop") for event in events))
            self.assertEqual([event["kind"] for event in events if event.get("request_id") == warp_id],
                             ["pending", "completed"])
            server.record(["observation", "repl-disconnect"], mode=mode, client_pid=process.pid,
                          ticks_before_exit=before_exit, ticks_at_exit=at_exit, final_ticks=(index + 1) * 12)
        directory = server.artifacts / "explicit-shutdown"
        directory.mkdir()
        stdout = self.enterContext((directory / "stdout.log").open("wb"))
        stderr = self.enterContext((directory / "stderr.log").open("wb"))
        process = subprocess.Popen([str(CLI), "--address", server.address, "session", "repl", session_id],
                                   cwd=ROOT, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, start_new_session=True)
        client = ReplIO(process, directory)
        self.addCleanup(client.cleanup)
        client.wait(lambda text: f"connected {session_id} SessionReady" in text)
        mark = client.mark()
        client.send(b"shutdown\n")
        shutdown_id = client.pending("shutdown", after=mark)
        self.assertEqual(shutdown_id, 18446744073709551615)
        self.assertEqual(client.terminal(shutdown_id, after=mark)["kind"], "completed")
        self.assertEqual(client.wait_exit(), 0)
        self.assertEqual((directory / "stderr.log").read_bytes(), b"")
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "Ended")
        self.assertIsNone(server.process.poll(), "REPL shutdown must not stop the server")
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)

        # External technical-end phase: no game, window or simulated tick.
        runs = ROOT / "target/test-runs"
        fixture = Path(tempfile.mkdtemp(prefix="repl-end-fixture-", dir=runs))
        self.artifact_dirs = [*self.artifact_dirs, str(fixture)]
        manifest, environment = protocol_fixture(fixture / "peer-package")
        holder = {}
        self.addCleanup(cleanup_peer, holder)
        broken = start_server(self, "repl-technical-end", env=environment, expected_exit_code=1)
        configuration = load_config()
        configuration["launch"].update(manifest_path=str(manifest), package="woodpecker-protocol-fixture",
                                       target={"kind": "binary", "name": "protocol-peer"}, features=[], arguments=["recording-end"])
        config = broken.artifacts / "peer.toml"
        write_config(config, configuration)
        self.assertEqual(load_config(config)["launch"], configuration["launch"])
        self.assertEqual(configuration["launch"]["package"], "woodpecker-protocol-fixture")
        created = broken.cli("session", "create", "--config", str(config))
        peer_id = created["id"]
        root = Path(created["artifact_dir"])
        holder["path"] = root / "protocol-peer.json"
        deadline = time.monotonic() + 30
        while True:
            detail = broken.cli("session", "inspect", peer_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail("External end peer did not become ready")
            time.sleep(0.02)
        directory = broken.artifacts / "client"
        directory.mkdir()
        stdout = self.enterContext((directory / "stdout.log").open("wb"))
        stderr = self.enterContext((directory / "stderr.log").open("wb"))
        process = subprocess.Popen([str(CLI), "--address", broken.address, "session", "repl", peer_id],
                                   cwd=ROOT, stdin=subprocess.PIPE, stdout=stdout, stderr=stderr, start_new_session=True)
        client = ReplIO(process, directory)
        self.addCleanup(client.cleanup, 1)
        client.wait(lambda text: f"connected {peer_id} SessionReady" in text)
        mark = client.mark()
        client.send(b"tick warp 5\n")
        client.pending("tick.warp.start", after=mark)
        client.send(b"inspect resources\n")  # The explicitly chosen peer ends on this second command.
        self.assertEqual(client.wait_exit(timeout=10), 1)
        error = json.loads((directory / "stderr.log").read_bytes())["error"]
        self.assertIn("session failed:", error)
        self.assertEqual(broken.cli("session", "inspect", peer_id)["state"], "Failed")
        journal = [json.loads(line) for line in (root / "protocol-peer.jsonl").read_text().splitlines()]
        self.assertEqual([row["value"]["command"] for row in journal if row["kind"] == "stdin"],
                         ["tick.warp.start", "inspect.query"])
        broken.cli("server", "stop")
        self.assertEqual(broken.process.wait(timeout=5), 1)
