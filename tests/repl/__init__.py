"""Transport, transcripts and owned REPL-client cleanup; no sessions or game actions."""
import errno
import json
import os
import re
import select
import signal
import subprocess
import termios
import time


def plain_output(data):
    """Separate redraws without emulating a terminal or discarding JSON event fields."""
    text = data.decode("utf-8", errors="replace")
    text = re.sub(r"\x1b\[[0-9]*G", "\r", text)
    return re.sub(r"\x1b\[[0-?]*[ -/]*[@-~]", "", text)


def tty_settings(attributes):
    """Compare all termios settings, excluding only the kernel PENDIN status bit."""
    settings = list(attributes)
    settings[3] &= ~getattr(termios, "PENDIN", 0)
    return settings


def output_events(text):
    events = []
    for line in text.splitlines():
        if line.startswith("{"):
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue  # A read may end in the middle of the final output line.
            if isinstance(event, dict) and "kind" in event:
                events.append(event)
    return events


class ReplIO:
    def __init__(self, process, directory, *, master=None):
        self.process = process
        self.directory = directory
        self.master = master

    def raw(self):
        if self.master is not None:
            while select.select([self.master], [], [], 0)[0]:
                try:
                    chunk = os.read(self.master, 65536)
                except OSError as error:
                    if error.errno in (errno.EIO, errno.ENXIO, errno.ENOTTY) and self.process.poll() is not None:
                        break
                    raise
                if not chunk:
                    break
                with (self.directory / "stdout.log").open("ab") as output:
                    output.write(chunk)
        path = self.directory / "stdout.log"
        return path.read_bytes() if path.exists() else b""

    def mark(self):
        return len(self.raw())

    def text(self, after=0):
        return plain_output(self.raw()[after:])

    def send(self, data):
        """Write exactly supplied bytes; no implicit Enter, action, retry or reset."""
        with (self.directory / "input.jsonl").open("a") as journal:
            journal.write(json.dumps({"hex": data.hex()}) + "\n")
        if self.master is None:
            self.process.stdin.write(data)
            self.process.stdin.flush()
        else:
            view = memoryview(data)
            while view:
                count = os.write(self.master, view)
                view = view[count:]

    def wait(self, predicate, *, after=0, timeout=10):
        deadline = time.monotonic() + timeout
        while True:
            text = self.text(after)
            found = predicate(text)
            if found:
                return found
            if self.process.poll() is not None:
                raise AssertionError(f"REPL exited before expected output; see {self.directory}\n{text[-2000:]}")
            if time.monotonic() >= deadline:
                raise TimeoutError(f"REPL output deadline exceeded; see {self.directory}\n{text[-2000:]}")
            time.sleep(0.01)

    def pending(self, command, *, after=0):
        pattern = re.compile(r"\[(\d+)\] " + re.escape(command) + r" pending")
        return int(self.wait(lambda text: (match.group(1) if (match := pattern.search(text)) else None),
                             after=after))

    def terminal(self, request_id, *, after=0):
        return self.wait(lambda text: next((event for event in output_events(text)
                         if event.get("request_id") == request_id
                         and event["kind"] in ("completed", "rejected", "failed")), None), after=after)

    def wait_exit(self, timeout=5):
        """Drain owned PTY output while waiting; its finite buffer must not block exit."""
        deadline = time.monotonic() + timeout
        while self.process.poll() is None:
            self.raw()
            if time.monotonic() >= deadline:
                raise subprocess.TimeoutExpired(self.process.args, timeout)
            time.sleep(0.01)
        self.raw()
        return self.process.returncode

    def cleanup(self, expected_exit=0):
        forced = False
        if self.process.poll() is None:
            # Client-only graceful interruption; never sends Stop/Shutdown to a session.
            self.process.send_signal(signal.SIGINT)
            try:
                self.wait_exit(timeout=5)
            except subprocess.TimeoutExpired:
                forced = True
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait(timeout=5)
        self.raw()
        if self.process.stdin is not None:
            self.process.stdin.close()
        (self.directory / "client-cleanup.json").write_text(json.dumps({
            "client_pid": self.process.pid, "returncode": self.process.returncode,
            "expected_returncode": expected_exit, "forced": forced}) + "\n")
        if forced or self.process.returncode != expected_exit:
            raise AssertionError(f"REPL cleanup failed; see {self.directory}")
