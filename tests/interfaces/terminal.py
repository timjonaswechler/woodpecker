"""Real terminal editing and terminal-mode restoration on a Unix PTY."""
import fcntl
import os
import select
import signal
import struct
import subprocess
import sys
import termios
import time

from tests.session.application import ROOT


class Terminal:
    def __init__(self, session):
        self.text = ""
        self.master, self.slave = os.openpty()
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
        self.log = (session.server.directory / "terminal.log").open("w")

        def controlling_terminal():
            os.setsid()
            fcntl.ioctl(0, termios.TIOCSCTTY, 0)

        # Keep the session leader alive on macOS until restoration is inspected.
        supervisor = """
import signal, subprocess, sys, termios
original = termios.tcgetattr(0)
child = subprocess.Popen(sys.argv[1:])
signal.signal(signal.SIGINT, lambda *_: child.send_signal(signal.SIGINT))
code = child.wait()
assert termios.tcgetattr(0) == original, "terminal mode not restored"
sys.exit(code)
"""
        self.process = subprocess.Popen(
            [sys.executable, "-c", supervisor, session.server.binary,
             "--address", session.server.address, "session", "repl", session.id],
            cwd=ROOT, stdin=self.slave, stdout=self.slave, stderr=self.slave,
            preexec_fn=controlling_terminal,
        )

    def send(self, value):
        os.write(self.master, value.encode() if isinstance(value, str) else value)

    def drain(self):
        if select.select([self.master], [], [], 0.05)[0]:
            try:
                data = os.read(self.master, 65536)
            except OSError:
                if self.process.poll() is None:
                    raise
                return
            text = data.decode(errors="replace")
            self.text += text
            self.log.write(text)
            self.log.flush()

    def wait(self, fragment):
        deadline = time.monotonic() + 10
        while fragment not in self.text:
            assert time.monotonic() < deadline, (fragment, self.text)
            self.drain()

    def finish(self):
        deadline = time.monotonic() + 5
        while self.process.poll() is None:
            assert time.monotonic() < deadline, ("Terminal beendet sich nicht", self.text)
            self.drain()
        assert self.process.returncode == 0, self.text

    def close(self):
        try:
            if self.process.poll() is None:
                self.process.send_signal(signal.SIGINT)
                try:
                    self.finish()
                finally:
                    if self.process.poll() is None:
                        os.killpg(self.process.pid, signal.SIGKILL)
                        self.process.wait(timeout=5)
        finally:
            os.close(self.master)
            os.close(self.slave)
            self.log.close()
