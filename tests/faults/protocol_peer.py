"""External JSONL fault peer, not a Bevy application or headless game mode.

The generated CARGO adapter delegates only metadata/version to real Cargo and
executes this peer for the test-owned sentinel package's run command.
"""
import json
import os
from pathlib import Path
import sys


MODES = {
    "wrong-version", "invalid-capabilities", "missing-fixed-clock", "missing-engine-startup",
    "malformed-known", "wrong-name", "bad-output", "malformed-unassigned", "unknown-id", "duplicate",
    "recording-end", "replay-ok", "repl", "shutdown-starting", "shutdown-hold",
    *("report-marker-" + name for name in ("valid", "json", "checksum", "base64", "duplicate", "missing", "truncated", "process-payload")),
    *("report-before-" + name for name in ("panic", "tracing", "invalid", "exit-0", "exit-17")),
}


def cargo(arguments):
    if arguments[0] in ("metadata", "--version"):
        real = os.environ["WOODPECKER_TEST_REAL_CARGO"]
        os.execv(real, [real, *arguments])
    if arguments[0] != "run":
        raise SystemExit("unexpected Cargo operation")
    manifest = arguments[arguments.index("--manifest-path") + 1]
    assert Path(manifest).resolve() == Path(os.environ["WOODPECKER_TEST_PEER_MANIFEST"]).resolve()
    assert arguments[arguments.index("--package") + 1] == "woodpecker-protocol-fixture"
    assert arguments[arguments.index("--bin") + 1] == "protocol-peer"
    mode, = arguments[arguments.index("--") + 1:]
    assert mode in MODES
    os.execv(sys.executable, [sys.executable, str(Path(__file__).resolve()), "peer", mode])


def peer(mode):
    assert mode in MODES
    if mode.startswith("report-"):
        os.execv(sys.executable, [sys.executable, str(Path(__file__).with_name("report_peer.py")), mode])
    root = Path(os.environ["WOODPECKER_ARTIFACT_DIR"])
    (root / "protocol-peer.json").write_text(json.dumps({"pid": os.getpid(), "mode": mode}) + "\n")

    def journal(kind, value):
        with (root / "protocol-peer.jsonl").open("a") as output:
            output.write(json.dumps({"kind": kind, "value": value}) + "\n")

    def emit(value):
        line = value if isinstance(value, str) else json.dumps(value)
        journal("stdout", line)
        print(line, flush=True)

    def completed(request, output, **overrides):
        emit({"status": "completed", "request_id": request["request_id"],
              "command": request["command"], "output": output, **overrides})

    ready = {"status": "ready", "version": 3,
             "capabilities": {"screenshot": False, "fixed_clock": True}}
    if mode == "wrong-version":
        ready["version"] = 4
    elif mode == "invalid-capabilities":
        ready["capabilities"]["screenshot"] = "not a boolean"
    elif mode == "missing-fixed-clock":
        del ready["capabilities"]["fixed_clock"]
    if mode == "shutdown-starting":
        import time
        (root / "before-ready").write_text("external process held before Ready\n")
        while not (root / "release-ready").exists():
            time.sleep(0.02)
    emit(ready)
    pending = None
    faulted = False
    for line in sys.stdin:
        request = json.loads(line)
        journal("stdin", request)
        command = request["command"]
        if command == "shutdown":
            assert pending is None
            if mode == "shutdown-hold":
                import time
                (root / "shutdown-entered").write_text("owned peer holding Shutdown\n")
                while not (root / "release-shutdown").exists():
                    time.sleep(0.02)
            completed(request, None)
            return
        if command == "tick.warp.stop":
            was_running = pending is not None
            if pending is not None:
                completed(pending, {"requested_ticks": pending["arguments"]["ticks"],
                                    "executed_ticks": 0, "outcome": "stopped"})
                pending = None
            completed(request, {"was_running": was_running})
            continue
        if mode == "recording-end":
            if command == "input.text.input":
                completed(request, None)
            elif command == "tick.warp.start":
                assert pending is None
                pending = request
            else:
                assert command == "inspect.query" and pending is not None
                return  # EOF with two unanswered requests; no game or renderer.
            continue
        if mode in ("repl", "shutdown-hold", "shutdown-starting"):
            # An external protocol probe, not a simulated world: Warp stays open
            # until explicit Stop; queries expose only an absent resource/empty list.
            if command == "tick.warp.start":
                assert pending is None
                pending = request
            elif command == "inspect.query":
                arguments = request["arguments"]
                if arguments["source"] == "resources" and arguments["selector"]["kind"] == "type":
                    result = {"kind": "value", "type_path": arguments["selector"]["type_path"],
                              "value": {"status": "unavailable", "reason": "not_registered"}}
                    completed(request, {"items": [{"kind": "resource", "result": result}]})
                else:
                    completed(request, {"items": []})
            else:
                raise AssertionError("Unexpected external REPL probe command")
            continue
        if mode == "replay-ok":
            if command == "tick.warp.start":
                completed(request, {"requested_ticks": request["arguments"]["ticks"],
                                    "executed_ticks": request["arguments"]["ticks"], "outcome": "completed"})
            else:
                assert command in ("input.keyboard.press", "input.keyboard.release",
                                   "input.pointer.move_to", "input.pointer.press")
                completed(request, None)
            continue
        assert command == "tick.warp.start" and not faulted
        faulted = True
        normal = {"requested_ticks": request["arguments"]["ticks"],
                  "executed_ticks": request["arguments"]["ticks"], "outcome": "completed"}
        if mode == "malformed-known":
            emit({"status": "damaged", "request_id": request["request_id"]})
        elif mode == "wrong-name":
            completed(request, normal, command="inspect.query")
        elif mode == "bad-output":
            completed(request, {"not": "a warp completion"})
        elif mode == "duplicate":
            completed(request, normal)
            completed(request, normal)
        elif mode == "malformed-unassigned":
            pending = request
            emit("{broken")
        elif mode == "unknown-id":
            pending = request
            completed(request, normal, request_id=request["request_id"] + 10000)
        else:
            raise AssertionError("Invalid handshake should not receive commands")


if __name__ == "__main__":
    if sys.argv[1] == "cargo":
        cargo(sys.argv[2:])
    else:
        assert sys.argv[1] == "peer"
        peer(sys.argv[2])
