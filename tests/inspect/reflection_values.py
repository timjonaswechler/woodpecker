"""Reflection values preserve maps, ordered sets, enum shapes, handles and explicit unavailability."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class ReflectionValuesTest(unittest.TestCase):
    def _value(self, server, session_id, path):
        output = server.call(session_id, {"command": "inspect.query", "arguments": {
            "source": "resources", "selector": {"kind": "type", "type_path": path},
            "projection": {"kind": "value"},
        }})
        self.assertEqual(len(output["items"]), 1, output)
        item = output["items"][0]
        self.assertEqual(item["kind"], "resource")
        self.assertEqual(item["result"]["kind"], "value")
        self.assertEqual(item["result"]["type_path"], path)
        return item["result"]["value"]

    def test_reflection_shapes_and_unavailable_reasons_are_lossless_and_explicit(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One reflection-opt-in Signalstation window, manual startup, no ticks/inputs. "
                         "Permission for this run is required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "reflection-values")
        config = server.artifacts / "reflection.toml"
        configuration = load_config()
        configuration["launch"]["arguments"][:0] = ["--woodpecker-diagnostics", "--woodpecker-reflection"]
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
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
        prefix = "test_app::diagnostics::reflection::"
        before_ticks = server.read_resource(session_id, "test_app::woodpecker::SimulationTicks")
        before_time = server.read_resource(session_id, "bevy_time::time::Time<()>")
        result = self._value(server, session_id, prefix + "Values")
        self.assertEqual(result["status"], "readable", result)
        values = result["value"]
        self.assertEqual(set(values), {"map", "integer_keys", "set", "variants", "path_handle",
                                      "ephemeral_handle", "untyped_handle", "other_ephemeral_handle",
                                      "other_untyped_handle", "uuid_handle"})
        self.assertEqual(values["map"], {"alpha": 1, "beta": 2})
        self.assertEqual(values["integer_keys"], {"-2": "negative", "7": "positive"})
        self.assertEqual(values["set"], ["a", "m", "z"])
        self.assertEqual(values["variants"], ["Idle", {"Count": 7}, {"Named": {"enabled": True}}])
        self.assertEqual(values["path_handle"], {"Path": "textures/prototypebits_texture.png"})
        self.assertEqual(values["uuid_handle"], {"Uuid": "12345678-1234-5678-90ab-cdef01234567"})
        bits = json.loads((Path(detail["artifact_dir"]) / "reflection-handle-bits.json").read_text())
        self.assertEqual(set(bits), {"ephemeral_handle", "other_ephemeral_handle"})
        self.assertNotEqual(bits["ephemeral_handle"], bits["other_ephemeral_handle"])
        for typed, untyped in (("ephemeral_handle", "untyped_handle"),
                               ("other_ephemeral_handle", "other_untyped_handle")):
            self.assertIs(type(bits[typed]), int)
            self.assertGreaterEqual(bits[typed], 0)
            self.assertLess(bits[typed], 1 << 64)
            expected = {"Ephemeral": {"id": f"{bits[typed]:016x}"}}
            self.assertEqual(values[typed], expected, typed)
            self.assertEqual(values[untyped], {
                "asset_type": "bevy_image::image::Image", "reference": expected,
            }, untyped)
        # This identity is session-local, not a hard-coded allocator index or
        # an expected equality between different application instances.
        self.assertEqual(self._value(server, session_id, prefix + "Values"), result)
        unavailable = {}
        for name, reason in (("NonFinite", "not_serializable"),
                             ("UnsupportedMapKey", "not_serializable"),
                             ("OpaqueValue", "not_serializable"),
                             ("NotReflectableResource", "not_reflectable")):
            unavailable[name] = self._value(server, session_id, prefix + name)
            self.assertEqual(unavailable[name], {"status": "unavailable", "reason": reason}, name)
        self.assertEqual(self._value(server, session_id, prefix + "OpaqueNull"),
                         {"status": "readable", "value": None})
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), before_ticks)
        self.assertEqual(server.read_resource(session_id, "bevy_time::time::Time<()>"), before_time)
        server.record(["observation", "reflection-values"], values=values, unavailable=unavailable,
                      opaque_null={"status": "readable", "value": None}, state_unchanged=True)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
