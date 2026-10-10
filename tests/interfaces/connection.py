"""An independent observer and deliberately nonreading WebSocket client."""
import base64
import hashlib
import json
import os
import socket
import struct


class Connection:
    def __init__(self, session):
        self.session = session
        self.socket = None
        self.call_id = 0

    def __enter__(self):
        host, port = self.session.server.address.rsplit(":", 1)
        self.socket = socket.socket()
        try:
            self.socket.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 4096)
            self.socket.settimeout(20)
            self.socket.connect((host, int(port)))
            key = base64.b64encode(os.urandom(16)).decode()
            self.socket.sendall((f"GET /v1/sessions/{self.session.id}/connect HTTP/1.1\r\n"
                f"Host: {self.session.server.address}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n"
                f"Sec-WebSocket-Version: 13\r\nSec-WebSocket-Key: {key}\r\n\r\n").encode())
            header = bytearray()
            while not header.endswith(b"\r\n\r\n"):
                header.extend(self.read(1))
                assert len(header) < 16384, "WebSocket-Handshake zu groß"
            lines = header.decode().split("\r\n")
            assert lines[0].split()[1] == "101", lines
            fields = dict(line.lower().split(": ", 1) for line in lines[1:] if ": " in line)
            expected = base64.b64encode(hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()).decode()
            # Header names are case insensitive; their values are not.
            accept = next(line.split(":", 1)[1].strip() for line in lines if line.lower().startswith("sec-websocket-accept:"))
            assert accept == expected and fields["upgrade"] == "websocket", lines
            return self
        except BaseException:
            self.socket.close()
            raise

    def read(self, size):
        result = bytearray()
        while len(result) < size:
            part = self.socket.recv(size - len(result))
            if not part:
                raise AssertionError("WebSocket vor der vollständigen Antwort geschlossen")
            result.extend(part)
        return bytes(result)

    def frame(self, opcode, payload):
        size = len(payload)
        prefix = bytes([0x80 | opcode])
        if size < 126:
            prefix += bytes([0x80 | size])
        elif size < 65536:
            prefix += bytes([0x80 | 126]) + struct.pack("!H", size)
        else:
            prefix += bytes([0x80 | 127]) + struct.pack("!Q", size)
        mask = os.urandom(4)
        self.socket.sendall(prefix + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(payload)))

    def send(self, operation):
        self.call_id += 1
        self.frame(1, json.dumps({"version": 1, "call": self.call_id, "operation": operation}).encode())

    def receive(self):
        payload = bytearray()
        while True:
            first, second = self.read(2)
            assert not second & 0x80, "Serverantwort darf nicht maskiert sein"
            size = second & 0x7f
            if size == 126:
                size = struct.unpack("!H", self.read(2))[0]
            elif size == 127:
                size = struct.unpack("!Q", self.read(8))[0]
            body = self.read(size)
            opcode = first & 0x0f
            if opcode == 9:
                self.frame(10, body)
                continue
            assert opcode in (0, 1), f"Unerwarteter WebSocket-Opcode {opcode}"
            payload.extend(body)
            if first & 0x80:
                reply = json.loads(payload)
                assert reply["version"] == 1 and reply["call"] == self.call_id, reply
                return reply["result"]

    def call(self, operation):
        self.send(operation)
        return self.receive()

    def __exit__(self, *_):
        self.socket.close()


def snapshot(session):
    with Connection(session) as connection:
        reply = connection.call({"kind": "snapshot"})
        assert reply["kind"] == "snapshot", reply
        return reply["snapshot"]
