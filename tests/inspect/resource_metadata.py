"""Resource selection distinguishes absent, unknown and metadata-only values."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class ResourceMetadataTest(unittest.TestCase):
    def _query(self, server, session_id, selector, projection):
        return server.call(session_id, {"command": "inspect.query", "arguments": {
            "source": "resources", "selector": selector, "projection": {"kind": projection},
        }}, timeout=30)["items"]

    def test_all_and_type_resource_metadata_are_sorted_and_do_not_serialize_values(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One reflection-opt-in Signalstation window, manual startup, no ticks/inputs. "
                         "Permission for this run is required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "resource-metadata", activity_bytes=16 * 1024 * 1024)
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
        ticks_type = "test_app::woodpecker::SimulationTicks"
        before = server.read_resource(session_id, ticks_type)
        prefix = "test_app::diagnostics::reflection::"
        fixture_paths = {prefix + name for name in (
            "Values", "NonFinite", "UnsupportedMapKey", "OpaqueValue", "OpaqueNull",
        )}
        metadata = self._query(server, session_id, {"kind": "all"}, "metadata")
        paths = [item["result"]["type_path"] for item in metadata]
        self.assertTrue(paths)
        self.assertEqual(paths, sorted(set(paths)))
        for item in metadata:
            self.assertEqual(item, {"kind": "resource", "result": {
                "kind": "metadata", "type_path": item["result"]["type_path"],
            }})
        # Both All projections need an independent oracle: agreeing with each
        # other must not hide the same omitted fixture resource in both lists.
        self.assertEqual({path for path in paths if path.startswith(prefix)}, fixture_paths)
        for path in sorted(fixture_paths):
            # NaN and opaque values must not prevent metadata access.
            self.assertEqual(self._query(server, session_id, {"kind": "type", "type_path": path}, "metadata"),
                             [{"kind": "resource", "result": {"kind": "metadata", "type_path": path}}])
        self.assertNotIn(prefix + "AbsentResource", paths)
        self.assertNotIn(prefix + "NotReflectableResource", paths)
        self.assertEqual(self._query(server, session_id, {
            "kind": "type", "type_path": prefix + "NotReflectableResource",
        }, "metadata"), [{"kind": "resource", "result": {
            "kind": "metadata", "type_path": prefix + "NotReflectableResource",
        }}])
        values = self._query(server, session_id, {"kind": "all"}, "value")
        value_paths = [item["result"]["type_path"] for item in values]
        self.assertEqual(value_paths, sorted(set(value_paths)))
        self.assertEqual({path for path in value_paths if path.startswith(prefix)}, fixture_paths)
        self.assertEqual(value_paths, paths)
        self.assertTrue(all(item["kind"] == "resource" and item["result"]["kind"] == "value"
                            for item in values))
        absent = prefix + "AbsentResource"
        self.assertEqual(self._query(server, session_id, {"kind": "type", "type_path": absent}, "metadata"), [])
        self.assertEqual(self._query(server, session_id, {"kind": "type", "type_path": absent}, "value"),
                         [{"kind": "resource", "result": {"kind": "value", "type_path": absent,
                           "value": {"status": "unavailable", "reason": "missing"}}}])
        for projection in ("metadata", "value"):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "inspect.query", "arguments": {"source": "resources",
                    "selector": {"kind": "type", "type_path": prefix + "UnknownResource"},
                    "projection": {"kind": projection}},
            }))
            rejected = server.result(session_id, pending)
            self.assertEqual(rejected["kind"], "rejected", rejected)
            self.assertEqual(rejected["error"]["code"], "unknown_type_path")
        self.assertEqual(server.read_resource(session_id, ticks_type), before)
        server.record(["observation", "resource-metadata"], resource_paths=paths,
                      expected_fixture_paths=sorted(fixture_paths),
                      absent_metadata=[], absent_value="missing", unknown="unknown_type_path")
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
