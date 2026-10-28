"""Malformed Command-JSON must fail before binding, sending or allocating an ID."""
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import unittest

from tests._support import CLI, ROOT


class CommandErrorsTest(unittest.TestCase):
    def test_invalid_json_utf8_and_command_shapes_do_not_send_even_a_valid_prefix(self):
        self.assertTrue(CLI.is_file(), "Build the CLI first")
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        artifacts = Path(tempfile.mkdtemp(prefix="cli-command-errors-", dir=runs))
        self.artifact_dirs = [str(artifacts)]
        print(f"\nArtefakte: {artifacts}", flush=True)
        # No server/session exists. Any bind/send would hit this owned listener.
        # A valid prefix is deliberately a shutdown, not something to execute.
        listener = self.enterContext(socket.socket())
        listener.bind(("127.0.0.1", 0))
        listener.listen()
        listener.setblocking(False)
        address = f"127.0.0.1:{listener.getsockname()[1]}"
        prefix = '{"command":"shutdown","arguments":{}}'
        cases = [
            ("empty", "", "invalid_json"),
            ("truncated", '{"command":', "invalid_json"),
            ("single-quotes", "{'command':'shutdown','arguments':{}}", "invalid_json"),
            ("second-document", prefix + prefix, "invalid_json"),
            ("bad-tail", prefix + "\nnot-json", "invalid_json"),
            ("lone-surrogate", '{"command":"input.text.input","arguments":{"text":"\\uD800"}}',
             "invalid_json"),
        ]
        for label, command in (
            ("null", None), ("array", []), ("empty-object", {}),
            ("unknown-command", {"command": "invented.command", "arguments": {}}),
            ("missing-arguments", {"command": "shutdown"}),
            ("missing-command", {"arguments": {}}),
            ("outer-field", {"command": "shutdown", "arguments": {}, "request_id": 7}),
            ("bad-argument-type", {"command": "tick.warp.start", "arguments": {"ticks": "one"}}),
            ("unknown-argument", {"command": "tick.warp.start", "arguments": {"ticks": 1, "extra": True}}),
        ):
            cases.append((label, json.dumps(command), "invalid_command"))
        # Real non-UTF-8 argv bytes, not merely an escaped Unicode surrogate.
        cases.append(("raw-invalid-utf8", b'{"command":"input.text.input","arguments":{"text":"\xff"}}',
                      "invalid_json"))

        for label, source, code in cases:
            command = [str(CLI), "--address", address, "session", "submit", "0" * 32,
                       "--command", source]
            result = subprocess.run(command, cwd=ROOT, capture_output=True, timeout=10)
            (artifacts / f"{label}.stdout").write_bytes(result.stdout)
            (artifacts / f"{label}.stderr").write_bytes(result.stderr)
            (artifacts / f"{label}.exit").write_text(str(result.returncode) + "\n")
            (artifacts / f"{label}.input").write_bytes(
                source if isinstance(source, bytes) else source.encode("utf-8"))
            try:
                connection, _ = listener.accept()
            except BlockingIOError:
                pass
            else:
                connection.close()
                self.fail(f"{label}: invalid input attempted to bind/send")
            self.assertEqual(result.returncode, 1, label)
            self.assertEqual(result.stdout, b"", (label, "No Pending/request ID may be printed"))
            try:
                error = json.loads(result.stderr)
            except (json.JSONDecodeError, UnicodeDecodeError):
                self.fail(f"{label}: expected machine-readable {code}, got exit "
                          f"{result.returncode} with stderr {result.stderr!r}")
            self.assertEqual(set(error), {"error"}, label)
            self.assertIsInstance(error["error"], str, label)
            self.assertTrue(error["error"].startswith(code + ":"), (label, error))
