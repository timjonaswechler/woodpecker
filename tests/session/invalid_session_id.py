"""Malformed session IDs are rejected before looking up a session."""
import json
import unittest

from tests._support import start_server


class InvalidSessionIdTest(unittest.TestCase):
    def test_invalid_session_ids_are_rejected_without_stopping_server(self):
        server = start_server(self, "invalid-session-id")

        # Too short, too long, and the right length with non-hex characters.
        for invalid_id in ("0" * 7, "0" * 33, "g" * 32):
            with self.subTest(session_id=invalid_id):
                result = server.run_cli("session", "inspect", invalid_id)
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual(result.stdout, "")
                self.assertIn("invalid_session_id", json.loads(result.stderr)["error"])

        self.assertIsNone(server.process.poll(), "Invalid input stopped the server")
        self.assertEqual(server.cli("session", "ls"), [])
        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
