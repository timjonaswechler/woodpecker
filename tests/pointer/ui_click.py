"""Absolute virtual positioning activates an actual laid-out UI button only on ticks."""
import os
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.pointer import MOUSE, TICKS, layout_target, pointer_states, read_components


class UiClickTest(unittest.TestCase):
    def test_absolute_pointer_picks_the_real_map_button_without_early_effects(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One window, up to 3600 engine ticks/60 s and 120 readiness ticks/10 s, "
                         "then three ticks: move_to, left press, left release on the map button. "
                         "No OS cursor input. Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "pointer-ui-click")
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
        view_type = "test_app::player::camera::ViewState"
        button_type = "bevy_ui_widgets::button::Button"
        pressed_type = "bevy_ui::interaction_states::Pressed"
        target = layout_target(server, session_id, button_type, label="Gesamtkarte [G]")
        center = target["position"]
        before_ticks = server.read_resource(session_id, TICKS)["ticks"]
        before_pointers = pointer_states(server, session_id)
        self.assertTrue(before_pointers)
        self.assertTrue(all(item["location"] is None for item in before_pointers))
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.move_to", "arguments": {"position": center},
        }))
        self.assertEqual(pointer_states(server, session_id), before_pointers)
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        active, = [item for item in pointer_states(server, session_id) if item["location"] is not None]
        self.assertEqual(active["location"]["position"], center)
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.press", "arguments": {"button": "left"},
        }))
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], [])
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Overview")
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"], before_ticks + 1)
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Map")
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], ["Left"])
        pressed = read_components(server, session_id, [button_type, pressed_type], [])
        self.assertEqual([item["entity"] for item in pressed], [target["entity"]])
        self.assertIsNone(server.call(session_id, {
            "command": "input.pointer.release", "arguments": {"button": "left"},
        }))
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], ["Left"])
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertEqual(server.read_resource(session_id, MOUSE)["pressed"], [])
        self.assertEqual(server.read_resource(session_id, view_type)["mode"], "Map")
        pressed = read_components(server, session_id, [button_type, pressed_type], [])
        self.assertEqual(pressed, [])
        final, = [item for item in pointer_states(server, session_id) if item["entity"] == active["entity"]]
        self.assertEqual(final["location"]["position"], center)
        self.assertFalse(final["buttons"]["primary"])
        self.assertEqual(server.read_resource(session_id, TICKS)["ticks"] - before_ticks, 3)
        server.record(["observation", "ui-click"], target=target, position=center, action_ticks=3,
                      mode="Map", pointer=final)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
