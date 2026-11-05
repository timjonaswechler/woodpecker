"""Own logical device loss after a real app capture terminates through RenderFailed."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import load_config, owned_process, start_server, write_app_config, wait_process_gone
from tests.headless._windows import owned_windows


class HeadlessTerminalTest(unittest.TestCase):
    def test_real_app_device_loss_is_typed_and_its_owned_process_is_reaped(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1")
        server = start_server(self, "headless-terminal", expected_exit_code=1)
        configuration = load_config()
        configuration["launch"].update(headless={"width": 1440, "height": 900, "scale_factor": 1.0}, no_default_features=True)
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe", "--woodpecker-render-loss"]
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
        path = server.artifacts / "headless.toml"
        write_app_config(path, configuration)
        session_id = server.cli("session", "create", "--config", str(path))["id"]
        deadline = time.monotonic() + 90
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "EngineReady":
                break
            self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
            if time.monotonic() >= deadline:
                self.fail("Image EngineReady deadline")
            time.sleep(0.05)
        root = Path(detail["artifact_dir"])
        identity = owned_process(server, root)
        self.assertEqual(owned_windows(identity["pid"]), [])
        server.call(session_id, {"command": "screenshot.capture", "arguments": {"path": "healthy.png"}}, timeout=35)
        self.assertTrue((root / "healthy.png").is_file())
        cursor = server.cli("session", "poll", session_id)["cursor"]
        server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f10"}})
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {"ticks": 1}}))
        deadline = time.monotonic() + 10
        events = []
        while True:
            activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor), "--wait-ms", "100")
            cursor = activity["cursor"]
            events += [row["event"] for row in activity["entries"]]
            ended = [row["event"] for row in events if row["kind"] == "event" and row["event"]["kind"] == "ended"]
            if ended:
                break
            if time.monotonic() >= deadline:
                self.fail("Own device loss did not produce a terminal event")
        self.assertEqual(len(ended), 1)
        self.assertEqual(ended[0]["reason"]["kind"], "render_failed", ended)
        self.assertIn(ended[0]["reason"]["error"]["code"], ("render_device_lost", "render_panicked"))
        server.record(["observation", "typed-app-device-loss"], terminal=ended[0], warp=pending, action_ticks=1)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 1)
        wait_process_gone(identity)
