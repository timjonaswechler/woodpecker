"""Transport/process cleanup only; never starts a Session or submits a report implicitly."""
import json
import select
import time


def request(process, message, timeout=45):
    process.stdin.write(json.dumps(message) + "\n")
    process.stdin.flush()
    end = time.monotonic() + timeout
    while True:
        remaining = end - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("Live SDK operation unknown; never resubmitted")
        ready, _, _ = select.select([process.stdout], [], [], remaining)
        if ready:
            line = process.stdout.readline()
            if not line:
                raise AssertionError(f"Live driver exited {process.poll()}")
            result = json.loads(line)
            if not result["ok"]:
                raise AssertionError(result["error"])
            return result["value"]


def cleanup(process, artifacts):
    forced = False
    try:
        if process.poll() is None:
            request(process, {"action": "quit"}, timeout=10)
            process.wait(timeout=10)
    finally:
        if process.poll() is None:
            from tests._support import force_owned_tree
            forced = True
            force_owned_tree(process)
        for pipe in (process.stdin, process.stdout):
            try:
                pipe.close()
            except OSError:
                pass
        (artifacts / "driver-cleanup.json").write_text(json.dumps({
            "pid": process.pid, "returncode": process.returncode, "forced": forced}) + "\n")
    if forced or process.returncode != 0:
        raise AssertionError("Live driver needed harness cleanup; not a pass")


def pages(text):
    decoder = json.JSONDecoder()
    values = []
    while text.strip():
        page, offset = decoder.raw_decode(text.lstrip())
        if not isinstance(page, list):
            raise AssertionError("Expected issue-list page")
        values.extend(page)
        text = text.lstrip()[offset:]
    return values
