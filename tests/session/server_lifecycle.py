"""An empty server can start, answer the public CLI and stop."""
import unittest

from tests._support import start_server


class ServerLifecycleTest(unittest.TestCase):
    def test_server_starts_lists_sessions_and_stops(self):
        server = start_server(self, "server-lifecycle")
        self.assertEqual(server.ready["state"], "Ready")
        host, port = server.address.rsplit(":", 1)
        self.assertEqual(host, "127.0.0.1")
        self.assertGreater(int(port), 0)

        self.assertEqual(server.cli("session", "ls"), [])
        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
