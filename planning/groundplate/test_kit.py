"""Windowless checks for authoring placement and the committed runtime exports."""
import json
from pathlib import Path
import unittest

from kit import Placement, StairModule, WORKSHOP, WORKSHOP_STAIR, game_point

ASSETS = Path(__file__).resolve().parents[2] / "test_app/asset"


class PlacementTests(unittest.TestCase):
    def test_fine_grid_and_quarter_turn_validation(self):
        for origin, turn in [((.125, 0, 0), 0), ((float("nan"), 0, 0), 0),
                             ((0, float("inf"), 0), 0), ((0, 0, 0), 4),
                             ((0, 0, 0), True)]:
            with self.subTest(origin=origin, turn=turn), self.assertRaises(ValueError):
                Placement(origin, turn)
        self.assertEqual(Placement((.25, .5, 3)).point((0, 0, 0)), (.25, .5, 3))

    def test_rotated_stair_keeps_model_socket_surfaces_and_rail_proxies_together(self):
        expected_exits = [(4, 13, 3), (-4, 5, 3), (4, -3, 3), (12, 5, 3)]
        for turn, exit_point in enumerate(expected_exits):
            with self.subTest(turn=turn):
                stair = StairModule(Placement((4, 5, 0), turn))
                self.assertEqual(stair.metadata()["exit"], game_point(exit_point))
                surfaces = list(stair.surfaces())
                self.assertEqual(len(surfaces), 12)
                self.assertEqual(surfaces[-1]["height"], 3)
                self.assertEqual(surfaces[-1]["half"],
                                 [1.5, 8 / 24] if turn % 2 == 0 else [8 / 24, 1.5])
                self.assertEqual(len(list(stair.rails())), 24)
                heights = [0] + [s["height"] for s in surfaces]
                self.assertTrue(all(abs(b - a - .25) < 1e-6
                                    for a, b in zip(heights, heights[1:])))

    def test_instances_share_one_asset_but_not_collider_names(self):
        other = StairModule(Placement((42, 23, 0)), name="Second stair instance")
        self.assertEqual(other.asset, WORKSHOP_STAIR.asset)
        original_names = {r["name"] for r in WORKSHOP_STAIR.rails()}
        other_names = {r["name"] for r in other.rails()}
        self.assertTrue(original_names.isdisjoint(other_names))
        self.assertNotEqual(other.metadata()["name"], WORKSHOP_STAIR.metadata()["name"])

    def test_exported_pilot_uses_the_authoring_definition(self):
        courtyard = json.loads((ASSETS / "meshes/environment/courtyard.gltf").read_text())
        data = courtyard["extras"]["signalstation"]["layout"]
        self.assertEqual(data["modules"], [WORKSHOP_STAIR.metadata()])
        self.assertEqual(data["workshop"], WORKSHOP.metadata())
        self.assertEqual(data["gears"][0], data["workshop"]["part"]["base"])
        for surface in WORKSHOP_STAIR.surfaces():
            self.assertIn(surface, data["surfaces"])
        for rail in WORKSHOP_STAIR.rails():
            self.assertIn(rail, data["obstacles"])
        self.assertNotIn(WORKSHOP_STAIR.name, [n.get("name") for n in courtyard["nodes"]])

    def test_complete_stair_asset_has_local_pivot_and_shared_material_mesh(self):
        path = ASSETS / WORKSHOP_STAIR.asset
        doc = json.loads(path.read_text())
        self.assertEqual(len(doc["nodes"]), 1)
        self.assertEqual(len(doc["meshes"]), 1)
        node = doc["nodes"][0]
        self.assertEqual(node.get("translation", [0, 0, 0]), [0, 0, 0])
        self.assertNotIn("rotation", node)
        primitives = doc["meshes"][node["mesh"]]["primitives"]
        self.assertEqual(len(primitives), 2)  # Treads and integrated handrails.
        bounds = [doc["accessors"][p["attributes"]["POSITION"]] for p in primitives]
        self.assertEqual(bounds[0]["min"], [-1.5, 0, -8])
        self.assertEqual(bounds[0]["max"], [1.5, 3, 0])
        for b in bounds:
            self.assertGreaterEqual(b["min"][0], -1.5)
            self.assertLessEqual(b["max"][0], 1.5)
            self.assertGreaterEqual(b["min"][2], -8)
            self.assertLessEqual(b["max"][2], 0)
            self.assertLessEqual(b["max"][1], 4.02)
        for buffer in doc["buffers"]:
            self.assertEqual((path.parent / buffer["uri"]).stat().st_size, buffer["byteLength"])


if __name__ == "__main__":
    unittest.main()
