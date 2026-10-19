"""The main rendered walkthrough shares one session and reports each check."""
import json
import time

from tests.catalog import scenario
from tests.commands._png import rgb_pixels
from tests.session.application import ROOT
from tests.session.signalstation import (
    CAMERA, NAME, RESOURCES, TRANSFORM, Signalstation, components, entities, resource_query,
)


@scenario("workflows", "signalstation", capabilities=("signalstation", "screenshot", "keyboard", "pointer", "text"),
          features=("connection", "inspect", "screenshot", "warp", "keyboard", "pointer", "text"),
          commands=("inspect.query", "screenshot.capture", "tick.warp.start", "tick.warp.stop",
                    "tick.warp.set_pace", "input.keyboard.press", "input.keyboard.release",
                    "input.pointer.move_to", "input.pointer.move_by",
                    "input.pointer.press", "input.pointer.release", "input.pointer.scroll", "input.text.input"))
def signalstation(ctx):
    # The app must seed its own workspace; tests never replace the user's settings.
    personal = ROOT / "test_app/settings.json"
    original = personal.read_bytes() if personal.exists() else None
    seed = (ctx.app.path.parent / ctx.app.data["settings_seed"]).read_bytes()
    with ctx.session() as driver:
        game = Signalstation(driver, ctx.app)
        steps = ctx.steps
        steps.run("1.engine_ready", game.engine_ready)
        steps.run("1.game_ready", game.wait_ready, requires=("1.engine_ready",))
        before = None

        def baseline():
            nonlocal before
            settings = game.resource("settings")
            assert settings == json.loads(seed), settings
            profile = game.artifacts / "workspace/config/settings.json"
            assert profile.read_bytes() == seed, "Session-Profil weicht von der Ausgangskonfiguration ab"
            assert (personal.read_bytes() if personal.exists() else None) == original
            state = game.snapshot("baseline")
            assert state["player"]["position"] == [-32.0, 22.0], state["player"]
            assert state["player"]["route"] == [], state["player"]
            assert state["view"]["mode"] != "FirstPerson" and not state["flow"]["menu"], state["view"]
            assert state["expedition"]["cells"] == ["Locked"] * 3, state["expedition"]
            assert state["practice"]["shots"] == state["practice"]["hits"] == 0
            before = state
            return {"snapshot": "checks/baseline.json", "settings": settings,
                    "ticks_since_ready": game.tick_count() - game.ready_ticks}

        steps.run("0.baseline", baseline, requires=("1.game_ready",))

        def resources():
            all_resources = game.query({"source": "resources", "selector": {"kind": "all"},
                                        "projection": {"kind": "metadata"}})
            paths = [i["result"]["type_path"] for i in all_resources["items"]]
            assert paths == sorted(set(paths)), paths
            assert set(RESOURCES.values()) <= set(paths), set(RESOURCES.values()) - set(paths)
            for path in RESOURCES.values():
                assert game.query(resource_query(path, "metadata")) == {"items": [{
                    "kind": "resource", "result": {"kind": "metadata", "type_path": path}}]}
            game.query(resource_query("suite::DoesNotExist"), expected_error="unknown_type_path")
            # Name is a registered component, but no Name resource exists.
            missing = game.query(resource_query(NAME))["items"]
            assert missing[0]["result"]["value"] == {"status": "unavailable", "reason": "missing"}, missing
            return game.unchanged(before, "after_resources")

        steps.run("1.resource_queries", resources, requires=("0.baseline",),
                  precondition=lambda: game.unchanged(before, "before_resources"))

        def entity_queries():
            cameras = game.query(entities(with_types=(CAMERA, NAME)))["items"]
            assert {"Spielkamera", "HUD-Kamera"} <= {c["result"]["name"] for c in cameras}, cameras
            camera = next(c for c in cameras if c["result"]["name"] == "Spielkamera")["entity"]
            named = game.query(entities({"kind": "component_names"}, camera))["items"]
            names = {c["type_path"] for c in named[0]["result"]["components"]}
            assert {NAME, CAMERA, TRANSFORM} <= names, names
            assert game.query(entities(entity=camera, with_types=(NAME,)))["items"]
            assert game.query(entities(entity=camera, without=(NAME,)))["items"] == []
            assert game.query(entities(with_types=(CAMERA,), without=(CAMERA,)))["items"] == []
            values = game.query(entities(components(NAME, TRANSFORM, RESOURCES["player"]), camera))["items"]
            values = {c["component"]["type_path"]: c["value"] for c in values[0]["result"]["components"]}
            assert values[NAME] == {"status": "readable", "value": "Spielkamera"}, values[NAME]
            assert values[TRANSFORM]["status"] == "readable", values[TRANSFORM]
            assert values[RESOURCES["player"]] == {"status": "unavailable", "reason": "missing"}
            hierarchy = game.query(entities({"kind": "hierarchy", "depth": 1}, camera))["items"]
            assert hierarchy[0]["result"]["root"]["entity"] == camera, hierarchy
            assert hierarchy[0]["result"]["root"]["name"] == "Spielkamera", hierarchy
            game.query(entities(with_types=("suite::DoesNotExist",)), expected_error="unknown_type_path")
            stale = {**camera, "generation": camera["generation"] + 1}
            game.query(entities(entity=stale), expected_error="entity_not_found")
            return game.unchanged(before, "after_entities")

        steps.run("1.entity_queries", entity_queries, requires=("0.baseline",),
                  precondition=lambda: game.unchanged(before, "before_entities"))

        def idle():
            time.sleep(0.25)
            game.unchanged(before, "after_idle")
            time.sleep(0.25)
            return game.unchanged(before, "after_second_idle")

        steps.run("1.idle", idle, requires=("0.baseline",),
                  precondition=lambda: game.unchanged(before, "before_idle"))

        def screenshots():
            expected = ctx.app.data["screenshot"]
            images = []
            landmarks = []
            for path, overwritten in (("screenshots/start.png", False),
                                      ("screenshots/repeated.png", False),
                                      ("screenshots/repeated.png", True)):
                result = driver.command("screenshot.capture", {"path": path})
                assert result == {"path": path, "width": expected["width"],
                                  "height": expected["height"], "overwritten": overwritten}, result
                width, height, rows = rgb_pixels(game.artifacts / path)
                assert (width, height) == (expected["width"], expected["height"])
                assert any(any(row) for row in rows), "Screenshot ist schwarz"
                measured = {}
                # Fixed screen regions cover HUD, entrance floor and the teal start pad.
                # Broad color bounds tolerate rendering differences, but not missing scene content.
                for landmark in expected["landmarks"]:
                    x0, y0, x1, y1 = landmark["rect"]
                    assert 0 <= x0 < x1 <= width and 0 <= y0 < y1 <= height, landmark
                    matching = sum(all(landmark["rgb_min"][c] <= row[x * 3 + c] <= landmark["rgb_max"][c]
                                       for c in range(3))
                                   for row in rows[y0:y1] for x in range(x0, x1))
                    measured[landmark["name"]] = matching / ((x1 - x0) * (y1 - y0))
                    assert measured[landmark["name"]] >= landmark["minimum_fraction"], (landmark, measured)
                landmarks.append(measured)
                images.append(rows)
                game.unchanged(before, f"after_capture_{len(images)}")
            tolerance = expected["channel_tolerance"]
            fractions = []
            for rows in images[1:]:
                changed = sum(any(abs(a[i + c] - b[i + c]) > tolerance for c in range(3))
                              for a, b in zip(images[0], rows) for i in range(0, len(a), 3))
                fraction = changed / (width * height)
                fractions.append(fraction)
                assert fraction <= expected["maximum_changed_fraction"], fractions
            return {"images": ["screenshots/start.png", "screenshots/repeated.png"],
                    "landmarks": landmarks, "changed_fractions": fractions,
                    "ticks_since_ready": game.tick_count() - game.ready_ticks}

        steps.run("1.screenshots", screenshots, requires=("0.baseline",),
                  precondition=lambda: game.unchanged(before, "before_screenshots"))

        def invalid_paths():
            for path in ("../escape.png", "/escape.png", "screenshots/../escape.png", "image.jpg"):
                driver.command("screenshot.capture", {"path": path}, expected_error="invalid_screenshot_path")
            assert not (game.artifacts.parent / "escape.png").exists()
            return game.unchanged(before, "after_invalid_paths")

        steps.run("1.invalid_screenshot_paths", invalid_paths, requires=("0.baseline",),
                  precondition=lambda: game.unchanged(before, "before_invalid_paths"))
        from tests.workflows import fpv, movement, settings
        movement.run(ctx, game)
        fpv.run(ctx, game)
        settings.run(ctx, game)
