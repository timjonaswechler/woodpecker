"""Entity filtering and component projections stay complete, ordered and read-only."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class EntityQueryTest(unittest.TestCase):
    def _query(self, server, session_id, projection, *, entity=None, with_types=(), without=()):
        return server.call(session_id, {"command": "inspect.query", "arguments": {
            "source": "entities", "entity": entity, "with": list(with_types), "without": list(without),
            "projection": projection,
        }})["items"]

    def test_filters_and_all_listed_projections_exclude_resource_entities_and_preserve_world(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One reflection-opt-in Signalstation window, manual startup, no ticks/inputs. "
                         "Permission for this run is required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "entity-query")
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
        marker, absent, nonreflect = (prefix + name for name in ("Marker", "AbsentComponent", "NotReflectableComponent"))
        summary = {"kind": "summary"}
        before = self._query(server, session_id, summary)
        handles = [(item["entity"]["index"], item["entity"]["generation"]) for item in before]
        self.assertEqual(handles, sorted(set(handles)))
        self.assertTrue(any(item["result"]["name"] is None for item in before))
        selected = self._query(server, session_id, summary, with_types=[marker])
        self.assertEqual(len(selected), 1)
        fixture = selected[0]
        handle = fixture["entity"]
        self.assertEqual(fixture["kind"], "entity")
        self.assertEqual(fixture["result"], {"kind": "summary", "name": "Woodpecker reflection fixture",
                                           "component_count": 4})
        self.assertEqual(self._query(server, session_id, summary, entity=handle), selected)
        self.assertEqual(self._query(server, session_id, summary, with_types=[marker, nonreflect]), selected)
        self.assertEqual(self._query(server, session_id, summary, with_types=[marker], without=[absent]), selected)
        for with_types, without in (([marker, absent], []), ([marker], [marker]),
                                    ([marker], [absent, nonreflect])):
            self.assertEqual(self._query(server, session_id, summary, with_types=with_types, without=without), [])
        # Resource types exist in this registry and world, but their internal
        # Bevy resource entities must never appear in the entity source.
        for resource_type in (prefix + "Values", "test_app::app::settings::Settings"):
            self.assertEqual(self._query(server, session_id, summary, with_types=[resource_type]), [])

        names = self._query(server, session_id, {"kind": "component_names"}, entity=handle)
        metadata = names[0]["result"]["components"]
        self.assertEqual(len(metadata), 4)
        self.assertTrue(all(set(component) == {"name", "type_path"} for component in metadata))
        self.assertEqual(metadata, sorted(metadata, key=lambda component:
                         (component["type_path"] or component["name"], component["name"])))
        unregistered = [component for component in metadata if component["type_path"] is None
                        and component["name"].endswith("::Unregistered")]
        self.assertEqual(len(unregistered), 1, metadata)
        self.assertEqual({component["type_path"] for component in metadata},
                         {marker, nonreflect, "bevy_ecs::name::Name", None})
        all_components = self._query(server, session_id, {
            "kind": "components", "selection": {"kind": "all"},
        }, entity=handle)[0]["result"]["components"]
        self.assertEqual([component["component"] for component in all_components], metadata)
        by_path = {component["component"]["type_path"]: component["value"] for component in all_components}
        self.assertEqual(by_path[marker], {"status": "readable", "value": {"label": "reflection"}})
        self.assertEqual(by_path[None], {"status": "unavailable", "reason": "not_registered"})
        self.assertEqual(by_path[nonreflect], {"status": "unavailable", "reason": "not_reflectable"})
        requested = [nonreflect, absent, marker]
        listed = self._query(server, session_id, {"kind": "components", "selection": {
            "kind": "listed", "type_paths": requested,
        }}, entity=handle)[0]["result"]["components"]
        self.assertEqual([component["component"]["type_path"] for component in listed], requested)
        self.assertEqual([component["value"] for component in listed], [
            {"status": "unavailable", "reason": "not_reflectable"},
            {"status": "unavailable", "reason": "missing"},
            {"status": "readable", "value": {"label": "reflection"}},
        ])
        self.assertEqual(self._query(server, session_id, {"kind": "components", "selection": {
            "kind": "listed", "type_paths": [],
        }}, entity=handle)[0]["result"]["components"], [])
        for invalid_part in ("with", "without", "listed"):
            arguments = {"source": "entities", "entity": handle, "with": [], "without": [],
                         "projection": summary}
            unknown = prefix + "UnknownComponent"
            if invalid_part == "listed":
                arguments["projection"] = {"kind": "components", "selection": {
                    "kind": "listed", "type_paths": [marker, unknown],
                }}
            else:
                arguments[invalid_part] = [unknown]
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "inspect.query", "arguments": arguments,
            }))
            rejected = server.result(session_id, pending)
            self.assertEqual(rejected["kind"], "rejected", rejected)
            self.assertEqual(rejected["error"]["code"], "unknown_type_path")
        self.assertEqual(self._query(server, session_id, summary), before)
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 0})
        server.record(["observation", "entity-query"], fixture=fixture, component_names=metadata,
                      listed_order=requested, world_summary_unchanged=True)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
