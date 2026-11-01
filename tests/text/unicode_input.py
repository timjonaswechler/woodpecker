"""Unicode reaches a pointer-focused real notes widget without keyboard synthesis."""
import os
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.text import BUTTON, FOCUS, KEYS, SETTINGS, TEXT_INPUT, TICKS, layout_target


class UnicodeInputTest(unittest.TestCase):
    def test_unicode_and_newlines_reach_pointer_focused_notes_only_after_ticks(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One window, up to 3600 engine ticks/60 s and 120 readiness ticks/10 s; "
                         "eight action ticks: click settings, click notes, Unicode text and "
                         "two explicit edit/sync ticks. No keyboard commands or OS input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "text-unicode")
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
        empty_keys = {"pressed": [], "just_pressed": [], "just_released": []}
        self.assertEqual(server.read_resource(session_id, KEYS), empty_keys)
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        targets = []
        # These are the existing settings button and unbounded multiline notes field.
        for marker, selector in ((BUTTON, {"label": "Einstellungen [M]"}),
                                 (TEXT_INPUT, {"name": "Notizen"})):
            target = layout_target(server, session_id, marker, **selector)
            targets.append(target)
            for command, arguments in (
                ("input.pointer.move_to", {"position": target["position"]}),
                ("input.pointer.press", {"button": "left"}),
                ("input.pointer.release", {"button": "left"}),
            ):
                self.assertIsNone(server.call(session_id, {"command": command, "arguments": arguments}))
                self.assertEqual(server.call(session_id, {
                    "command": "tick.warp.start", "arguments": {"ticks": 1},
                }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
                self.assertEqual(server.read_resource(session_id, KEYS), empty_keys)
        focus = server.read_resource(session_id, FOCUS)
        self.assertIsNotNone(focus["current_focus"])
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        text = "Grüße 🦉\nCafe\u0301 – 東京"
        self.assertIsNone(server.call(session_id, {
            "command": "input.text.input", "arguments": {"text": text},
        }))
        self.assertEqual(server.read_resource(session_id, SETTINGS), seed)
        self.assertEqual(server.read_resource(session_id, FOCUS), focus)
        self.assertEqual(server.read_resource(session_id, KEYS), empty_keys)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 6)
        # Bevy edits in PostUpdate; the app mirrors the field into Settings in Update.
        # Both ticks are fixed preparation, not a retry-until-content-matches loop.
        for _ in range(2):
            server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            self.assertEqual(server.read_resource(session_id, KEYS), empty_keys)
        expected = {**seed, "notes": text}
        self.assertEqual(server.read_resource(session_id, SETTINGS), expected)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 8)
        server.record(["observation", "unicode-input"], targets=targets, text=text, action_ticks=8,
                      utf8_bytes=len(text.encode("utf-8")), physical_keyboard=empty_keys)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
