"""Windowless checks for authoring placement and the committed runtime exports."""
import json
from pathlib import Path
import unittest

from kit import Placement, StairModule, WORKSHOP, WORKSHOP_STAIR, STAIR_MODULES, game_point, station_metadata
import layout
from passages import PASSAGES, metadata as passage_metadata

ASSETS = Path(__file__).resolve().parents[2] / "test_app/asset"


class PlacementTests(unittest.TestCase):
    def test_build_grid_rejects_submetre_anchors_and_invalid_turns(self):
        for origin, turn in [((.125, 0, 0), 0), ((.25, 0, 0), 0),
                             ((.5, 0, 0), 0), ((float("nan"), 0, 0), 0),
                             ((0, float("inf"), 0), 0), ((0, 0, 0), 4),
                             ((0, 0, 0), True)]:
            with self.subTest(origin=origin, turn=turn), self.assertRaises(ValueError):
                Placement(origin, turn)
        self.assertEqual(Placement((1, 3, 3)).point((0, 0, 0)), (1, 3, 3))

    def test_task_anchors_and_approaches_use_the_build_grid(self):
        for station in WORKSHOP.metadata().values():
            for key in ("base", "approach"):
                for coordinate in station[key]:
                    self.assertEqual(coordinate, round(coordinate), (key, station))

    def test_model_details_are_not_snapped_to_the_build_grid(self):
        placement = Placement((1, 3, 0))
        self.assertEqual(placement.point((.125, .5, .25)), (1.125, 3.5, .25))
        first_tread, _ = next(WORKSHOP_STAIR.treads())
        self.assertEqual(first_tread[2] * 2, .25)

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
        self.assertEqual(data["modules"], [s.metadata() for s in STAIR_MODULES])
        self.assertEqual(data["workshop"], WORKSHOP.metadata())
        self.assertEqual(data["gears"][0], data["workshop"]["part"]["base"])
        for stair in STAIR_MODULES:
            for surface in stair.surfaces():
                self.assertIn(surface, data["surfaces"])
            for rail in stair.rails():
                self.assertIn(rail, data["obstacles"])
            self.assertNotIn(stair.name, [n.get("name") for n in courtyard["nodes"]])

    def test_entire_level_has_grid_aligned_architecture_and_shared_task_stations(self):
        doc = json.loads((ASSETS / "meshes/environment/courtyard.gltf").read_text())
        data = doc["extras"]["signalstation"]["layout"]
        self.assertEqual(len(data["modules"]), 6)
        self.assertEqual(len({m["asset"] for m in data["modules"]}), 1)
        self.assertEqual(data["stations"], station_metadata())
        for station in data["stations"].values():
            for key in ("base", "approach"):
                Placement(tuple(station[key]))
        for _, _, bounds, _ in layout.ROOMS:
            self.assertTrue(all(v == round(v) for v in bounds))
        for _, bounds, _ in layout.DECKS:
            self.assertTrue(all(v == round(v) for v in bounds))
        for _, coord, lo, hi in layout.walls():
            self.assertTrue(all(v == round(v) for v in (coord, lo, hi)))
            self.assertGreaterEqual(hi-lo, 1)
            self.assertLessEqual(hi-lo, 3)

    def test_secret_architecture_exports_two_portals_covers_and_original_physical_rewards(self):
        doc = json.loads((ASSETS / "meshes/environment/courtyard.gltf").read_text())
        data = doc["extras"]["signalstation"]["layout"]
        self.assertEqual(data["passages"], passage_metadata())
        self.assertEqual(data["blueprints"][:3], [game_point(p["reward"]) for p in PASSAGES])
        names = {n.get("name") for n in doc["nodes"]}
        self.assertTrue(data["cutaways"])
        self.assertTrue(all(c["name"] in names for c in data["cutaways"]))
        self.assertTrue({"G1", "G2", "G3"}.isdisjoint(names))
        for i, passage in enumerate(data["passages"]):
            self.assertEqual(len(passage["portals"]), 2)
            for portal in passage["portals"]:
                self.assertIn({k: portal[k] for k in ("name", "center", "half")}, data["obstacles"])
            self.assertTrue(any(c["secret"] == i for c in data["cutaways"]))

    def test_workshop_gallery_has_visible_rails_except_at_its_connections(self):
        doc = json.loads((ASSETS / "meshes/environment/courtyard.gltf").read_text())
        obstacles = doc["extras"]["signalstation"]["layout"]["obstacles"]
        mesh_names = {n.get("name") for n in doc["nodes"] if "mesh" in n}

        def rails_at(x, y):
            point = game_point((x, y, 3.75))
            return [o for o in obstacles if o["name"].startswith("Gallery rail")
                    and all(abs(p - c) <= h + 1e-5 for p, c, h
                            in zip(point, o["center"], o["half"]))]

        for x, y in [(9.75, 31), (15, 31), (9, 33.5), (14, 36),
                     (18, 31.5), (18, 35.5)]:
            with self.subTest(edge=(x, y)):
                rails = rails_at(x, y)
                self.assertTrue(rails, "unprotected workshop gallery edge")
                self.assertTrue(all(o["name"] in mesh_names for o in rails))
        self.assertFalse(rails_at(12, 31), "stair landing must stay open")
        self.assertFalse(rails_at(18, 33.5), "bridge connection must stay open")

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
