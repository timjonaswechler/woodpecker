"""Invalid launch settings are rejected without creating a session."""
import json
import unittest

from tests._support import load_config, start_server, write_app_config


class InvalidLaunchConfigTest(unittest.TestCase):
    def test_invalid_launch_config_is_rejected_before_session_creation(self):
        server = start_server(self, "invalid-launch-config")

        # Required Cargo settings and an explicitly supplied executable cannot be empty.
        for field in ("package", "executable"):
            configuration = load_config()
            configuration["launch"][field] = ""
            config = server.artifacts / f"invalid-{field}.toml"
            write_app_config(config, configuration)
            self.assertEqual(load_config(config)["launch"][field], "")

            result = server.run_cli("session", "create", "--config", str(config))
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertEqual(result.stdout, "")
            self.assertIn("invalid_config", json.loads(result.stderr)["error"])
            self.assertEqual(server.cli("session", "ls"), [])

        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
