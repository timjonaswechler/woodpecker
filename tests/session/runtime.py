"""Own real processes and artifact lifetimes; never tick or retry commands implicitly."""
from contextlib import contextmanager
import json
import os
from pathlib import Path
import select
import signal
import subprocess
import time
import tomllib

from tests.session.application import ROOT


def until(callback, description, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        result = callback()
        if result:
            return result
        time.sleep(0.025)
    raise AssertionError(description)


def process_gone(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return True
    return False


class Server:
    def __init__(self, binary, directory, shutdown_seconds=5, activity_bytes=4 * 1024 * 1024, env=None):
        self.binary = str(Path(binary).resolve())
        self.directory = Path(directory)
        self.sessions = []
        self.process = None
        self.shutdown_seconds = shutdown_seconds
        self.activity_bytes = activity_bytes
        self.env = env
        self.journal = self.log = None

    def __enter__(self):
        self.directory.mkdir(parents=True, exist_ok=False)
        self.log = (self.directory / "server.log").open("w")
        self.journal = (self.directory / "commands.jsonl").open("w")
        try:
            self.process = subprocess.Popen(
                [self.binary, "--address", "127.0.0.1:0", "server", "start",
                 "--artifact-dir", str(self.directory / "artifacts"),
                 "--shutdown-seconds", str(self.shutdown_seconds),
                 "--activity-bytes", str(self.activity_bytes)],
                cwd=ROOT, stdout=subprocess.PIPE, stderr=self.log, text=True, env=self.env,
            )
            if not select.select([self.process.stdout], [], [], 15)[0]:
                raise AssertionError("Server meldet kein Ready")
            self.address = json.loads(self.process.stdout.readline())["address"]
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def cli(self, *args, success=True, allow_empty=False):
        result = subprocess.run(
            [self.binary, "--address", self.address, *args], cwd=ROOT,
            capture_output=True, text=True, timeout=40,
        )
        self.journal.write(json.dumps({"arguments": args, "exit": result.returncode,
                                       "stdout": result.stdout, "stderr": result.stderr}) + "\n")
        self.journal.flush()
        if (result.returncode == 0) != success:
            raise AssertionError((args, result.returncode, result.stdout, result.stderr))
        if allow_empty and not result.stdout:
            return None
        return json.loads(result.stdout)

    @contextmanager
    def session(self, config, ready=True):
        created = self.cli("session", "create", "--config", str(config))
        session = Session(self, created["id"])
        self.sessions.append(session)
        try:
            if ready:
                startup = tomllib.loads(Path(config).read_text()).get("startup", {}).get("kind", "manual")
                expected = "EngineReady" if startup == "engine" else "SessionReady"
                until(lambda: session.ready(expected), f"Session wurde nicht {expected}", timeout=120)
            yield session
        finally:
            if self.process.poll() is None:
                session.stop()

    def __exit__(self, *_):
        try:
            if self.process is not None and self.process.poll() is None:
                self.process.send_signal(signal.SIGINT)
                try:
                    self.process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    self.process.send_signal(signal.SIGINT)
                    try:
                        self.process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        self.process.kill()
                        self.process.wait(timeout=5)
                        raise AssertionError("Server konnte seine Sessions nicht bereinigen")
        finally:
            if self.process is not None and self.process.stdout is not None:
                self.process.stdout.close()
            if self.log is not None:
                self.log.close()
            if self.journal is not None:
                self.journal.close()


class Session:
    def __init__(self, server, identifier):
        self.server = server
        self.id = identifier
        self.cursor = None
        self.events = []
        self.pids = set()

    def detail(self):
        return self.server.cli("session", "inspect", self.id)

    def ready(self, expected="SessionReady"):
        detail = self.detail()
        if detail["state"] in ("Failed", "Ended"):
            raise AssertionError(detail)
        return detail["state"] == expected or (expected == "SessionReady" and detail["state"] == "EngineReady")

    @property
    def artifacts(self):
        return Path(self.detail()["artifact_dir"])

    def poll(self):
        args = ["session", "poll", self.id, "--wait-ms", "100"]
        if self.cursor is not None:
            args += ["--cursor", json.dumps(self.cursor)]
        reply = self.server.cli(*args)
        if reply["kind"] != "activity":
            raise AssertionError(reply)
        self.cursor = reply["cursor"]
        self.events.extend(entry["event"] for entry in reply["entries"])
        return self.events

    def outcome(self, request, command):
        def available():
            matches = [event for event in self.poll() if event.get("request_id") == request
                       and event["kind"] in ("completed", "rejected", "failed")]
            if len(matches) > 1:
                raise AssertionError(f"Doppelte Completion für {request}: {matches}")
            if matches and matches[0]["command"] != command:
                raise AssertionError(f"Falsch zugeordnete Completion: {matches[0]}")
            return matches[0] if matches else None
        return until(available, f"Keine Completion für {command}, Request {request}")

    def stop(self):
        if self.detail()["state"] not in ("Ended", "Failed"):
            self.server.cli("session", "stop", self.id, allow_empty=True)
        until(lambda: self.detail()["state"] in ("Ended", "Failed"), "Session-Stopp hängt")
        for pid in self.pids:
            until(lambda: process_gone(pid), f"App-Prozess {pid} lebt nach Stop weiter")


class Context:
    def __init__(self, server, app, mode, interface, steps=None):
        self.server, self.app, self.mode, self.interface = server, app, mode, interface
        from tests.session.steps import Steps
        self.steps = steps if steps is not None else Steps(server.directory / "steps.json")

    @contextmanager
    def session(self, config=None, ready=True):
        from tests.interfaces.json import Json
        from tests.interfaces.repl import Repl
        if config is None:
            config, _ = self.app.mode(self.mode)
        with self.server.session(config, ready=ready) as session:
            driver = Json(session) if self.interface == "json" or not ready else Repl(session)
            try:
                yield driver
            finally:
                driver.close()

    def program(self, name, mode="normal", tracing=False, provider="local", output="reports"):
        """Explicit infrastructure fixture; game contracts use the shared app."""
        targets = {
            "protocol": ("tests/connection/protocol_faults/peer/Cargo.toml", "process_fixture", "binary"),
            "observation": ("Cargo.toml", "observation_fixture", "example"),
            "activity": ("Cargo.toml", "activity_fixture", "example"),
            "abort": ("tests/reports/panic_abort/program/Cargo.toml", "panic_abort_fixture", "binary"),
        }
        manifest, target, kind = targets[name]
        package = "woodpecker" if kind == "example" else target
        path = self.server.directory / f"launch-{len(list(self.server.directory.glob('launch-*.toml')))}.toml"
        path.write_text(f'''version = 1
[launch]
manifest_path = {json.dumps(str(ROOT / manifest))}
package = {json.dumps(package)}
features = []
arguments = [{json.dumps(mode)}]
[launch.target]
kind = {json.dumps(kind)}
name = {json.dumps(target)}
[tick.pace]
kind = "as_fast_as_possible"
[report]
tracing_errors = {str(tracing).lower()}
output = {json.dumps(output)}
[report.provider]
kind = {json.dumps(provider)}
''')
        return path

    def probe(self, driver):
        if self.app.type_path is None:
            from tests.session.application import Unavailable
            raise Unavailable("Dieses Altszenario benötigt noch TestState; auf Signalstation-Abfragen umstellen")
        output = driver.command("inspect.query", {
            "source": "resources", "selector": {"kind": "type", "type_path": self.app.type_path},
            "projection": {"kind": "value"},
        })
        if len(output["items"]) != 1:
            raise AssertionError(output)
        result = output["items"][0]["result"]
        if result["type_path"] != self.app.type_path or result["value"]["status"] != "readable":
            raise AssertionError(output)
        state = result["value"]["value"]
        if type(state.get("ticks")) is not int or state["ticks"] < 0:
            raise AssertionError(f"TestState.ticks fehlt oder ist ungültig: {state}")
        pid = state.get("process_id")
        if type(pid) is not int or pid <= 0:
            raise AssertionError(f"TestState.process_id fehlt oder ist ungültig: {state}")
        driver.session.pids.add(pid)
        return state
