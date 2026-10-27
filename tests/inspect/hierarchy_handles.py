"""Hierarchy depth/order and stale handles follow real opt-in Bevy entity replacement."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class HierarchyHandlesTest(unittest.TestCase):
    def _nodes(self, server, session_id):
        path = "test_app::diagnostics::reflection::HierarchyNode"
        items = server.call(session_id, {"command": "inspect.query", "arguments": {
            "source": "entities", "entity": None, "with": [path], "without": [],
            "projection": {"kind": "components", "selection": {"kind": "listed", "type_paths": [path]}},
        }})["items"]
        self.assertEqual(len(items), 4)
        nodes = {}
        for item in items:
            component, = item["result"]["components"]
            self.assertEqual(component["value"]["status"], "readable")
            nodes[component["value"]["value"]["label"]] = item["entity"]
        self.assertEqual(set(nodes), {"root", "child-a", "child-z", "leaf"})
        return nodes

    def test_hierarchy_depth_children_order_and_replaced_handles_are_not_retargeted(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One reflection-opt-in Signalstation window, manual startup. F10 press/tick "
                         "replaces a four-entity hierarchy, then release/tick; exactly two ticks, no panic. "
                         "Permission for this run is required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "hierarchy-handles")
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
        old = self._nodes(server, session_id)
        self.assertGreater(old["child-z"]["index"], old["child-a"]["index"])
        names = {"root": "Inspect root", "child-a": "Inspect child A", "child-z": "Inspect child Z", "leaf": None}

        def node(handles, role, children=()):
            return {"entity": handles[role], "name": names[role], "children": list(children)}

        # Worked trees use handles discovered through a separate flat query,
        # never relationships copied from the hierarchy response under test.
        expected = {
            0: node(old, "root"),
            1: node(old, "root", [node(old, "child-z"), node(old, "child-a")]),
            2: node(old, "root", [node(old, "child-z", [node(old, "leaf")]), node(old, "child-a")]),
        }
        for depth in (0, 1, 2, 255):
            output = server.call(session_id, {"command": "inspect.query", "arguments": {
                "source": "entities", "entity": old["root"], "with": [], "without": [],
                "projection": {"kind": "hierarchy", "depth": depth},
            }})
            self.assertEqual(output, {"items": [{"kind": "entity", "entity": old["root"],
                             "result": {"kind": "hierarchy", "root": expected[min(depth, 2)]}}]})
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 0})
        server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f10"}})
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "f10"}})
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        replacement = self._nodes(server, session_id)
        for role in old:
            self.assertNotIn(old[role], replacement.values(), role)
        # Names and declared relationships stay fixed, but all handles are fresh.
        previous = old
        tree = node(replacement, "root", [node(replacement, "child-z", [node(replacement, "leaf")]),
                                           node(replacement, "child-a")])
        output = server.call(session_id, {"command": "inspect.query", "arguments": {
            "source": "entities", "entity": replacement["root"], "with": [], "without": [],
            "projection": {"kind": "hierarchy", "depth": 2},
        }})
        self.assertEqual(output["items"][0]["result"]["root"], tree)
        for handle in [*previous.values(), {"index": 4294967295, "generation": 0}]:
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "inspect.query", "arguments": {
                    "source": "entities", "entity": handle, "with": [], "without": [],
                    "projection": {"kind": "hierarchy", "depth": 2},
                },
            }))
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "rejected", outcome)
            self.assertEqual(outcome["error"]["code"], "entity_not_found")
        self.assertEqual(self._nodes(server, session_id), replacement)
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 2})
        server.record(["observation", "hierarchy-handles"], previous=previous, replacement=replacement,
                      depths=[0, 1, 2, 255], replacement_tree=tree)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
