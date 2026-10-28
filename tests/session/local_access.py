"""Reject remote binding and browser Origin requests while ordinary local CLI access works."""
import http.client
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import unittest

from tests._support import CLI, ROOT, start_server


class LocalAccessTest(unittest.TestCase):
    def test_remote_bind_and_browser_origin_are_rejected_without_disabling_local_access(self):
        self.assertTrue(CLI.is_file(), "Build the CLI first")
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        guard = Path(tempfile.mkdtemp(prefix="local-access-bind-", dir=runs))
        self.artifact_dirs = [str(guard)]
        # Keep this port exclusively occupied on loopback. Even a validation
        # regression cannot expose a wildcard listener on the reserved port.
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as reservation:
            reservation.bind(("127.0.0.1", 0))
            reservation.listen(1)
            command = [str(CLI), "--address", f"0.0.0.0:{reservation.getsockname()[1]}",
                       "server", "start", "--artifact-dir", str(guard / "sessions")]
            rejected = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=5)
        (guard / "bind.json").write_text(json.dumps({
            "command": command, "returncode": rejected.returncode,
            "stdout": rejected.stdout, "stderr": rejected.stderr,
        }, indent=2) + "\n")
        self.assertEqual(rejected.returncode, 1, rejected.stderr)
        self.assertIn("invalid_config", rejected.stderr)
        self.assertEqual(rejected.stdout, "")
        self.assertFalse((guard / "sessions").exists(), "Invalid bind must fail before artifact creation")

        server = start_server(self, "local-access")
        self.assertEqual(server.cli("session", "ls"), [])
        host, port = server.address.rsplit(":", 1)
        for origin in ("https://example.invalid", "null", ""):
            for method, path in (("GET", "/v1/sessions"), ("POST", "/v1/stop")):
                connection = http.client.HTTPConnection(host, int(port), timeout=5)
                try:
                    connection.request(method, path, headers={"Origin": origin})
                    response = connection.getresponse()
                    body = json.loads(response.read())
                finally:
                    connection.close()
                server.record(["http", method, path], origin=origin, status=response.status, body=body)
                self.assertEqual(response.status, 403)
                self.assertEqual(body["version"], 1)
                self.assertEqual(body["error"]["code"], "unsupported_origin")
                self.assertNotIn("data", body)
            # In particular, browser-origin POST /stop did not stop the server.
            self.assertIsNone(server.process.poll())
            self.assertEqual(server.cli("session", "ls"), [])
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
