"""A real controlling PTY stays usable during work and restores only its own tty."""
import ast
import fcntl
import json
import os
import pty
import re
import struct
import subprocess
import sys
import termios
import time
import unicodedata
import unittest

from tests._support import CLI, ROOT, start_server
from tests.repl import ReplIO, tty_settings


class ReplTerminalTest(unittest.TestCase):
    def test_pending_work_unicode_editing_and_normal_and_error_exits_restore_the_pty(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup game window; one 120-tick Warp at 2/s stopped early. "
                         "Two owned PTY clients (normal and missing-session error), terminal-only "
                         "Unicode/editing/help/pending bytes and one external read-only query. "
                         "No desktop input/resizing or virtual game input. Permission required: "
                         "WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "repl-terminal")
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
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 0})
        directory = server.artifacts / "normal-pty"
        directory.mkdir()
        master, slave = pty.openpty()
        self.addCleanup(os.close, master)
        self.addCleanup(os.close, slave)
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 12, 44, 0, 0))
        before = termios.tcgetattr(slave)
        (directory / "tty-before.txt").write_text(repr(before) + "\n")
        stderr = self.enterContext((directory / "stderr.log").open("wb"))
        process = subprocess.Popen([sys.executable, str(ROOT / "tests/repl/pty_client.py"), str(directory), str(CLI),
                                   "--address", server.address, "session", "repl", session_id],
                                  cwd=ROOT, stdin=slave, stdout=slave, stderr=stderr, start_new_session=True,
                                  env={**os.environ, "TERM": "xterm-256color"})
        client = ReplIO(process, directory, master=master)
        self.addCleanup(client.cleanup)
        client.wait(lambda text: f"connected {session_id} SessionReady" in text)
        raw_attributes = termios.tcgetattr(slave)
        self.assertFalse(raw_attributes[3] & (termios.ECHO | termios.ICANON))
        mark = client.mark()
        client.send(b"tick warp 120 pace 2\r")
        warp_id = client.pending("tick.warp.start", after=mark)
        mark = client.mark()
        client.send(b"pending\r")
        client.wait(lambda text: "server pending: 1" in text, after=mark)
        mark = client.mark()
        client.send(b"help\r")
        client.wait(lambda text: "quit, Ctrl+C and EOF disconnect only this client" in text, after=mark)
        # Long/wide text forces horizontal clipping in this 44-column owned PTY.
        target = "absent::Type<Grüße, 東京>"
        client.send(("inspect resource " + target + "Z").encode())
        client.wait(lambda text: "東京" in text)
        # Publish an unrelated result while the REPL line is still being edited.
        query = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "inspect.query", "arguments": {"source": "resources", "selector": {
                "kind": "type", "type_path": "test_app::app::settings::Settings"}, "projection": {"kind": "value"}}}))
        external = server.result(session_id, query)
        self.assertEqual(external["kind"], "completed", external)
        self.assertEqual(client.terminal(query["request_id"])["output"], external["output"])
        # Left/Delete removes Z; Home/insert/Backspace/End preserves the full Unicode line.
        client.send(b"\x1b[D\x1b[3~\x1b[HQ\x7f\x1b[F")
        mark = client.mark()
        client.send(b"\r")
        edited_id = client.pending("inspect.query", after=mark)
        self.assertNotEqual(edited_id, query["request_id"])
        edited = client.terminal(edited_id, after=mark)
        # Unknown selectors are rejected. The exact echoed path also catches
        # leftover Z/Q characters or damaged Unicode from terminal editing.
        self.assertEqual(edited["kind"], "rejected", edited)
        self.assertEqual(edited["error"]["code"], "unknown_type_path", edited)
        self.assertEqual(edited["error"]["message"], target, edited)
        mark = client.mark()
        client.send(b"tick warp stop\r")
        stop_id = client.pending("tick.warp.stop", after=mark)
        self.assertEqual(client.terminal(stop_id, after=mark)["output"], {"was_running": True})
        warp = client.terminal(warp_id)
        self.assertEqual(warp["output"]["requested_ticks"], 120)
        self.assertEqual(warp["output"]["outcome"], "stopped")
        self.assertGreaterEqual(warp["output"]["executed_ticks"], 0)
        self.assertLess(warp["output"]["executed_ticks"], 120)
        client.send(b"quit\r")
        self.assertEqual(client.wait_exit(), 0)
        transcript = client.raw().decode("utf-8")
        redraws = re.findall(r"> ([^\r\n\x1b]*)\x1b\[(\d+)G", transcript)
        self.assertTrue(redraws)
        for visible, column in redraws:
            cells = sum(0 if unicodedata.combining(char) else
                        2 if unicodedata.east_asian_width(char) in ("W", "F") else 1 for char in visible)
            self.assertLessEqual(cells + 2, 44, visible)
            self.assertTrue(1 <= int(column) <= 44, column)
        self.assertIn("Grüße", transcript)
        self.assertIn("東京", transcript)
        self.assertEqual(tty_settings(ast.literal_eval((directory / "tty-after.txt").read_text())), tty_settings(before))
        self.assertEqual((directory / "stderr.log").read_bytes(), b"")
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"),
                         {"ticks": warp["output"]["executed_ticks"]})

        error_directory = server.artifacts / "error-pty"
        error_directory.mkdir()
        error_master, error_slave = pty.openpty()
        self.addCleanup(os.close, error_master)
        self.addCleanup(os.close, error_slave)
        fcntl.ioctl(error_slave, termios.TIOCSWINSZ, struct.pack("HHHH", 12, 44, 0, 0))
        error_before = termios.tcgetattr(error_slave)
        error_stderr = self.enterContext((error_directory / "stderr.log").open("wb"))
        missing_id = "0" * 32 if session_id != "0" * 32 else "1" * 32
        failing = subprocess.Popen([sys.executable, str(ROOT / "tests/repl/pty_client.py"), str(error_directory), str(CLI),
                                   "--address", server.address, "session", "repl", missing_id],
                                  cwd=ROOT, stdin=error_slave, stdout=error_slave, stderr=error_stderr,
                                  start_new_session=True, env={**os.environ, "TERM": "xterm-256color"})
        error_client = ReplIO(failing, error_directory, master=error_master)
        self.addCleanup(error_client.cleanup, 1)
        self.assertEqual(error_client.wait_exit(), 1)
        error_client.raw()
        self.assertIn("binding_rejected", json.loads((error_directory / "stderr.log").read_bytes())["error"])
        self.assertEqual(tty_settings(ast.literal_eval((error_directory / "tty-after.txt").read_text())), tty_settings(error_before))
        (error_directory / "tty-before.txt").write_text(repr(error_before) + "\n")
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
