"""First-person controls, target practice and expedition through real game input."""
import math

from tests.session.controls import Controls, PRIMARY, PROJECTION
from tests.session.signalstation import CAMERA, NAME, TRANSFORM, components, entities

CURSOR = "bevy_window::window::CursorOptions"
VISIBILITY = "bevy_camera::visibility::Visibility"
TEXT = "bevy_ui::widget::text::Text"
PARENT = "bevy_ecs::hierarchy::ChildOf"


class FirstPerson(Controls):
    def __init__(self, game):
        super().__init__(game)
        self.scene_names = None

    def named(self, name, *types):
        # These level roots survive R. Resolve them once, then inspect just the
        # requested entity instead of serializing every named mesh each time.
        if self.scene_names is None:
            self.scene_names = {}
            for item in self.game.query(entities(with_types=(NAME,)))["items"]:
                self.scene_names.setdefault(item["result"]["name"], []).append(item["entity"])
        matches = self.scene_names.get(name, [])
        assert len(matches) == 1, (name, matches)
        items = self.game.query(entities(components(NAME, *types), entity=matches[0]))["items"]
        assert len(items) == 1, items
        values = {c["component"]["type_path"]: c["value"] for c in items[0]["result"]["components"]}
        assert values[NAME] == {"status": "readable", "value": name}, values
        assert all(v["status"] == "readable" for v in values.values()), values
        return {path: values[path]["value"] for path in types}

    def tap(self, key):
        self.key(key)
        self.game.advance(1)
        self.key(key, False)
        self.game.advance(1)

    def click(self):
        self.button()
        self.game.advance(1)
        self.button(down=False)
        self.game.advance(1)

    def captured(self):
        view = self.game.resource("view")
        assert view["first_person"] and view["captured"], view
        assert not self.game.resource("ui")["menu"]

    def cursor_free(self):
        item = self.game.query(entities(components(CURSOR), with_types=(PRIMARY,)))["items"]
        assert len(item) == 1, item
        cursor = item[0]["result"]["components"][0]["value"]
        assert cursor["status"] == "readable", cursor
        assert cursor["value"]["grab_mode"] == "None" and cursor["value"]["visible"], cursor
        return cursor["value"]

    def camera_pose(self):
        player, view = self.game.resource("player"), self.game.resource("view")
        camera = self.named("Spielkamera", TRANSFORM)[TRANSFORM]
        assert math.dist(camera["translation"], [player["position"][0], player["height"] + 1.5,
                                                player["position"][1]]) < 0.0001, camera
        x, y, z, w = camera["rotation"]
        forward = [-2 * (x * z + w * y), 2 * (w * x - y * z), 2 * (x * x + y * y) - 1]
        yaw, pitch = player["yaw"], view["pitch"]
        expected = [-math.sin(yaw) * math.cos(pitch), math.sin(pitch), -math.cos(yaw) * math.cos(pitch)]
        assert math.dist(forward, expected) < 0.0001, (camera, player, view)
        assert self.named("Blaster-Kamera", TRANSFORM)[TRANSFORM] == camera
        return camera

    def aim(self, yaw, pitch=0.0):
        player, view = self.game.resource("player"), self.game.resource("view")
        settings = self.game.resource("settings")
        scale = 0.0025 * settings["sensitivity"]
        turn = (yaw - player["yaw"] + math.pi) % math.tau - math.pi
        self.move_by([-turn / scale, (view["pitch"] - pitch) / scale])
        self.game.advance(1)
        actual, view = self.game.resource("player"), self.game.resource("view")
        assert abs((actual["yaw"] - yaw + math.pi) % math.tau - math.pi) < 0.0001, actual
        assert abs(view["pitch"] - pitch) < 0.0001, view

    def look_at(self, point):
        player = self.game.resource("player")
        x, y, z = point[0] - player["position"][0], point[1] - player["height"] - 1.5, point[2] - player["position"][1]
        self.aim(math.atan2(-x, -z), math.atan2(y, math.hypot(x, z)))

    def walk_to(self, point):
        """Follow a clear segment with W and relative mouse input, never teleport."""
        self.captured()
        speed = self.game.resource("settings")["speed"]
        step = speed * self.game.application.data["tick_nanos"] / 1_000_000_000
        for _ in range(4):
            player = self.game.resource("player")
            x, z = point[0] - player["position"][0], point[2] - player["position"][1]
            distance = math.hypot(x, z)
            if distance < 0.045:
                assert abs(player["height"] - point[1]) < 0.1, (point, player)
                return player
            self.aim(math.atan2(-x, -z))
            sprint = distance > 1
            if sprint:
                self.key("shift_left")
            self.key("w")
            try:
                stride = step * (1.65 if sprint else 1)
                ticks = max(1, int(distance / stride))
                assert ticks <= 1200, ("FPV-Wegbudget überschritten", point, player)
                remaining = distance
                while ticks:
                    batch = min(10, ticks)
                    self.game.advance(batch)
                    ticks -= batch
                    after = self.game.resource("player")
                    next_distance = math.dist(after["position"], [point[0], point[2]])
                    assert next_distance < remaining - 0.01, ("FPV-Weg blockiert", point, after)
                    remaining = next_distance
            finally:
                # Stop also makes cleanup possible when a warp completion times out.
                self.driver.command("tick.warp.stop", {})
                self.key("w", False)
                if sprint:
                    self.key("shift_left", False)
                self.game.advance(1)
        raise AssertionError(("FPV-Wegbudget überschritten", point, self.game.resource("player")))

    def travel(self, destination):
        """Ask the normal map click for a route, then walk its waypoints in FPV."""
        view = self.game.resource("view")
        if not view["first_person"]:
            self.tap("v")
        # Leaving FPV places the overview camera immediately; its actual transform
        # and projection determine the click, with no guessed settling tick count.
        self.tap("v" if view["map"] else "g")
        self.move_to(self.screen_position(destination))
        self.click()
        route = self.game.resource("player")["route"]
        assert route and math.dist(route[-1], destination) < 0.025, (destination, route)
        self.tap("v")
        self.captured()
        assert self.game.resource("player")["route"] == []
        for waypoint in route:
            self.walk_to(waypoint)
        assert math.dist(self.game.resource("player")["position"], [destination[0], destination[2]]) < 0.07
        return {"destination": destination, "waypoints": route, "player": self.game.resource("player")}

    def progress(self):
        return {key: value for key, value in self.game.resource("expedition").items() if key != "elapsed"}

    def inventory(self):
        """Read the displayed counts next to their labels, not a copied count formula."""
        items = self.game.query(entities(components(TEXT, PARENT), with_types=(TEXT, PARENT)))["items"]
        rows = []
        for item in items:
            values = {c["component"]["type_path"]: c["value"] for c in item["result"]["components"]}
            assert all(v["status"] == "readable" for v in values.values()), values
            rows.append((values[PARENT]["value"], values[TEXT]["value"]))
        inventory = {}
        for name in ("Bauteile", "Energiezellen", "Baupläne", "Münzen"):
            parents = [parent for parent, text in rows if text == name]
            assert len(parents) == 1, (name, parents)
            values = [text for parent, text in rows if parent == parents[0] and text != name]
            assert len(values) == 1 and values[0].isdigit(), (name, values)
            inventory[name] = int(values[0])
        return inventory

    def face_station(self, name):
        point = self.named(name, TRANSFORM)[TRANSFORM]["translation"]
        self.look_at([point[0], point[1] + 0.8, point[2]])
        self.captured()

    def interact(self):
        self.captured()
        self.tap("e")
        return {"expedition": self.progress(), "inventory": self.inventory()}

    def no_interaction(self):
        before = self.progress(), self.inventory()
        self.interact()
        assert (self.progress(), self.inventory()) == before, (before, self.progress(), self.inventory())


