"""Configuration files, process ownership, CLI transport and logs; no app actions or ticks."""
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import tomllib


ROOT = Path(__file__).resolve().parents[1]
CLI = ROOT / "target/debug/woodpecker"
# Initial desktop positions in physical pixels; shared by the two-session tests.
WINDOW_POSITIONS = ((640, 400), (800, 500))


def load_config(path=ROOT / "tests/app.toml"):
    """Read a fresh configuration as data; never change the base file."""
    return tomllib.loads(Path(path).read_text(encoding="utf-8"))


def _toml_string(value):
    if not isinstance(value, str):
        raise TypeError("TOML keys and strings must be strings")
    return json.dumps(value, ensure_ascii=False).replace("\x7f", r"\u007f")


def _toml_value(value):
    """Encode the scalar, array and inline-table types used by our test configs."""
    if isinstance(value, str):
        return _toml_string(value)
    if isinstance(value, (bool, int, float)):
        return str(value).lower()
    if isinstance(value, list):
        return "[" + ", ".join(_toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(f"{_toml_string(key)} = {_toml_value(item)}"
                                for key, item in value.items()) + " }"
    raise TypeError(f"Unsupported test configuration value: {type(value).__name__}")


def write_config(path, configuration):
    """Write TOML only after a lossless parse check, including intentional invalid values.

    Inline tables keep this dependency-free; this is not a general TOML formatter.
    No SDK validation, session creation or simulation advancement happens here.
    """
    text = "".join(f"{_toml_string(key)} = {_toml_value(value)}\n"
                   for key, value in configuration.items())
    parsed = tomllib.loads(text)
    # Canonical JSON also compares non-finite values used by rejection tests;
    # ordinary dict equality cannot compare two independently parsed NaNs.
    if json.dumps(parsed, sort_keys=True) != json.dumps(configuration, sort_keys=True):
        raise ValueError("Written TOML does not preserve the requested configuration")
    Path(path).write_text(text, encoding="utf-8")


def _process_snapshot():
    """Read PID ancestry, group and start-time/command identity without signaling."""
    output = subprocess.run(["ps", "-axo", "pid=,ppid=,pgid=,lstart=,command="],
                            capture_output=True, text=True, check=True, timeout=5).stdout
    return {int(fields[0]): (int(fields[1]), int(fields[2]), " ".join(fields[3:]))
            for line in output.splitlines() if len(fields := line.split(None, 8)) == 9}


def force_owned_tree(process):
    """Failure cleanup only: verify live ownership, kill leaf groups first, then reap.

    No foreign, reparented or reused identity is adopted. This operation never
    turns forced cleanup into successful product evidence.
    """
    if process.poll() is not None:
        return []
    before = _process_snapshot()
    root = before.get(process.pid)
    if root is None:
        if process.poll() is not None:
            return []
        raise RuntimeError("Owned process missing before cleanup snapshot")
    if process.pid <= 1 or root[:2] != (os.getpid(), process.pid):
        raise RuntimeError("Refusing cleanup: root is not our separate-session child")
    depth = {process.pid: 0}
    while True:
        children = {pid: depth[parent] + 1 for pid, (parent, _, _) in before.items()
                    if parent in depth and pid not in depth}
        if not children:
            break
        depth.update(children)
    groups = {before[pid][1] for pid in depth}
    groups = sorted(groups, key=lambda group: (
        -min(depth[pid] for pid in depth if before[pid][1] == group), group))
    observed = [{"pid": pid, "group": before[pid][1]} for pid in depth]
    for group in groups:
        current = _process_snapshot()
        members = {pid for pid, (_, pgid, _) in current.items() if pgid == group}
        if not members:
            continue
        if group <= 1 or group == os.getpgrp() or current.get(process.pid) != root:
            raise RuntimeError("Refusing signal: unsafe group or owned root identity lost")
        for pid in members:
            if pid not in depth or current[pid] != before.get(pid):
                raise RuntimeError(f"Refusing signal: foreign/reused PID {pid} in group {group}")
            ancestor, seen = pid, set()
            while ancestor != process.pid and ancestor in current and ancestor not in seen:
                seen.add(ancestor)
                ancestor = current[ancestor][0]
            if ancestor != process.pid:
                raise RuntimeError(f"Refusing signal: live ancestry lost for {pid}")
        try:
            os.killpg(group, signal.SIGKILL)
        except ProcessLookupError:
            pass
    process.wait(timeout=10)
    for identity in observed:
        wait_process_gone(identity)
    return observed


def cleanup_server(server, artifacts, expected_exit_code=0, owned=()):
    """Reap our server at its declared exit status; forced cleanup is always an error."""
    expected_exit_code = expected_exit_code() if callable(expected_exit_code) else expected_exit_code
    forced = False
    escalated = []
    try:
        if server.poll() is None:
            server.send_signal(signal.SIGINT)
            try:
                server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                forced = True
                escalated = force_owned_tree(server)
    finally:
        (artifacts / "cleanup.json").write_text(json.dumps({
            "server_pid": server.pid,
            "returncode": server.returncode,
            "expected_returncode": expected_exit_code,
            "forced": forced,
            "forced_owned_processes": escalated,
        }, indent=2) + "\n", encoding="utf-8")
    # App/Cargo groups are separate from the server group. Never infer their
    # disappearance from the server's return code, and never signal them here.
    for process in owned:
        wait_process_gone(process)
    if forced or server.returncode != expected_exit_code:
        raise AssertionError(
            f"Server cleanup expected exit {expected_exit_code}, got {server.returncode}; "
            f"forced={forced}; see {artifacts}"
        )


class Server:
    def __init__(self, process, artifacts):
        self.process = process
        self.artifacts = artifacts
        self.ready = {}
        self.address = None
        self.owned_processes = []

    def record(self, arguments, **result):
        with (self.artifacts / "commands.jsonl").open("a", encoding="utf-8") as journal:
            journal.write(json.dumps({"arguments": arguments, **result}, ensure_ascii=False) + "\n")

    def run_cli(self, *arguments, timeout=10):
        """Return exit status and output from one invocation, including expected errors."""
        command = [str(CLI), "--address", self.address, *arguments]
        self.record(command, status="submitted")
        try:
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=timeout)
        except subprocess.TimeoutExpired:
            self.record(command, status="timeout")
            raise
        self.record(command, returncode=result.returncode, stdout=result.stdout, stderr=result.stderr)
        return result

    def cli(self, *arguments, timeout=10):
        """Require a successful invocation and decode its JSON; never retry."""
        result = self.run_cli(*arguments, timeout=timeout)
        if result.returncode != 0:
            raise AssertionError(f"CLI failed: {arguments}\n{result.stderr}\nSee {self.artifacts}")
        return json.loads(result.stdout)

    def result(self, session_id, pending, *, timeout=10, cursor=None):
        """Observe one accepted request from an explicit cursor; never resubmit or recover gaps."""
        if pending.get("kind") != "pending":
            raise ValueError(f"Expected a pending reply, got {pending}")
        request_id = pending["request_id"]
        command = pending["command"]
        deadline = time.monotonic() + timeout
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise TimeoutError(
                    f"Outcome unknown for {session_id}/{request_id} ({command}); "
                    f"deadline exceeded; see {self.artifacts}"
                )
            arguments = ["session", "poll", session_id, "--wait-ms",
                         str(max(1, min(500, int(remaining * 1000))))]
            if cursor is not None:
                arguments.extend(["--cursor", json.dumps(cursor)])
            activity = self.cli(*arguments, timeout=min(10, remaining))
            if activity.get("kind") != "activity":
                raise AssertionError(f"Expected activity, got {activity}; see {self.artifacts}")
            next_cursor = activity["cursor"]
            if next_cursor["session_id"] != session_id or (
                cursor is not None and next_cursor["position"] < cursor["position"]
            ):
                raise AssertionError(f"Uncorrelated activity cursor: {activity}; see {self.artifacts}")
            ended = None
            for entry in activity["entries"]:
                if entry["cursor"]["session_id"] != session_id:
                    raise AssertionError(f"Foreign activity entry: {entry}; see {self.artifacts}")
                event = entry["event"]
                if event.get("request_id") == request_id:
                    if event.get("command") != command:
                        raise AssertionError(f"Uncorrelated command outcome: {event}")
                    if event["kind"] in ("completed", "rejected", "failed"):
                        return event
                if event.get("kind") == "lifecycle" and event["state"] in ("Ended", "Failed"):
                    ended = event
            if ended is not None:
                raise AssertionError(
                    f"Outcome unknown for {session_id}/{request_id} ({command}): "
                    f"{ended}; see {self.artifacts}"
                )
            cursor = next_cursor

    def call(self, session_id, command, *, timeout=10):
        """Send exactly the supplied command once and return its completed output."""
        pending = self.cli("session", "submit", session_id, "--command", json.dumps(command))
        outcome = self.result(session_id, pending, timeout=timeout)
        if outcome["kind"] != "completed":
            raise AssertionError(f"Command did not complete: {outcome}; see {self.artifacts}")
        return outcome["output"]

    def read_resource(self, session_id, type_path):
        """Send exactly one read-only resource query and return its readable value."""
        query = {
            "command": "inspect.query",
            "arguments": {
                "source": "resources",
                "selector": {"kind": "type", "type_path": type_path},
                "projection": {"kind": "value"},
            },
        }
        pending = self.cli("session", "submit", session_id, "--command", json.dumps(query))
        outcome = self.result(session_id, pending)
        if outcome["kind"] != "completed":
            raise AssertionError(f"Resource query did not complete: {outcome}; see {self.artifacts}")
        items = outcome["output"]["items"]
        if len(items) != 1:
            raise AssertionError(f"Expected one resource, got {items}; see {self.artifacts}")
        item = items[0]
        result = item["result"]
        if item["kind"] != "resource" or result["kind"] != "value" or (
            result["type_path"] != type_path or result["value"]["status"] != "readable"
        ):
            raise AssertionError(f"Resource is not readable: {item}; see {self.artifacts}")
        return result["value"]["value"]


