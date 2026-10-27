"""Owned literal wire partner for internal coordinator tests; not a game/World."""
import json
import os
from pathlib import Path
import select
import sys

def main(root):
    (root / "peer.json").write_text(json.dumps({"pid": os.getpid(), "group": os.getpgrp()}) + "\n")

    def completed(request, output):
        print(json.dumps({"status": "completed", "request_id": request["request_id"],
                          "command": request["command"], "output": output}), flush=True)

    print(json.dumps({"status": "ready", "version": 3,
                      "capabilities": {"screenshot": False, "fixed_clock": True}}), flush=True)
    pending = None
    buffer = b""
    while True:
        if pending is not None and (root / "release-warp").exists():
            completed(pending, {"requested_ticks": pending["arguments"]["ticks"],
                                "executed_ticks": pending["arguments"]["ticks"], "outcome": "completed"})
            pending = None
        readable, _, _ = select.select([0], [], [], 0.01)
        if not readable:
            continue
        chunk = os.read(0, 65536)
        if not chunk:
            break
        buffer += chunk
        while b"\n" in buffer:
            line, buffer = buffer.split(b"\n", 1)
            request = json.loads(line)
            with (root / "wire.jsonl").open("a") as stream:
                stream.write(json.dumps(request) + "\n")
            command = request["command"]
            if command == "test.fence":
                # Diagnostic pipe fence: no SUT command/response or game behavior.
                (root / request["label"]).write_text("all earlier pipe lines observed\n")
                continue
            if command == "shutdown":
                assert pending is None
                completed(request, None)
                raise SystemExit(0)
            if command == "tick.warp.start":
                assert pending is None
                pending = request
            elif command == "tick.warp.stop":
                running = pending is not None
                if running:
                    completed(pending, {"requested_ticks": pending["arguments"]["ticks"],
                                        "executed_ticks": 0, "outcome": "stopped"})
                    pending = None
                completed(request, {"was_running": running})
            elif command == "inspect.query":
                completed(request, {"items": []})
            else:
                assert command in ("input.keyboard.press", "input.keyboard.release")
                completed(request, None)


if __name__ == "__main__":
    main(Path(sys.argv[1]))