def run(ctx, game):
    controls = FirstPerson(game)

    def orientation():
        controls.background()
        controls.move_to([720, 450])
        controls.key("v")
        assert not game.resource("view")["first_person"]
        game.advance(1)
        controls.key("v", False)
        game.advance(1)
        controls.captured()
        camera = controls.named("Spielkamera", TRANSFORM, PROJECTION)
        assert camera[TRANSFORM]["translation"] == [-32.0, 1.5, 22.0], camera
        assert "Perspective" in camera[PROJECTION], camera
        assert controls.named("Blaster-Kamera", CAMERA)[CAMERA]["is_active"]
        assert controls.named("Blender Trainingsblaster", VISIBILITY)[VISIBILITY] == "Visible"
        controls.cursor_free()
        before = game.resource("player")["yaw"]
        rotations = []
        for index in range(2):
            controls.move_by([4000, -4000])
            assert game.resource("player")["yaw"] == before - index * 10
            game.advance(1)
            yaw, pitch = game.resource("player")["yaw"], game.resource("view")["pitch"]
            assert abs(yaw - (before - (index + 1) * 10)) < 0.00001, yaw
            assert abs(pitch - 1.45) < 0.00001, pitch
            assert controls.pointer()[0] > 1439.9 and controls.pointer()[1] == 0
            controls.camera_pose()
            rotations.append({"yaw": yaw, "pitch": pitch})
        controls.aim(0)
        controls.key("w")
        game.advance(4)
        controls.key("w", False)
        game.advance(1)
        assert math.dist(game.resource("player")["position"], [-32, 21.7]) < 0.0001
        controls.camera_pose()
        controls.background()
        controls.cursor_free()
        controls.tap("escape")
        assert not game.resource("view")["captured"]
        paused = game.resource("player")
        controls.key("w")
        controls.move_by([20, 10])
        game.advance(2)
        controls.key("w", False)
        game.advance(1)
        assert game.resource("player") == paused
        controls.move_to([720, 450])
        controls.click()
        controls.captured()
        assert game.resource("practice")["shots"] == 0, "Mausfang-Klick hat geschossen"
        return {"camera": camera, "rotations_at_cursor_edge": rotations,
                "window_focused": False, "cursor": controls.cursor_free()}

    ctx.steps.run("3.fpv_controls", orientation, requires=("2.restore",), precondition=controls.reset)

    def shooting():
        route = controls.travel([-4, 0, -14])
        targets = [controls.named(f"Zielscheibe {i}", TRANSFORM)[TRANSFORM]["translation"]
                   for i in (1, 2, 3)]

        def expect(shots, hits, score, destroyed):
            practice = game.resource("practice")
            assert (practice["shots"], practice["hits"], practice["score"], practice["targets_hit"]) == (
                shots, hits, score, destroyed), practice
            for index, broken in enumerate(destroyed, 1):
                assert controls.named(f"Zielscheibe {index}", VISIBILITY)[VISIBILITY] == (
                    "Hidden" if broken else "Visible")
            return practice

        controls.look_at(targets[1])
        controls.click()
        expect(1, 1, 50, [False, True, False])
        controls.aim(0, 0.8)  # Shoot above every target.
        game.advance(12)
        controls.click()
        expect(2, 1, 50, [False, True, False])
        controls.walk_to([-10, 0, -20])
        controls.look_at(targets[0])
        controls.click()
        expect(3, 1, 50, [False, True, False])
        assert "Zu nah" in game.resource("ui")["status"]
        controls.walk_to([-10, 0, -14])
        controls.look_at(targets[0])
        controls.click()
        expect(4, 2, 100, [True, True, False])
        game.advance(12)
        controls.click()  # A destroyed target never awards another hit or score.
        expect(5, 2, 100, [True, True, False])
        controls.look_at(targets[2])
        controls.button()
        game.advance(1)
        expect(5, 2, 100, [True, True, False])  # Still cooling down.
        game.advance(12)
        expect(5, 2, 100, [True, True, False])  # Holding does not fire later.
        controls.button(down=False)
        game.advance(1)
        controls.click()
        # At this oblique angle, aiming at the model centre hits the 25-point ring.
        final = expect(6, 3, 125, [True, True, True])
        assert game.resource("expedition")["practice_reward"]
        return {"route": route, "targets": targets, "practice": final,
                "reward_unlocked": True}

    ctx.steps.run("3.shooting", shooting, requires=("2.restore",), precondition=controls.reset)

    def part_pickup():
        assert game.resource("practice")["targets_hit"] == [False] * 3
        assert game.resource("practice")["shots"] == game.resource("practice")["score"] == 0
        travel = controls.travel([-30, 0, -1])
        controls.face_station("Table part 0")
        controls.no_interaction()  # Correct aim, beyond reach.
        controls.walk_to([-32.3, 0, -1])
        controls.face_station("Table part 0")
        controls.aim(game.resource("player")["yaw"] + math.pi)
        controls.no_interaction()  # Within reach, facing away.
        controls.face_station("Table part 0")
        before = controls.progress()
        controls.key("e")
        assert controls.progress() == before
        game.advance(1)
        assert controls.progress()["parts"] == [True, False]
        assert controls.inventory()["Bauteile"] == 1
        game.advance(3)
        assert controls.inventory()["Bauteile"] == 1
        controls.key("e", False)
        game.advance(1)
        controls.no_interaction()  # A collected part cannot be collected again.
        return {"travel": travel, "expedition": controls.progress(), "inventory": controls.inventory()}

    ctx.steps.run("3.part_pickup", part_pickup, requires=("2.restore",), precondition=controls.reset)

    def rack_and_part():
        assert controls.progress()["parts"] == [True, False]
        route = controls.travel([20, 0, 1.5])
        controls.face_station("Station rack")
        before = controls.named("Movable storage rack", TRANSFORM)[TRANSFORM]["translation"]
        opened = controls.interact()
        assert opened["expedition"]["rack_open"], opened
        moved = controls.named("Movable storage rack", TRANSFORM)[TRANSFORM]["translation"]
        assert math.dist(moved, [before[0], before[1], before[2] - 3]) < 0.0001, (before, moved)
        controls.no_interaction()
        second_route = controls.travel([27, 0, 2.6])
        controls.face_station("Table part 1")
        collected = controls.interact()
        assert collected["expedition"]["parts"] == [True, True], collected
        assert collected["inventory"]["Bauteile"] == 2, collected
        controls.no_interaction()
        return {"rack_route": route, "part_route": second_route,
                "rack_before": before, "rack_after": moved, **collected}

    rack_state = ctx.steps.run("3.rack_and_part", rack_and_part, requires=("3.part_pickup",))

    def craft_and_collect():
        assert controls.progress()["parts"] == [True, True]
        assert controls.progress()["cells"] == ["Locked"] * 3
        route = controls.travel([-31.7, 0, -4.5])
        controls.face_station("Station craft")
        crafted = controls.interact()
        assert crafted["expedition"]["cells"] == ["Available", "Locked", "Locked"], crafted
        assert crafted["inventory"]["Bauteile"] == crafted["inventory"]["Energiezellen"] == 0, crafted
        controls.no_interaction()
        pickup_route = controls.travel([-36.2, 0, -4.3])
        controls.face_station("Table cell 0")
        collected = controls.interact()
        assert collected["expedition"]["cells"] == ["Carried", "Locked", "Locked"], collected
        assert collected["inventory"]["Bauteile"] == 0 and collected["inventory"]["Energiezellen"] == 1, collected
        controls.no_interaction()
        return {"craft_route": route, "pickup_route": pickup_route,
                "after_craft": crafted, "after_pickup": collected}

    ctx.steps.run("3.craft_and_collect", craft_and_collect, requires=("3.rack_and_part",))

    def install_cell():
        assert controls.progress()["cells"] == ["Carried", "Locked", "Locked"]
        route = controls.travel([22, 0, 21.5])
        controls.face_station("Station generator")
        installed = controls.interact()
        assert installed["expedition"]["cells"] == ["Installed", "Locked", "Locked"], installed
        assert installed["inventory"]["Energiezellen"] == installed["inventory"]["Bauteile"] == 0, installed
        assert not installed["expedition"]["finished"], installed
        controls.no_interaction()
        return {"route": route, **installed}

    ctx.steps.run("3.install_cell", install_cell, requires=("3.craft_and_collect",))

    def reset_progress():
        before = controls.progress()
        assert before["rack_open"] and before["parts"] == [True, True], before
        assert before["cells"] == ["Installed", "Locked", "Locked"], before
        settings = game.resource("settings")
        controls.aim(0, 0.8)
        controls.click()
        assert game.resource("practice")["shots"] > 0
        ticks = game.tick_count()
        controls.reset()
        assert game.tick_count() == ticks + 2, "Spielreset hat die Simulationsuhr zurückgesetzt"
        expected = {"blueprints": [False] * 6, "cells": ["Locked"] * 3,
                    "checkpoint": False, "chest_claimed": False, "finished": False,
                    "parts": [False] * 2, "practice_reward": False, "rack_open": False,
                    "scans": [False] * 3, "secrets": [False] * 3, "valves": [False] * 2}
        assert controls.progress() == expected, controls.progress()
        assert 0 <= game.resource("expedition")["elapsed"] < 0.05
        assert not any(game.resource("coins")["collected"])
        practice = game.resource("practice")
        assert practice["shots"] == practice["hits"] == practice["score"] == 0, practice
        assert practice["targets_hit"] == [False] * 3, practice
        for index in (1, 2, 3):
            assert controls.named(f"Zielscheibe {index}", VISIBILITY)[VISIBILITY] == "Visible"
        rack = controls.named("Movable storage rack", TRANSFORM)[TRANSFORM]["translation"]
        assert rack == rack_state["rack_before"], rack
        assert all(value == 0 for value in controls.inventory().values()), controls.inventory()
        assert game.resource("settings") == settings, "R hat die Einstellungen verändert"
        assert not controls.named("Blaster-Kamera", CAMERA)[CAMERA]["is_active"]
        assert controls.named("Blender Trainingsblaster", VISIBILITY)[VISIBILITY] == "Hidden"
        controls.cursor_free()
        return {"before": before, "after": game.snapshot("fpv_reset"), "rack": rack,
                "preserved_settings": settings}

    ctx.steps.run("3.reset_progress", reset_progress, requires=("3.install_cell",))
    ctx.steps.run("3.restore", lambda: game.snapshot("after_fpv"),
                  requires=("2.restore",), precondition=controls.reset)