def start_server(test, label, *, expected_exit_code=0, env=None, activity_bytes=None, command_prefix=(), shutdown_seconds=2):
    """Register cleanup immediately, including if readiness or the test fails."""
    test.assertTrue(CLI.is_file(), "Build first: cargo build --locked --features cli --bin woodpecker")
    runs = ROOT / "target/test-runs"
    runs.mkdir(parents=True, exist_ok=True)
    artifacts = Path(tempfile.mkdtemp(prefix=f"{label}-", dir=runs))
    test.artifact_dirs = [*getattr(test, "artifact_dirs", []), str(artifacts)]
    print(f"\nArtefakte: {artifacts}", flush=True)
    stdout = test.enterContext((artifacts / "server.stdout.log").open("wb"))
    stderr = test.enterContext((artifacts / "server.stderr.log").open("wb"))
    command = [str(CLI), "--address", "127.0.0.1:0", "server", "start",
               "--artifact-dir", str(artifacts / "sessions"), "--shutdown-seconds", str(shutdown_seconds)]
    if activity_bytes is not None:
        command.extend(["--activity-bytes", str(activity_bytes)])
    command = [*map(str, command_prefix), *command]
    environment = None if env is None else {**os.environ, **env}
    process = subprocess.Popen(command, cwd=ROOT, stdin=subprocess.DEVNULL,
                               stdout=stdout, stderr=stderr, start_new_session=True,
                               env=environment)
    server = Server(process, artifacts)
    server.expected_exit_code = expected_exit_code
    test.addCleanup(cleanup_server, process, artifacts,
                    lambda: server.expected_exit_code, server.owned_processes)
    server.record(command, server_pid=process.pid)

    # Read stdout only. No session creation, implicit warm-up or game commands.
    deadline = time.monotonic() + 15
    while True:
        with (artifacts / "server.stdout.log").open("r", encoding="utf-8") as output:
            line = output.readline(65537)
        test.assertLessEqual(len(line), 65536, "Server ready response is too large")
        if line.endswith("\n"):
            break
        test.assertIsNone(process.poll(), f"Server exited before Ready; see {artifacts}")
        if time.monotonic() >= deadline:
            test.fail(f"Server did not announce Ready within 15 seconds; see {artifacts}")
        time.sleep(0.02)
    server.ready = json.loads(line)
    server.address = server.ready["address"]
    return server


