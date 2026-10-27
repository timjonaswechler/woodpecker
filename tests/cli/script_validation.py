"""The whole script is checked before its apparently executable prefix is submitted."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class ScriptValidationTest(unittest.TestCase):
    def _state(self, server, session_id):
        return {name: server.read_resource(session_id, path) for name, path in (
            ("ticks", "test_app::woodpecker::SimulationTicks"),
            ("time", "bevy_time::time::Time<()>"),
            ("keys", "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"),
            ("view", "test_app::player::camera::ViewState"),
            ("settings", "test_app::app::settings::Settings"),
        )}

    def test_bad_tail_version_and_shutdown_position_reject_the_entire_script(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual-startup window, zero ticks. Nine invalid scripts with a "
                         "g-press/one-tick prefix must submit nothing; empty valid script and final "
                         "g press/release without ticks check recovery. No OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "cli-script-validation")
        session_id = server.cli("session", "create", "--config", str(ROOT / "tests/app.toml"))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail("SessionReady deadline exceeded")
            time.sleep(0.05)
        before = self._state(server, session_id)
        self.assertEqual(before["ticks"], {"ticks": 0})
        prefix = [
            {"command": "input.keyboard.press", "arguments": {"key": "g"}},
            {"command": "tick.warp.start", "arguments": {"ticks": 1}},
        ]
        shutdown = {"command": "shutdown", "arguments": {}}
        valid = {"version": 1, "commands": prefix}
        encode = lambda commands: json.dumps({"version": 1, "commands": commands})
        variants = [
            ("broken-end", json.dumps(valid)[:-2] + ',{"command":', None),
            ("trailing-data", json.dumps(valid) + "\nnot-json", None),
            ("bad-tail-command", encode([*prefix, {"command": "invented.command", "arguments": {}}]), 2),
            ("bad-tail-arguments", encode([*prefix, {"command": "tick.warp.start",
                                                   "arguments": {"ticks": "one"}}]), 2),
            ("shutdown-not-last", encode([prefix[0], shutdown, prefix[1]]), 1),
            ("two-shutdowns", encode([*prefix, shutdown, shutdown]), 2),
            ("future-version", json.dumps({**valid, "version": 2}), None),
            ("unknown-field", json.dumps({**valid, "unexpected": True}), None),
            ("duplicate-version", '{"version":1,"version":1,"commands":' + json.dumps(prefix) + "}", None),
        ]
        observations = []
        for label, source, index in variants:
            path = server.artifacts / f"{label}.json"
            path.write_text(source, encoding="utf-8")
            cursor = server.cli("session", "poll", session_id)["cursor"]
            result = server.run_cli("session", "script", session_id, "--file", str(path))
            self.assertEqual(result.returncode, 1, (label, result.stdout, result.stderr))
            error = json.loads(result.stdout)
            self.assertEqual(set(error), {"kind", "command_index", "message"}, label)
            self.assertEqual(error["kind"], "invalid_script", label)
            self.assertEqual(error["command_index"], index, label)
            self.assertIsInstance(error["message"], str, label)
            self.assertTrue(error["message"].strip(), label)
            self.assertIn("invalid_script", json.loads(result.stderr)["error"], label)
            activity = server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))
            self.assertEqual(activity["kind"], "activity", label)
            self.assertEqual(activity["entries"], [], "No prefix or shutdown may be submitted")
            self.assertEqual(self._state(server, session_id), before, label)
            self.assertEqual(path.read_text(encoding="utf-8"), source, label)
            observations.append({"case": label, "command_index": index})
        empty = server.artifacts / "empty.json"
        empty.write_text('{"version":1,"commands":[]}\n')
        cursor = server.cli("session", "poll", session_id)["cursor"]
        result = server.run_cli("session", "script", session_id, "--file", str(empty))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr, "")
        self.assertEqual(json.loads(result.stdout), {"kind": "passed", "completed": []})
        self.assertEqual(server.cli("session", "poll", session_id,
                                    "--cursor", json.dumps(cursor))["entries"], [])
        self.assertEqual(self._state(server, session_id), before)
        # If a rejected prefix leaked into the logical queue, this fresh press fails.
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.press", "arguments": {"key": "g"}}))
        self.assertIsNone(server.call(session_id, {
            "command": "input.keyboard.release", "arguments": {"key": "g"}}))
        self.assertEqual(self._state(server, session_id), before)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
        (server.artifacts / "observations.json").write_text(json.dumps(observations, indent=2) + "\n")
