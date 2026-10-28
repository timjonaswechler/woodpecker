"""Only the addressed session applies queued keyboard and pointer input to its world."""
import os
import time
import unittest

from tests._support import ROOT, WINDOW_POSITIONS, start_server, load_config, write_config


class InputIsolationTest(unittest.TestCase):
    def _pointer_state(self, server, session_id):
        """Pure public query; discover pointers without copying handles between worlds."""
        location_type = "bevy_picking::pointer::PointerLocation"
        buttons_type = "bevy_picking::pointer::PointerPressState"
        output = server.call(session_id, {
            "command": "inspect.query", "arguments": {
                "source": "entities", "entity": None,
                "with": [location_type, buttons_type], "without": [],
                "projection": {"kind": "components", "selection": {
                    "kind": "listed", "type_paths": [location_type, buttons_type],
                }},
            },
        })
        pointers = []
        for item in output["items"]:
            self.assertEqual(item["kind"], "entity", item)
            self.assertEqual(item["result"]["kind"], "components", item)
            components = item["result"]["components"]
            self.assertEqual(len(components), 2, item)
            values = {}
            for component in components:
                self.assertEqual(component["value"]["status"], "readable", component)
                values[component["component"]["type_path"]] = component["value"]["value"]
            pointers.append({"entity": item["entity"],
                             "location": values[location_type]["location"],
                             "buttons": values[buttons_type]})
        self.assertTrue(pointers, output)
        return pointers

    def test_keyboard_pointer_and_player_state_do_not_leak_into_the_other_session(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens two staggered Signalstation windows. Each permits up to "
            "3600 engine-startup ticks in 60 seconds and 120 explicit readiness ticks "
            "in ten seconds, then two action ticks. Only A receives arrow_right, "
            "pointer move_to [720,450] and middle press/release. No OS input. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "input-isolation")
        sessions = []
        ticks_type = "test_app::woodpecker::SimulationTicks"
        player_type = "test_app::player::Player"
        keys_type = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
        mouse_type = "bevy_input::button_input::ButtonInput<bevy_input::mouse::MouseButton>"
        for label, (x, y) in zip(("A", "B"), WINDOW_POSITIONS):
            config = server.artifacts / f"session-{label}.toml"
            configuration = load_config()
            configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
            configuration["launch"]["arguments"][:0] = ["--window-position", f"{x},{y}"]
            write_config(config, configuration)
            session_id = server.cli("session", "create", "--config", str(config))["id"]
            sessions.append(session_id)
            deadline = time.monotonic() + 90
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "EngineReady":
                    break
                self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
                if time.monotonic() >= deadline:
                    self.fail(f"{label} engine did not become ready; see {server.artifacts}")
                time.sleep(0.05)
            startup_ticks = server.read_resource(session_id, ticks_type)["ticks"]
            # Read game readiness separately. Every extra tick is explicit here.
            deadline = time.monotonic() + 10
            for additional_ticks in range(121):
                load = server.read_resource(session_id, "test_app::level::LevelLoad")
                if load == "Ready":
                    break
                self.assertEqual(load, "Loading", load)
                if additional_ticks == 120 or time.monotonic() >= deadline:
                    self.fail(f"{label} game not ready within budget; see {server.artifacts}")
                self.assertEqual(server.call(session_id, {
                    "command": "tick.warp.start", "arguments": {"ticks": 1},
                }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
            server.record(["preparation", "game-ready"], session_id=session_id, label=label,
                          startup_ticks=startup_ticks, additional_ticks=additional_ticks)

        session_a, session_b = sessions
        self.assertNotEqual(session_a, session_b)
        initial = {}
        for session_id in sessions:
            initial[session_id] = {
                "position": server.read_resource(session_id, player_type)["position"],
                "ticks": server.read_resource(session_id, ticks_type)["ticks"],
                "keys": server.read_resource(session_id, keys_type),
                "mouse": server.read_resource(session_id, mouse_type),
                "pointers": self._pointer_state(server, session_id),
            }
            self.assertEqual(initial[session_id]["keys"]["pressed"], [])
            self.assertEqual(initial[session_id]["mouse"]["pressed"], [])
            self.assertTrue(all(pointer["location"] is None for pointer in initial[session_id]["pointers"]))

        self.assertIsNone(server.call(session_a, {
            "command": "input.keyboard.press", "arguments": {"key": "arrow_right"},
        }))
        self.assertIsNone(server.call(session_a, {
            "command": "input.pointer.move_to", "arguments": {"position": [720.0, 450.0]},
        }))
        self.assertIsNone(server.call(session_a, {
            "command": "input.pointer.press", "arguments": {"button": "middle"},
        }))
        # Tick B first, while A's input is still queued. A shared queue would be
        # consumed by the wrong world here; a shared pressed state also leaks.
        self.assertEqual(server.call(session_b, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertEqual(server.read_resource(session_b, player_type)["position"], initial[session_b]["position"])
        self.assertEqual(server.read_resource(session_b, keys_type), initial[session_b]["keys"])
        self.assertEqual(server.read_resource(session_b, mouse_type), initial[session_b]["mouse"])
        self.assertEqual(self._pointer_state(server, session_b), initial[session_b]["pointers"])
        self.assertEqual(server.read_resource(session_a, player_type)["position"], initial[session_a]["position"])
        self.assertEqual(server.read_resource(session_a, ticks_type)["ticks"], initial[session_a]["ticks"])

        self.assertEqual(server.call(session_a, {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        moved_a = server.read_resource(session_a, player_type)["position"]
        self.assertGreater(moved_a[0], initial[session_a]["position"][0])
        self.assertAlmostEqual(moved_a[1], initial[session_a]["position"][1], delta=1e-6)
        self.assertEqual(server.read_resource(session_a, keys_type)["pressed"], ["ArrowRight"])
        self.assertEqual(server.read_resource(session_a, mouse_type)["pressed"], ["Middle"])
        pointers_a = self._pointer_state(server, session_a)
        active = [pointer for pointer in pointers_a if pointer["buttons"]["middle"]]
        self.assertEqual(len(active), 1, pointers_a)
        active = active[0]
        self.assertEqual(active["location"]["position"], [720.0, 450.0])

        self.assertIsNone(server.call(session_a, {
            "command": "input.keyboard.release", "arguments": {"key": "arrow_right"},
        }))
        self.assertIsNone(server.call(session_a, {
            "command": "input.pointer.release", "arguments": {"button": "middle"},
        }))
        for session_id in (session_b, session_a):
            self.assertEqual(server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        self.assertEqual(server.read_resource(session_a, keys_type)["pressed"], [])
        self.assertEqual(server.read_resource(session_a, mouse_type)["pressed"], [])
        released = [pointer for pointer in self._pointer_state(server, session_a)
                    if pointer["entity"] == active["entity"]]
        self.assertEqual(len(released), 1)
        self.assertFalse(released[0]["buttons"]["middle"])
        self.assertEqual(released[0]["location"]["position"], [720.0, 450.0])
        self.assertEqual(server.read_resource(session_b, player_type)["position"], initial[session_b]["position"])
        self.assertEqual(server.read_resource(session_b, keys_type), initial[session_b]["keys"])
        self.assertEqual(server.read_resource(session_b, mouse_type), initial[session_b]["mouse"])
        self.assertEqual(self._pointer_state(server, session_b), initial[session_b]["pointers"])
        for session_id in sessions:
            self.assertEqual(server.read_resource(session_id, ticks_type)["ticks"] - initial[session_id]["ticks"], 2)
        server.record(["observation", "input-isolation"], session_a=session_a, session_b=session_b,
                      initial_a=initial[session_a], initial_b=initial[session_b],
                      moved_a=moved_a, applied_pointer=active, released_pointer=released[0])
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
