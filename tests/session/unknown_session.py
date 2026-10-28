"""An unknown session is rejected without taking down the server."""
import json
import unittest

from tests._support import start_server


class UnknownSessionTest(unittest.TestCase):
    def test_unknown_session_is_rejected_without_stopping_server(self):
        server = start_server(self, "unknown-session")

        # Valid ID syntax, but no sessions exist on this fresh server.
        result = server.run_cli("session", "inspect", "0" * 32)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("session_not_found", json.loads(result.stderr)["error"])

        self.assertIsNone(server.process.poll(), "The lookup error stopped the server")
        self.assertEqual(server.cli("session", "ls"), [])
        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
