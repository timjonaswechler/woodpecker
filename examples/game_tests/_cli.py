"""Small public-CLI transport; no server starts or implicit game actions."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
CLI = ROOT / "target/debug/woodpecker"
CONFIG = Path(__file__).with_name("app.toml")
SEED = Path(__file__).with_name("settings.json")
TICKS = "test_app::woodpecker::SimulationTicks"
LEVEL = "test_app::level::LevelLoad"
PLAYER = "test_app::player::Player"
SETTINGS = "test_app::app::settings::Settings"


def arguments(description):
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--address", required=True, help="Prestarted local server, e.g. 127.0.0.1:4100")
    parser.add_argument(
        "--rendered", action="store_true",
        help="Acknowledge real windows; scoped permission is still required before running",
    )
    args = parser.parse_args()
    if not args.rendered:
        parser.error("--rendered is required; obtain scoped graphical permission first")
    if not __debug__:
        parser.error("Run without -O: these examples use assertions")
    return args


class Run:
    """Journal CLI calls and clean up only IDs confirmed by this example's creates."""

    def __init__(self, args, label):
        for binary in (CLI, ROOT / "target/debug/test_app"):
            if not binary.is_file():
                raise FileNotFoundError(f"Build first (see README): {binary}")
        self.address = args.address
        runs = ROOT / "target/game-example-runs"
        runs.mkdir(parents=True, exist_ok=True)
        self.artifacts = Path(tempfile.mkdtemp(prefix=f"{label}-", dir=runs))
        self.cursors = {}
        self.stop_attempted = set()
        self.stopped = set()
        print(f"Client journal: {self.artifacts}", flush=True)

    def note(self, **entry):
        with (self.artifacts / "cli.jsonl").open("a", encoding="utf-8") as journal:
            journal.write(json.dumps(entry, ensure_ascii=False) + "\n")

    def cli(self, *args, timeout=10):
        """Invoke the public CLI once. A timeout/error is never resubmitted."""
        command = [str(CLI), "--address", self.address, *args]
        self.note(command=command, status="invoking")
        try:
            result = subprocess.run(
                command, cwd=ROOT, stdin=subprocess.DEVNULL, capture_output=True,
                text=True, encoding="utf-8", timeout=timeout,
            )
        except BaseException as error:
            try:
                self.note(command=command, status="unconfirmed", error=repr(error))
            except BaseException as journal_error:
                error.add_note(f"Could not journal unconfirmed CLI call: {journal_error}")
            raise
        self.note(command=command, returncode=result.returncode,
                  stdout=result.stdout, stderr=result.stderr)
        if result.returncode:
            raise RuntimeError(
                f"CLI failed (not retried; any unconfirmed action remains unknown): "
                f"{args}\n{result.stdout}\n{result.stderr}"
            )
        return json.loads(result.stdout)

    def own(self, session_id):
        """Register only a full ID returned by the explicit create at the call site."""
        if not re.fullmatch(r"[0-9a-f]{32}", session_id) or session_id in self.cursors:
            raise ValueError(f"Expected a fresh full session ID: {session_id!r}")
        self.cursors[session_id] = {"session_id": session_id, "position": 0}
        print(f"Owned session: {session_id}", flush=True)
        return session_id

    def detail(self, session_id, *, timeout=5):
        if session_id not in self.cursors:
            raise ValueError("Refusing access to an unowned session")
        detail = self.cli("session", "inspect", session_id, timeout=timeout)
        if detail["id"] != session_id:
            raise RuntimeError(f"Uncorrelated session detail: {detail}")
        return detail

    def wait_engine(self, session_id):
        """Read-only observation, at most 90 s; the config bounds startup at 60 s."""
        deadline = time.monotonic() + 90
        while time.monotonic() < deadline:
            detail = self.detail(session_id, timeout=min(5, deadline - time.monotonic()))
            if detail["error"] is not None:
                raise RuntimeError(f"Session failed: {detail}")
            if detail["state"] == "EngineReady":
                root = Path(detail["artifact_dir"])
                if not root.is_absolute() or root.name != session_id:
                    raise RuntimeError(f"Unexpected public artifact_dir: {detail}")
                print(f"Session artifacts: {root}", flush=True)
                return detail
            if detail["state"] not in ("Starting", "SessionReady"):
                raise RuntimeError(f"Engine readiness failed: {detail}")
            time.sleep(0.05)
        raise TimeoutError(f"Engine readiness unconfirmed for {session_id}")

    def call(self, session_id, command, arguments, *, timeout=10):
        """Submit exactly once; await only its session/request/command-correlated result."""
        if session_id not in self.cursors:
            raise ValueError("Refusing access to an unowned session")
        deadline = time.monotonic() + timeout
        pending = self.cli(
            "session", "submit", session_id, "--command",
            json.dumps({"command": command, "arguments": arguments}, ensure_ascii=False),
            timeout=timeout,
        )
        if (pending.get("kind") != "pending" or pending.get("command") != command
                or type(pending.get("request_id")) is not int or pending["request_id"] <= 0):
            raise RuntimeError(f"Unconfirmed command acceptance: {pending}")
        request_id = pending["request_id"]
        while time.monotonic() < deadline:
            remaining = deadline - time.monotonic()
            cursor = self.cursors[session_id]
            batch = self.cli(
                "session", "poll", session_id, "--cursor", json.dumps(cursor),
                "--wait-ms", str(max(1, min(500, int(remaining * 1000)))),
                timeout=remaining,
            )
            if batch.get("kind") != "activity":
                raise RuntimeError(f"Outcome unknown; no Gap repair: {batch}")
            next_cursor = batch["cursor"]
            if (next_cursor["session_id"] != session_id
                    or next_cursor["position"] < cursor["position"]):
                raise RuntimeError(f"Uncorrelated Activity cursor: {batch}")
            outcome = None
            ended = False
            position = cursor["position"]
            for entry in batch["entries"]:
                entry_cursor, event = entry["cursor"], entry["event"]
                if (entry_cursor["session_id"] != session_id
                        or not position < entry_cursor["position"] <= next_cursor["position"]):
                    raise RuntimeError(f"Uncorrelated Activity entry: {entry}")
                position = entry_cursor["position"]
                if event.get("request_id") == request_id:
                    if event.get("command") != command:
                        raise RuntimeError(f"Uncorrelated command result: {event}")
                    if event["kind"] in ("completed", "rejected", "failed"):
                        if outcome is not None:
                            raise RuntimeError(f"Duplicate terminal result: {event}")
                        outcome = event
                ended |= event.get("kind") == "lifecycle" and event["state"] in ("Ended", "Failed")
            self.cursors[session_id] = next_cursor
            if outcome is not None:
                if outcome["kind"] != "completed":
                    raise RuntimeError(f"Command did not complete: {outcome}")
                return outcome["output"]
            if ended:
                raise RuntimeError(f"Outcome unknown: session ended during {command}/{request_id}")
        raise TimeoutError(f"Outcome unknown: {session_id}/{request_id} ({command}); not resubmitted")

    def resource(self, session_id, type_path):
        """One read-only public query; require exactly one readable resource."""
        items = self.call(session_id, "inspect.query", {
            "source": "resources", "selector": {"kind": "type", "type_path": type_path},
            "projection": {"kind": "value"},
        })["items"]
        if len(items) != 1:
            raise RuntimeError(f"Expected one {type_path}: {items}")
        item, result = items[0], items[0]["result"]
        if (item["kind"] != "resource" or result["kind"] != "value"
                or result["type_path"] != type_path or result["value"]["status"] != "readable"):
            raise RuntimeError(f"Resource is not readable: {item}")
        return result["value"]["value"]

    def stop(self, session_id):
        """One public management stop, then bounded read-only Ended observation."""
        if session_id not in self.cursors or session_id in self.stop_attempted:
            raise RuntimeError(f"Refusing unowned or repeated stop: {session_id}")
        self.stop_attempted.add(session_id)
        deadline = time.monotonic() + 40
        self.cli("session", "stop", session_id, timeout=5)
        while time.monotonic() < deadline:
            detail = self.detail(session_id, timeout=min(5, deadline - time.monotonic()))
            if detail["state"] == "Failed" or detail["error"] is not None:
                raise RuntimeError(f"Owned-session cleanup failed: {detail}")
            if detail["state"] == "Ended":
                self.stopped.add(session_id)
                self.note(cleanup="ended", session_id=session_id)
                return
            time.sleep(0.05)
        raise TimeoutError(f"Owned-session cleanup unconfirmed: {session_id}; no forced kill")

    def __enter__(self):
        return self

    def __exit__(self, exc_type, error, traceback):
        failures = []
        for session_id in self.cursors:
            if session_id in self.stopped:
                continue
            try:
                if session_id in self.stop_attempted:
                    raise RuntimeError(f"Previous stop unconfirmed for {session_id}; not resubmitted")
                self.stop(session_id)
            except BaseException as cleanup_error:
                failures.append(cleanup_error)
        try:
            self.note(
                result="failed" if error is not None or failures else "passed",
                error=repr(error) if error is not None else None,
                cleanup_errors=[repr(failure) for failure in failures],
            )
        except BaseException as journal_error:
            failures.append(journal_error)
        if failures:
            if error is not None:
                for failure in failures:
                    error.add_note(f"Cleanup also failed: {failure}")
            else:
                raise BaseExceptionGroup("Owned-session cleanup failed", failures)
        return False
