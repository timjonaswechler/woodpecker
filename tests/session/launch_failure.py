"""A failed application launch remains inspectable without stopping the server."""
import time
import unittest

from tests._support import load_config, start_server, write_config


class LaunchFailureTest(unittest.TestCase):
    def test_launch_failure_keeps_session_diagnostics_and_server_available(self):
        # A retained Failed session makes the final server result unsuccessful.
        server = start_server(self, "launch-failure", expected_exit_code=1)
        missing_manifest = server.artifacts / "missing-project/Cargo.toml"
        self.assertFalse(missing_manifest.exists())
        configuration = load_config()
        configuration["launch"]["manifest_path"] = str(missing_manifest)
        config = server.artifacts / "missing-manifest.toml"
        write_config(config, configuration)
        self.assertEqual(load_config(config)["launch"]["manifest_path"], str(missing_manifest))

        # Intrinsically valid configuration is accepted before file resolution.
        created = server.cli("session", "create", "--config", str(config))
        session_id = created["id"]
        self.assertEqual(created["state"], "Starting")
        deadline = time.monotonic() + 10
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertEqual(detail["id"], session_id)
            if detail["state"] == "Failed":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Missing manifest did not result in Failed; see {server.artifacts}")
            time.sleep(0.02)

        self.assertEqual(detail["error"]["code"], "invalid_config", detail)
        self.assertIn("manifest", detail["error"]["message"])
        self.assertIsNone(server.process.poll(), "A launch failure stopped the server")
        self.assertEqual(server.cli("session", "inspect", session_id), detail)
        sessions = server.cli("session", "ls")
        self.assertEqual(
            [(item["id"], item["state"]) for item in sessions],
            [(session_id, "Failed")],
        )

        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 1)
        self.assertIn(
            "shutdown_incomplete",
            (server.artifacts / "server.stderr.log").read_text(encoding="utf-8"),
        )
