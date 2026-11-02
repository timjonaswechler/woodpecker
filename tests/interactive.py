"""Run compatible tests in one persistent app. Explicit --stop owns its shutdown."""
import argparse
import contextlib
import hashlib
import io
import json
import os
from pathlib import Path
import select
import socket
import subprocess
import sys
import tempfile
import uuid

from tests import _artifacts, _shared, acceptance
from tests._support import ROOT, use_products

STATE = ROOT / "target/shared-app.json"


def implementation():
    # The daemon imports cases once. Never silently execute old Python after an edit.
    paths = sorted(path for path in (ROOT / "tests").rglob("*.py")
                   if "runner_checks" not in path.parts and path.name != "_cargo_cache.py")
    paths += [ROOT / "tests/app.toml", ROOT / "tests/settings.json"]
    digest = hashlib.sha256()
    for path in paths:
        digest.update(str(path.relative_to(ROOT)).encode() + b"\0" + path.read_bytes())
    return digest.hexdigest()


def plan(arguments):
    if (not arguments or len(arguments) % 2 or any(arguments[index] not in ("--test", "--category")
            or not arguments[index + 1] or arguments[index + 1].startswith("-")
            for index in range(0, len(arguments), 2))):
        raise ValueError("Only explicit --test ID or --category NAME selections are accepted")
    if "--test" in arguments and "--category" in arguments:
        raise ValueError("Choose --test or --category, not both")
    output = io.StringIO()
    with contextlib.redirect_stdout(output):
        code = acceptance.main([*arguments, "--list"])
    data = json.loads(output.getvalue())
    if code or data["inventory_errors"]:
        raise ValueError(str(data["inventory_errors"]))
    isolated = [key for key, row in data["selected"].items() if row.get("module") not in _shared.MODULES]
    if isolated:
        raise ValueError(f"Persistent app refuses isolated cases: {isolated}; use tests.acceptance separately")
    return data


def request(state, operation, **fields):
    with socket.create_connection(("127.0.0.1", state["port"]), timeout=300) as connection:
        connection.sendall((json.dumps(dict(instance=state["instance"], operation=operation, **fields)) + "\n").encode())
        response = json.loads(connection.makefile("rb").readline(1024 * 1024))
    if not response.get("ok"):
        raise RuntimeError(response.get("error", "Shared host did not confirm the request"))
    return response


def disconnected(connection):
    try:
        return bool(select.select([connection], [], [], 0)[0]) and not connection.recv(1, socket.MSG_PEEK)
    except OSError:
        return True


