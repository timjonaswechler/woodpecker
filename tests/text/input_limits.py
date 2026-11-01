"""UTF-8 transport limits are separate from focus requirements and widget length rules."""
import json
import os
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.text import BUTTON, FOCUS, SETTINGS, TEXT_INPUT, TICKS, layout_target


class TextInputLimitsTest(unittest.TestCase):
    def test_focus_empty_text_utf8_byte_boundary_and_widget_limit_are_distinct(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One window, up to 3600 engine ticks/60 s and 120 readiness ticks/10 s; "
                         "18 action ticks. Pointer-focus notes/name through real UI. Empty text, "
                         "16384 UTF-8 bytes, 16385-byte rejection, and widget-only 25-character "
                         "rejection. Three expected SDK rejections; 4 MiB Activity, no OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "text-input-limits", activity_bytes=4 * 1024 * 1024)
        config = server.artifacts / "engine.toml"
        configuration = load_config()
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
        deadline = time.monotonic() + 90
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "EngineReady":
                break
            self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
            if time.monotonic() >= deadline:
                self.fail("EngineReady deadline exceeded")
            time.sleep(0.05)
        startup_ticks = server.read_resource(session_id, TICKS)["ticks"]
        deadline = time.monotonic() + 10
        for additional in range(121):
            load = server.read_resource(session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            self.assertEqual(load, "Loading")
            if additional == 120 or time.monotonic() >= deadline:
                self.fail("Game readiness budget exceeded")
            self.assertEqual(server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        server.record(["preparation", "game-ready"], startup_ticks=startup_ticks, additional_ticks=additional)
        seed = server.read_resource(session_id, SETTINGS)
        self.assertEqual(seed["notes"], "")
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        # Bevy may focus the window itself at startup. That is not an editable
        # text target; do not confuse non-None InputFocus with valid text focus.
        non_editable_focus = server.read_resource(session_id, FOCUS)
        rejected = []
        for text in ("", "ohne Fokus"):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "input.text.input", "arguments": {"text": text},
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], "text_focus_unavailable")
            rejected.append(outcome)
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        self.assertEqual(server.read_resource(session_id, FOCUS), non_editable_focus)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks)
        for marker, selector in ((BUTTON, {"label": "Einstellungen [M]"}),
                                 (TEXT_INPUT, {"name": "Notizen"})):
            target = layout_target(server, session_id, marker, **selector)
            for command, arguments in (
                ("input.pointer.move_to", {"position": target["position"]}),
                ("input.pointer.press", {"button": "left"}),
                ("input.pointer.release", {"button": "left"}),
            ):
                self.assertIsNone(server.call(session_id, {"command": command, "arguments": arguments}))
                server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertIsNotNone(server.read_resource(session_id, FOCUS)["current_focus"])
        # Empty is valid with focus; after editing/synchronization it remains a no-op.
        self.assertIsNone(server.call(session_id, {
            "command": "input.text.input", "arguments": {"text": ""},
        }))
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 2}})
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        payload = "é" * 8192
        self.assertEqual(len(payload), 8192)
        self.assertEqual(len(payload.encode("utf-8")), 16384)
        self.assertIsNone(server.call(session_id, {
            "command": "input.text.input", "arguments": {"text": payload},
        }))
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 8)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 2}})
        accepted = {**seed, "notes": payload}
        self.assertEqual(server.read_resource(session_id, SETTINGS), accepted)
        too_large = payload + "a"
        self.assertEqual(len(too_large), 8193)
        self.assertEqual(len(too_large.encode("utf-8")), 16385)
        focus = server.read_resource(session_id, FOCUS)
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "input.text.input", "arguments": {"text": too_large},
        }))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "rejected", outcome)
        self.assertEqual(outcome["error"]["code"], "text_too_large")
        rejected.append(outcome)
        self.assertEqual(server.read_resource(session_id, SETTINGS), accepted)
        self.assertEqual(server.read_resource(session_id, FOCUS), focus)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 10)
        # Resume exposes the existing name widget, whose independent limit is 24.
        for marker, selector in ((BUTTON, {"label": "Zurück zum Spiel"}),
                                 (TEXT_INPUT, {"name": "Spielername"})):
            target = layout_target(server, session_id, marker, **selector)
            for command, arguments in (
                ("input.pointer.move_to", {"position": target["position"]}),
                ("input.pointer.press", {"button": "left"}),
                ("input.pointer.release", {"button": "left"}),
            ):
                self.assertIsNone(server.call(session_id, {"command": command, "arguments": arguments}))
                server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        # Bevy rejects the whole insertion, not a truncated 24-character prefix.
        # SDK acceptance must not be confused with this widget-level decision.
        self.assertIsNone(server.call(session_id, {
            "command": "input.text.input", "arguments": {"text": "z" * 25},
        }))
        self.assertEqual(server.read_resource(session_id, SETTINGS), accepted)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 2}})
        self.assertEqual(server.read_resource(session_id, SETTINGS), accepted)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 18)
        self.assertEqual(len(rejected), 3)
        server.record(["observation", "text-limits"], accepted_characters=8192, accepted_bytes=16384,
                      rejected_characters=8193, rejected_bytes=16385, rejections=rejected,
                      widget_input_bytes=25, widget_unchanged=True, action_ticks=18)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
