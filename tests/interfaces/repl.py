"""A persistent real REPL, with command results read from its own output."""
import codecs
import json
import os
import re
import select
import signal
import subprocess
import time

from tests.session.application import ROOT
from tests.interfaces.json import check_result

PENDING = re.compile(r"^\[(\d+)\] ([\w.]+) pending$")


def line_for(name, arguments):
    if name == "tick.warp.start":
        line = f"tick warp {arguments['ticks']}"
        if arguments.get("pace") is not None:
            pace = arguments["pace"]
            line += f" pace {pace.get('target', 'max')}"
        return line
    if name == "tick.warp.set_pace":
        return f"tick warp pace {arguments['pace'].get('target', 'max')}"
    if name == "tick.warp.stop":
        return "tick warp stop"
    if name in ("recording.start", "replay.start"):
        return f"{name.split('.')[0]} start {arguments['path']}"
    if name in ("recording.stop", "replay.stop"):
        return name.replace(".", " ")
    if name == "inspect.query":
        return "inspect query " + json.dumps(arguments, ensure_ascii=False)
    if name == "shutdown":
        return "shutdown"
    return "command " + json.dumps({"command": name, "arguments": arguments}, ensure_ascii=False)


class Repl:
    def __init__(self, session):
        self.session = session
        self.lines, self.events = [], []
        self.buffer = ""
        self.decoder = codecs.getincrementaldecoder("utf-8")()
        self.closed = False
        self.finished = False
        self.log = (session.server.directory / f"repl-{session.id}.log").open("a")
        self.process = subprocess.Popen(
            [session.server.binary, "--address", session.server.address,
             "session", "repl", session.id], cwd=ROOT,
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
        )
        try:
            self.wait(lambda: any(line.startswith(f"connected {session.id} ") for line in self.lines),
                      "REPL verbindet sich nicht")
            # Connected also prints commands already pending in the session.
            # A local help response marks the end of that initial display without
            # submitting app work or consuming a request ID.
            self.send("help")
            self.wait(lambda: any(line.startswith("quit, Ctrl+C and EOF disconnect") for line in self.lines),
                      "REPL hat die initiale Pending-Anzeige nicht abgeschlossen")
        except BaseException:
            self.close()
            raise

    def drain(self):
        reader = self.process.stdout.fileno()
        if not select.select([reader], [], [], 0.05)[0]:
            return
        data = os.read(reader, 65536)
        if not data:
            self.closed = True
            return
        decoded = self.decoder.decode(data)
        self.log.write(decoded)
        self.log.flush()
        self.buffer += decoded
        while "\n" in self.buffer:
            line, self.buffer = self.buffer.split("\n", 1)
            self.lines.append(line)
            if line.startswith("{"):
                self.events.append(json.loads(line))

    def wait(self, callback, description, timeout=30):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            result = callback()
            if result:
                return result
            self.drain()
            if self.closed and not callback():
                raise AssertionError((description, "REPL beendet", self.lines))
        raise AssertionError((description, self.lines))

    def send(self, line):
        self.log.write(f"> {line}\n")
        self.log.flush()
        self.process.stdin.write((line + "\n").encode())
        self.process.stdin.flush()

    def submit(self, name, arguments):
        # Activity includes other clients' commands as well. Establish which
        # requests existed before this submission, so buffered broadcasts cannot
        # be mistaken for this REPL's newly accepted command.
        known = {event.get("request_id") for event in self.session.poll()}
        start = len(self.lines)
        self.send(line_for(name, arguments))

        def pending():
            for line in self.lines[start:]:
                match = PENDING.fullmatch(line)
                if match:
                    request = int(match[1])
                    if request not in known and match[2] == name:
                        return request
                if line.startswith(("input error:", "not sent:")):
                    raise AssertionError(line)
            return None
        return self.wait(pending, f"REPL akzeptiert {name} nicht")

    def outcome(self, request, name):
        def completed():
            matches = [e for e in self.events if e.get("request_id") == request
                       and e.get("kind") in ("completed", "rejected", "failed")]
            if len(matches) > 1 or (matches and matches[0]["command"] != name):
                raise AssertionError(matches)
            return matches[0] if matches else None
        return self.wait(completed, f"REPL zeigt kein Ergebnis für {name}, Request {request}")

    def command(self, name, arguments, expected_error=None):
        return check_result(self.outcome(self.submit(name, arguments), name), expected_error)

    def refuse(self, name, arguments, code):
        start = len(self.events)
        line_start = len(self.lines)
        self.send(line_for(name, arguments))
        def refused():
            reply = next((e for e in self.events[start:] if e.get("kind") == "error"), None)
            if reply:
                return reply
            line = next((line for line in self.lines[line_start:] if line.startswith(f"{code}:")), None)
            return {"kind": "client_error", "code": code, "message": line} if line else None
        reply = self.wait(refused, f"REPL zeigt keine Annahmeverweigerung für {name}")
        assert reply["code"] == code, reply
        return reply

    def close(self):
        if self.finished:
            return
        self.finished = True
        try:
            if self.process.poll() is None:
                self.process.send_signal(signal.SIGINT)
                try:
                    self.process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.process.kill()
                    self.process.wait(timeout=5)
                    raise AssertionError("REPL reagiert nicht auf Disconnect")
        finally:
            self.process.stdin.close()
            self.process.stdout.close()
            self.log.close()