def owned_process(server, root):
    """Observe a live test_app descendant and its separate group before any stop.

    Query only. The returned identity is never a license to signal a reused PID.
    Registered identities are also checked after server cleanup on assertion failure.
    """
    pid = json.loads((Path(root) / "process-probe.json").read_text())["pid"]
    if type(pid) is not int or pid <= 0 or server.process.poll() is not None:
        raise AssertionError("Invalid game probe or server already exited")
    current = pid
    for _ in range(16):
        result = subprocess.run(["ps", "-p", str(current), "-o", "ppid=", "-o", "command="],
                                capture_output=True, text=True, timeout=5)
        fields = result.stdout.strip().split(None, 1)
        if result.returncode != 0 or len(fields) != 2:
            break
        if current == pid and "test_app" not in fields[1]:
            raise AssertionError("Process probe no longer names the owned game")
        current = int(fields[0])
        if current == server.process.pid:
            group = os.getpgid(pid)
            if group in (os.getpgrp(), os.getpgid(server.process.pid)) or group <= 0:
                raise AssertionError("Game must have its own process group")
            observed = {"pid": pid, "group": group}
            if observed not in server.owned_processes:
                server.owned_processes.append(observed)
                server.record(["observation", "owned-game"], **observed)
            return observed
    raise AssertionError("Game PID is not a descendant of our live server")


def wait_process_gone(process, *, timeout=5):
    """Independently require PID and group absence; no cleanup signals or retries."""
    deadline = time.monotonic() + timeout
    while True:
        present = []
        for label, query in (("pid", os.kill), ("group", os.killpg)):
            try:
                query(process[label], 0)
            except ProcessLookupError:
                pass
            else:
                present.append(label)
        if not present:
            return
        if time.monotonic() >= deadline:
            raise AssertionError(f"Owned process/group still present: {process}; remaining={present}")
        time.sleep(0.01)


def read_tick_probe(root):
    """Read the existing atomic app counter, bracketed by observer monotonic times.

    No client connection, waiting, game action or retry. Callers own readiness
    and observation budgets. The app enables this only with its explicit opt-in.
    """
    before = time.monotonic()
    value = json.loads((Path(root) / "simulation-ticks.json").read_text(encoding="utf-8"))
    after = time.monotonic()
    if set(value) != {"ticks"} or type(value["ticks"]) is not int or value["ticks"] < 0:
        raise AssertionError(f"Invalid application tick probe: {value}")
    return {"ticks": value["ticks"], "before": before, "after": after}