def host(manifest):
    import fcntl
    STATE.parent.mkdir(parents=True, exist_ok=True)
    # One host per checkout, including simultaneous first clients. No PID adoption.
    with (STATE.parent / "shared-app.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        if STATE.exists():
            raise RuntimeError("Existing shared-app state needs explicit inspection/stop; refusing replacement")
        products = _artifacts.load(manifest)
        if not {"bin:woodpecker", "bin:test_app"} <= products.keys():
            raise ValueError("Persistent app needs prepared CLI and automation app")
        directory = Path(tempfile.mkdtemp(prefix="shared-host-", dir=ROOT / "target/test-runs"))
        identity = implementation()
        game = _shared.SharedGame()
        instance = uuid.uuid4().hex
        with use_products(products), (directory / "host.log").open("a", buffering=1) as log, socket.socket() as listener:
            try:
                with contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
                    game.acquire(_shared._Owner(), "warm-start")
                listener.bind(("127.0.0.1", 0))
                listener.listen(4)
                state = dict(instance=instance, port=listener.getsockname()[1], host_pid=os.getpid(),
                    app_pid=game.identity["pid"], session_id=game.session_id, address=game.server.address,
                    manifest=str(manifest.resolve()), artifacts=str(directory), implementation=identity)
                temporary = STATE.with_suffix(".preparing")
                temporary.write_text(json.dumps(state, indent=2) + "\n")
                temporary.chmod(0o600)
                temporary.replace(STATE)
                print(json.dumps(dict(ok=True, **state)), flush=True)
                while True:
                    connection, _ = listener.accept()
                    stop = False
                    with connection:
                        try:
                            connection.settimeout(10)
                            message = json.loads(connection.makefile("rb").readline(65537))
                            connection.settimeout(None)
                            if message.get("instance") != instance:
                                raise ValueError("Shared host identity mismatch")
                            operation = message["operation"]
                            if operation == "status":
                                response = dict(ok=True, **state, contaminated=game.failed,
                                                server_exit=game.server.process.poll())
                            elif operation == "stop":
                                with contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
                                    game.close()
                                stop = True
                                response = dict(ok=True, stopped=True, app_pid=game.identity["pid"])
                            elif operation == "run":
                                if message.get("implementation") != identity:
                                    raise ValueError("Runner implementation changed; explicit stop/restart required")
                                if game.failed:
                                    raise RuntimeError("Shared app is contaminated; left open for inspection, no retry/replacement")
                                arguments = message["arguments"]
                                plan(arguments)
                                # Full validation precedes every run; never rebuild or substitute an app.
                                if _artifacts.load(manifest) != products:
                                    raise ValueError("Prepared products changed; explicit stop/restart required")
                                output = io.StringIO()
                                with contextlib.redirect_stdout(output), contextlib.redirect_stderr(log):
                                    code = acceptance.main([*arguments, "--rendered", "--artifacts", str(manifest)],
                                        shared_game=game, should_stop=lambda: disconnected(connection))
                                response = dict(ok=True, code=code, output=output.getvalue(), app_pid=game.identity["pid"])
                            else:
                                raise ValueError(f"Unknown operation: {operation}")
                        except Exception as error:
                            response = dict(ok=False, error=f"{type(error).__name__}: {error}")
                            print(response["error"], file=log, flush=True)
                        try:
                            connection.sendall((json.dumps(response) + "\n").encode())
                        except OSError:
                            pass  # The current case finished; no subsequent cases run after disconnect.
                    if stop:
                        break
            finally:
                try:
                    with contextlib.redirect_stdout(log), contextlib.redirect_stderr(log):
                        game.close()
                finally:
                    if STATE.is_file() and json.loads(STATE.read_text()).get("instance") == instance:
                        STATE.unlink()


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts", type=Path)
    parser.add_argument("--test", action="append", default=[])
    parser.add_argument("--category", action="append", default=[])
    actions = parser.add_mutually_exclusive_group()
    actions.add_argument("--list", action="store_true")
    actions.add_argument("--status", action="store_true")
    actions.add_argument("--stop", action="store_true")
    actions.add_argument("--host", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    try:
        if args.host:
            host(args.artifacts)
            return 0
        arguments = [arg for name in args.test for arg in ("--test", name)]
        arguments += [arg for name in args.category for arg in ("--category", name)]
        if args.test and args.category:
            raise ValueError("Choose --test or --category, not both")
        if not arguments:
            arguments = [arg for name in ("keyboard", "pointer", "text") for arg in ("--category", name)]
        if args.list:
            print(json.dumps(plan(arguments), indent=2))
            return 0
        if args.status or args.stop:
            if not STATE.is_file():
                print("No persistent app is running.")
                return 0
            print(json.dumps(request(json.loads(STATE.read_text()), "stop" if args.stop else "status"), indent=2))
            return 0
        plan(arguments)  # Reject lifecycle/isolation cases before any process is started.
        if os.environ.get("WOODPECKER_TEST_RENDERED") != "1":
            raise ValueError("Scoped graphics approval required: WOODPECKER_TEST_RENDERED=1")
        if STATE.is_file():
            state = json.loads(STATE.read_text())
            if args.artifacts and args.artifacts.resolve() != Path(state["manifest"]):
                raise ValueError("Existing app uses another manifest; explicit stop required, never automatic replacement")
            _artifacts.load(Path(state["manifest"]))
        else:
            if args.artifacts is None:
                raise ValueError("First run needs --artifacts MANIFEST; no implicit build")
            products = _artifacts.load(args.artifacts)
            if not {"bin:woodpecker", "bin:test_app"} <= products.keys():
                raise ValueError("Persistent app needs prepared CLI and automation app")
            STATE.parent.mkdir(parents=True, exist_ok=True)
            log_path = STATE.parent / "shared-app-launch.log"
            with log_path.open("a") as log:
                child = subprocess.Popen([sys.executable, "-m", "tests.interactive", "--host", "--artifacts",
                    str(args.artifacts.resolve())], cwd=ROOT, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                    stderr=log, start_new_session=True, text=True)
                if not select.select([child.stdout], [], [], 90)[0]:
                    raise TimeoutError(f"Shared host startup is unresolved; no replacement; see {log_path}")
                line = child.stdout.readline()
                child.stdout.close()
                if not line:
                    raise RuntimeError(f"Shared host startup failed; see {log_path}")
                state = json.loads(line)
        response = request(state, "run", arguments=arguments, implementation=implementation())
        print(response["output"], end="")
        print(f"App remains open: PID {response['app_pid']}; stop: python3 -m tests.interactive --stop")
        return response["code"]
    except Exception as error:
        print(f"{type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
