"""Invalid launch settings are rejected without creating a session."""
import json
import unittest

from tests._support import load_config, start_server, write_config


class InvalidLaunchConfigTest(unittest.TestCase):
    def test_invalid_launch_config_is_rejected_before_session_creation(self):
        server = start_server(self, "invalid-launch-config")

        # Valid TOML, but a required launch field is empty.
        configuration = load_config()
        configuration["launch"]["package"] = ""
        config = server.artifacts / "invalid-launch.toml"
        write_config(config, configuration)
        self.assertEqual(load_config(config)["launch"]["package"], "")

        result = server.run_cli("session", "create", "--config", str(config))
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertIn("invalid_config", json.loads(result.stderr)["error"])
        self.assertEqual(server.cli("session", "ls"), [])

        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
