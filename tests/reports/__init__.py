"""Read-only Activity observations for report tests; no actions, starts or recovery."""
import json
from pathlib import Path
import time


def observe(server, session_id, predicate, *, cursor=None, timeout=15):
    events = []
    deadline = time.monotonic() + timeout
    while True:
        arguments = ["session", "poll", session_id, "--wait-ms", "100"]
        if cursor is not None:
            arguments += ["--cursor", json.dumps(cursor)]
        activity = server.cli(*arguments)
        if activity["kind"] != "activity":
            raise AssertionError(f"Expected Activity, not implicit Gap recovery: {activity}")
        events.extend(entry["event"] for entry in activity["entries"])
        cursor = activity["cursor"]
        if predicate(events):
            return events
        if time.monotonic() >= deadline:
            raise TimeoutError(f"Report observation deadline exceeded; see {server.artifacts}")


def audit_local_files(server, root, events):
    """Audit durable effects only after server exit/worker joins.

    This is not a final public Activity cursor. The internal report-completion
    owner test separately proves queue drain and one outcome per accepted job.
    """
    if server.process.poll() is None:
        raise AssertionError("Final report file audit requires a reaped server")
    root = Path(root)
    expected = set()
    for event in reports(events):
        result = event["result"]
        if result["status"] != "submitted":
            raise AssertionError(f"Expected completed local publication: {event}")
        outcome = result["outcome"]
        reference = outcome["reference"]
        if reference["kind"] != "file":
            raise AssertionError(f"Expected local file reference: {outcome}")
        relative = Path(reference["reference"]["path"])
        if relative.is_absolute() or ".." in relative.parts or relative.parts[:1] != ("reports",):
            raise AssertionError(f"Unsafe local report reference: {relative}")
        expected.add(relative)
        markdown = (root / relative).read_text(encoding="utf-8")
        marker = f"<!-- bug_hunter-signature: {event['report']['signature']['value']} -->"
        if markdown.splitlines().count(marker) != 1:
            raise AssertionError(f"Missing/duplicate signature in {relative}")
    actual = {path.relative_to(root) for path in (root / "reports").rglob("*.md")}
    if actual != expected:
        raise AssertionError(f"Final report files: expected {expected}, got {actual}")
    server.record(["observation", "final-local-report-files"], root=str(root),
                  files=sorted(map(str, actual)), activity_scope="bounded; completion owner: 4.82")


def failures(events):
    return [event["event"]["failure"] for event in events
            if event["kind"] == "event" and event["event"]["kind"] == "failure"]


def reports(events):
    return [event for event in events if event["kind"] == "report"]
