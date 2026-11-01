"""External loopback WebSocket fault peer for the real script CLI, never a game.

Implements only the literal five-command probe's outer-wire conversation.
The parent chooses each fault explicitly. No product imports or state-machine clone.
"""
import base64
import hashlib
import json
import os
from pathlib import Path
import select
import socket
import struct
import sys
import time


def record(root, kind, **fields):
    with (root / "wire.jsonl").open("a", encoding="utf-8") as output:
        output.write(json.dumps({"kind": kind, **fields}) + "\n")


def receive_exact(connection, length):
    result = bytearray()
    while len(result) < length:
        chunk = connection.recv(length - len(result))
        if not chunk:
            raise EOFError("CLI closed before the selected fault")
        result.extend(chunk)
    return bytes(result)


def receive(connection, root, *, allow_close=False):
    first, second = receive_exact(connection, 2)
    if (first != 0x81 and not (allow_close and first == 0x88)) or not second & 0x80:
        raise ValueError("Expected one complete masked text frame")
    length = second & 0x7f
    if length == 126:
        length = struct.unpack("!H", receive_exact(connection, 2))[0]
    elif length == 127:
        length = struct.unpack("!Q", receive_exact(connection, 8))[0]
    if length > 65536:
        raise ValueError("Probe frame exceeds 64 KiB")
    mask = receive_exact(connection, 4)
    payload = receive_exact(connection, length)
    if first == 0x88:
        record(root, "client_close")
        connection.sendall(b"\x88\x00")
        return None
    value = json.loads(bytes(byte ^ mask[index % 4] for index, byte in enumerate(payload)))
    record(root, "request", value=value)
    return value


def respond(connection, root, request, result, *, wrong_call=False):
    value = {"version": 1, "call": request["call"] + (100 if wrong_call else 0), "result": result}
    payload = json.dumps(value).encode("utf-8")
    header = (bytes((0x81, len(payload))) if len(payload) < 126 else
              bytes((0x81, 126)) + struct.pack("!H", len(payload)))
    connection.sendall(header + payload)
    record(root, "response", value=value)


def conversation(connection, root, mode, session_id):
    header = bytearray()
    while not header.endswith(b"\r\n\r\n"):
        if len(header) >= 32768:
            raise ValueError("Upgrade header too large")
        header.extend(receive_exact(connection, 1))
    lines = header.decode("ascii").split("\r\n")
    if lines[0] != f"GET /v1/sessions/{session_id}/connect HTTP/1.1":
        raise ValueError("Unexpected endpoint")
    fields = {key.lower(): value.strip() for key, value in
              (line.split(":", 1) for line in lines[1:] if line)}
    accept = base64.b64encode(hashlib.sha1(
        (fields["sec-websocket-key"] + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest())
    connection.sendall(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\n"
                       b"Connection: Upgrade\r\nSec-WebSocket-Accept: " + accept + b"\r\n\r\n")
    cursor = lambda position: {"session_id": session_id, "position": position}
    entry = lambda position, event: {"cursor": cursor(position), "event": event}
    expect_call = 0

    def request(kind, command=None):
        nonlocal expect_call
        value = receive(connection, root)
        expect_call += 1
        if value["version"] != 1 or value["call"] != expect_call:
            raise ValueError("Unexpected request envelope")
        operation = value["operation"]
        if operation["kind"] != kind or (command is not None and operation["command"] != command):
            raise ValueError(f"Unexpected probe operation: {operation}")
        if kind == "poll":
            expected_position = 0 if expect_call == 3 else 2
            if operation["cursor"] != cursor(expected_position) or operation["wait_ms"] != 0:
                raise ValueError("Unexpected probe continuation")
        return value

    initial = request("snapshot")
    respond(connection, root, initial, {"kind": "snapshot", "snapshot": {
        "cursor": cursor(0), "state": "SessionReady", "pending": []}})
    stop = {"command": "tick.warp.stop", "arguments": {}}
    first = request("submit", stop)
    respond(connection, root, first, {"kind": "pending", "request_id": 1, "command": "tick.warp.stop"})
    poll = request("poll")
    respond(connection, root, poll, {"kind": "activity", "cursor": cursor(2), "entries": [
        entry(1, {"kind": "pending", "request_id": 1, "command": "tick.warp.stop"}),
        entry(2, {"kind": "completed", "request_id": 1, "command": "tick.warp.stop",
                  "output": {"was_running": False}}),
    ]})
    warp = {"command": "tick.warp.start", "arguments": {"ticks": 60}}
    second = request("submit", warp)
    respond(connection, root, second, {"kind": "pending", "request_id": 2, "command": "tick.warp.start"})
    poll = request("poll")
    if mode == "gap":
        respond(connection, root, poll, {"kind": "gap", "cursor": cursor(99),
                                         "message": "fixture evicted Activity"})
    elif mode == "wrong-command":
        respond(connection, root, poll, {"kind": "activity", "cursor": cursor(3), "entries": [
            entry(3, {"kind": "completed", "request_id": 2, "command": "tick.warp.stop",
                      "output": {"was_running": False}}),
        ]})
    else:
        respond(connection, root, poll, {"kind": "activity", "cursor": cursor(3), "entries": [
            entry(3, {"kind": "pending", "request_id": 2, "command": "tick.warp.start"}),
        ]}, wrong_call=mode == "wrong-poll-call")
        if mode in ("drop-submit", "wrong-submit-call"):
            third = request("submit", {
                "command": "input.keyboard.press", "arguments": {"key": "shift_left"}})
            # Receipt is deliberately unconfirmed: the client must not assume it was unsent.
            record(root, "unconfirmed_receipt", command_index=2)
            if mode == "wrong-submit-call":
                respond(connection, root, third, {
                    "kind": "pending", "request_id": 3, "command": "input.keyboard.press"}, wrong_call=True)
    record(root, "fault_injected", mode=mode)
    # Only the transport-fault variant closes. Gap/correlation faults leave the
    # socket usable, so implicit recovery RPCs cannot hide behind a second EOF.
    if mode == "drop-submit":
        connection.shutdown(socket.SHUT_RDWR)


def main():
    mode, directory, session_id = sys.argv[1:]
    if mode not in {"drop-submit", "wrong-submit-call", "wrong-poll-call", "gap", "wrong-command"}:
        raise ValueError("Unknown explicit fault")
    root = Path(directory)
    deadline = time.monotonic() + 20
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.settimeout(10)
        print(json.dumps({"state": "Ready", "address": f"127.0.0.1:{listener.getsockname()[1]}",
                          "pid": os.getpid(), "mode": mode}), flush=True)
        connection, _ = listener.accept()
        record(root, "connection")
        with connection:
            connection.settimeout(5)
            conversation(connection, root, mode, session_id)
            active = None if mode == "drop-submit" else connection
            # Observe the entire CLI lifetime, including in-connection Gap recovery.
            while True:
                if time.monotonic() >= deadline:
                    raise TimeoutError("Parent did not finish the CLI within 20 seconds")
                readable, _, _ = select.select(
                    [listener, *([active] if active is not None else [])], [], [], 0.02)
                for source in readable:
                    if source is listener:
                        extra, _ = listener.accept()
                        record(root, "unexpected_connection")
                        extra.close()
                    elif not connection.recv(1, socket.MSG_PEEK):
                        active = None
                    else:
                        value = receive(connection, root, allow_close=True)
                        if value is None:
                            active = None
                        else:
                            record(root, "unexpected_request", value=value)
                if (root / "stop").exists() and not readable:
                    break
    record(root, "finished")


if __name__ == "__main__":
    main()
