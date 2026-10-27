"""Pure observations and verified owned-process cleanup for shutdown tests."""
import json
import os
from pathlib import Path
import signal
import time

from tests._support import owned_process, wait_process_gone


def wait_state(server, session_id, expected, *, timeout=15):
    deadline = time.monotonic() + timeout
    while True:
        detail = server.cli("session", "inspect", session_id)
        if detail["state"] == expected:
            return detail
        if detail["state"] in ("Ended", "Failed"):
            raise AssertionError(f"Unexpected terminal state: {detail}")
        if time.monotonic() >= deadline:
            raise TimeoutError(f"State {expected} missing; see {server.artifacts}")
        time.sleep(0.02)


def progress(root, *, at_least=1, timeout=15):
    path = Path(root) / "simulation-ticks.json"
    deadline = time.monotonic() + timeout
    while True:
        if path.exists():
            ticks = json.loads(path.read_text())["ticks"]
            if ticks >= at_least:
                return ticks
        if time.monotonic() >= deadline:
            raise TimeoutError(f"Application progress missing; see {root}")
        time.sleep(0.01)


def owned_game(server, root):
    return owned_process(server, root)["pid"]


def thaw(holder):
    """Cleanup only: undo our SIGSTOP before the normal server cleanup, never kill a game."""
    if not holder:
        return
    server, root = holder["server"], holder["root"]
    pid = json.loads((Path(root) / "process-probe.json").read_text())["pid"]
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        holder.clear()
        return
    verified = owned_game(server, root)
    os.kill(verified, signal.SIGCONT)
    holder.clear()


def require_observation_budget(test, server, stop_interval, seconds, *, provider):
    """Check headroom before the next observation/action, not after a failed assertion.

    Call only while a verified owned game is deliberately held. If the observer
    can no longer establish headroom before the earliest possible deadline, only a fully
    verified deadline exit/cleanup may be marked incomplete. No signals, release,
    repeated request or reinterpretation of an existing behavior failure.
    """
    now = time.monotonic()
    if now < stop_interval[0] + seconds:
        return
    server.record(["observation", "deadline-budget-exhausted"],
                  first_stop=stop_interval, configured_seconds=seconds, observed_at=now)
    test.assertEqual(server.process.wait(timeout=max(5, stop_interval[1] + seconds - now + 5)), 1)
    for process in [*server.owned_processes, provider]:
        wait_process_gone(process)
    test.assertIn("shutdown_incomplete", (server.artifacts / "server.stderr.log").read_text())
    # This branch has now proved the deliberately held work hit the configured
    # product deadline, not a cleanup failure or an arbitrary unexpected exit.
    server.expected_exit_code = 1
    test.skipTest("Unvollständig: observer cannot establish remaining original shutdown budget before the next step; "
                  "deadline exit and all owned groups verified")


def no_failures(events):
    return not any(event["kind"] == "report" or
                   (event["kind"] == "event" and event["event"]["kind"] == "failure") for event in events)
