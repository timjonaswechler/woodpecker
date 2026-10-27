"""External stderr/handshake peer. No Bevy, product imports, network or game."""
import base64
import hashlib
import json
import os
from pathlib import Path
import sys


PREFIX = b"\x1eWOODPECKER_REPORT:"


def frames(payload, *, event="external", chunk=64):
    encoded = json.dumps(payload, separators=(",", ":")).encode()
    pieces = [encoded[i:i + chunk] for i in range(0, len(encoded), chunk)]
    return [PREFIX + json.dumps({"version": 1, "event": event, "index": i, "count": len(pieces),
            "checksum": hashlib.sha256(encoded).hexdigest(), "data": base64.b64encode(data).decode()},
            separators=(",", ":")).encode() + b"\n" for i, data in enumerate(pieces)]


def peer(mode):
    root = Path(os.environ["WOODPECKER_ARTIFACT_DIR"])
    (root / "protocol-peer.json").write_text(json.dumps({"pid": os.getpid(), "mode": mode}) + "\n")
    failure = {"origin": {"kind": "panic", "location": {"file": "external.rs", "line": 7, "column": 2},
                         "backtrace": "external diagnostic\nframe", "backtrace_status": "captured"},
               "message": "external report panic"}
    normal = frames({"kind": "failure", "failure": failure})
    if mode.startswith("report-before-"):
        variant = mode.removeprefix("report-before-")
        if variant == "panic":
            os.write(2, b"".join(normal))
        elif variant == "tracing":
            failure = {"origin": {"kind": "tracing_error", "target": "external", "location": None},
                       "message": "external early tracing"}
            os.write(2, b"".join(frames({"kind": "failure", "failure": failure})))
        elif variant == "invalid":
            os.write(2, PREFIX + b"{broken\n")
        else:
            assert variant in ("exit-0", "exit-17")
        if variant == "invalid":
            print("not a Ready response", flush=True)
        return 17 if variant == "exit-17" else 0
    variant = mode.removeprefix("report-marker-")
    raw = b"".join(normal)
    if variant == "json":
        raw = PREFIX + b"{broken\n"
    elif variant == "checksum":
        data = json.loads(normal[0][len(PREFIX):]); data["checksum"] = "0" * 64
        raw = PREFIX + json.dumps(data).encode() + b"\n" + b"".join(normal[1:])
    elif variant == "base64":
        data = json.loads(normal[0][len(PREFIX):]); data["data"] = "!"
        raw = PREFIX + json.dumps(data).encode() + b"\n"
    elif variant == "duplicate":
        raw = normal[0] + b"".join(normal)
    elif variant == "missing":
        raw = normal[0]
    elif variant == "truncated":
        raw = PREFIX + b'{"version":1'
    elif variant == "process-payload":
        raw = b"".join(frames({"kind": "failure", "failure": {
            "origin": {"kind": "process_exit", "status": "invented"}, "message": "invented"}}))
    else:
        assert variant == "valid"
    before = b"before\xff\n"
    after = b"after\xfe" if variant == "truncated" else b"after\xfe\n"
    expected = before + after if variant == "valid" else before + raw + after
    (root / "expected-human-stderr.bin").write_bytes(expected)
    os.write(2, before + raw[:3])  # Split inside the prefix across the explicit Ready/command boundary.
    print(json.dumps({"status": "ready", "version": 3,
                      "capabilities": {"screenshot": False, "fixed_clock": True}}), flush=True)
    request = json.loads(sys.stdin.readline())
    assert request["command"] == "tick.warp.start"
    os.write(2, raw[3:] + after)
    if variant != "valid":
        return 17  # EOF flushes partial groups; damaged panic must not suppress ProcessExit.
    print(json.dumps({"status": "completed", "request_id": request["request_id"],
                      "command": request["command"], "output": {
                          "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}}), flush=True)
    for line in sys.stdin:
        request = json.loads(line)
        command = request["command"]
        assert command in ("tick.warp.stop", "shutdown")
        output = {"was_running": False} if command == "tick.warp.stop" else None
        print(json.dumps({"status": "completed", "request_id": request["request_id"],
                          "command": command, "output": output}), flush=True)
        if command == "shutdown":
            return 0


if __name__ == "__main__":
    raise SystemExit(peer(sys.argv[1]))
