"""UI, text editing and persisted settings through normal virtual input."""
import json
import math
import sys

from tests.commands._png import rgb_pixels
from tests.session.application import ROOT
from tests.session.signalstation import components, entities
from tests.workflows.fpv import FirstPerson, TEXT

NODE = "bevy_ui::ui_node::ComputedNode"
UI_TRANSFORM = "bevy_ui::ui_transform::UiGlobalTransform"
FOCUS = "bevy_input_focus::InputFocus"
TEXT_INPUT = "input.text.input"


class SettingsUi(FirstPerson):
    def open_menu(self):
        if not self.game.resource("flow")["menu"]:
            # Escape also leaves a focused name field before a menu shortcut.
            self.tap("escape")
        if not self.game.resource("flow")["menu"]:
            self.tap("m")
        assert self.game.resource("flow")["menu"]

    def select_all(self):
        modifier = "super_left" if sys.platform == "darwin" else "control_left"
        self.key(modifier)
        self.key("a")
        self.game.advance(1)
        self.key("a", False)
        self.key(modifier, False)
        self.game.advance(1)

    def commit(self, value):
        self.driver.command(TEXT_INPUT, {"text": value})
        # Bevy edits in PostUpdate; the app syncs and saves in the next Update.
        self.game.advance(2)

    def layout(self, values):
        node, transform = values[NODE], values[UI_TRANSFORM]
        assert min(node["size"]) > 0, "UI-Element ist nicht sichtbar"
        scale = node["inverse_scale_factor"]
        return {"center": [v * scale for v in transform[4:6]],
                "size": [v * scale for v in node["size"]],
                "content_origin": [(transform[4 + axis] - node["size"][axis] / 2
                                    + node["padding"]["min_inset"][axis]
                                    + node["border"]["min_inset"][axis]) * scale for axis in (0, 1)]}

    def label(self, text, row=None):
        items = self.game.query(entities(components(TEXT, NODE, UI_TRANSFORM), with_types=(TEXT,)))["items"]
        matches = []
        for item in items:
            values = {c["component"]["type_path"]: c["value"] for c in item["result"]["components"]}
            if values[TEXT] == {"status": "readable", "value": text}:
                assert all(v["status"] == "readable" for v in values.values()), values
                if min(values[NODE]["value"]["size"]) > 0:
                    matches.append(self.layout({k: v["value"] for k, v in values.items()}))
        if row is not None:
            y = self.label(row)["center"][1]
            matches = [match for match in matches if abs(match["center"][1] - y) < 3]
        assert len(matches) == 1, (text, row, matches)
        return matches[0]

    def field(self, name):
        return self.layout(self.named(name, NODE, UI_TRANSFORM))

    def hover(self, position):
        self.move_to(position)
        self.game.advance(1)

    def click_label(self, text, row=None):
        target = self.label(text, row)
        self.hover(target["center"])
        self.click()
        return target

    def focus_field(self, name):
        target = self.field(name)
        self.hover(target["center"])
        self.click()
        return target

    def saved(self):
        path = self.game.artifacts / "workspace/config/settings.json"
        value = json.loads(path.read_text())
        actual = self.game.resource("settings")
        assert value.keys() == actual.keys(), (value, actual)
        for name, stored in value.items():
            # JSON files serialize f32 values compactly; reflection promotes them.
            equal = (math.isclose(stored, actual[name], rel_tol=1e-6, abs_tol=1e-6)
                     if isinstance(stored, float) else stored == actual[name])
            assert equal, (name, stored, actual[name])
        assert not path.with_suffix(".json.tmp").exists()
        return value

    def restore_settings(self):
        self.driver.command("tick.warp.stop", {})
        self.release_all()
        seed = json.loads((self.game.application.path.parent / self.game.application.data["settings_seed"]).read_text())
        self.open_menu()
        self.click_label("Standardwerte")
        self.focus_field("Notizen")
        self.select_all()
        self.tap("backspace")
        if seed["notes"]:
            self.commit(seed["notes"])
        self.click_label("Zurück zum Spiel")
        if self.game.resource("capture")["active"]:
            self.tap("escape")
        self.focus_field("Spielername")
        self.select_all()
        self.commit(seed["player_name"])
        self.tap("enter")
        self.reset()
        assert self.saved() == seed
        return seed


def run(ctx, game):
    controls = SettingsUi(game)
    personal = ROOT / "test_app/settings.json"
    original_personal = personal.read_bytes() if personal.exists() else None
    seed_path = game.application.path.parent / game.application.data["settings_seed"]
    original_seed = seed_path.read_bytes()

    def menu():
        player, progress = game.resource("player"), controls.progress()
        practice, settings = game.resource("practice"), controls.saved()
        assert not game.resource("flow")["menu"]
        controls.driver.command(TEXT_INPUT, {"text": "kein Fokus"}, expected_error="text_focus_unavailable")
        target = controls.label("Einstellungen  [M]")
        controls.hover(target["center"])
        controls.button()
        assert not game.resource("flow")["menu"], "UI-Klick wirkte vor dem Tick"
        assert game.resource("player") == player
        game.advance(1)
        controls.button(down=False)
        game.advance(1)
        assert game.resource("flow")["menu"]
        assert game.resource("player") == player, "UI-Klick hat einen Laufweg ausgelöst"
        for key in ("w", "space", "v", "g", "r", "e"):
            controls.key(key)
        game.advance(4)
        controls.release_all()
        game.advance(1)
        assert game.resource("player") == player
        assert controls.progress() == progress and game.resource("practice") == practice
        assert game.resource("flow")["menu"] and game.resource("view")["mode"] != "FirstPerson"
        assert controls.saved() == settings
        controls.click_label("Zurück zum Spiel")
        assert not game.resource("flow")["menu"]
        assert game.resource("player") == player
        controls.tap("m")
        assert game.resource("flow")["menu"]
        controls.tap("escape")
        assert not game.resource("flow")["menu"]
        assert game.resource("player") == player
        return {"settings_button": target, "player": player, "settings": settings}

    ctx.steps.run("4.menu", menu, requires=("3.restore",), precondition=controls.reset)

    def notes_unicode():
        controls.tap("m")
        assert game.resource("flow")["menu"]
        field = controls.focus_field("Notizen")
        focus = game.resource(FOCUS)["current_focus"]
        assert focus is not None
        before = controls.saved()
        assert before["notes"] == ""
        message = "Grüße · 世界 · 🙂\nZweite Zeile: é · Ende: 界🙂"
        tick = game.tick_count()
        controls.driver.command(TEXT_INPUT, {"text": message})
        assert controls.saved() == before, "Text wurde schon bei Befehlsannahme gespeichert"
        game.advance(1)
        assert controls.saved() == before, "Die App speichert erst nach Bevys Textverarbeitung"
        game.advance(1)
        assert controls.saved()["notes"] == message
        assert game.tick_count() == tick + 2
        controls.tap("arrow_left")
        controls.commit("|")
        assert controls.saved()["notes"] == message[:-1] + "|🙂", "Links muss über das ganze Emoji springen"
        controls.tap("backspace")
        assert controls.saved()["notes"] == message
        controls.tap("backspace")
        assert controls.saved()["notes"] == message[:-2] + "🙂", "Backspace beschädigt Unicode"
        controls.commit("界")
        assert controls.saved()["notes"] == message
        controls.tap("arrow_right")
        controls.tap("enter")
        assert game.resource(FOCUS)["current_focus"] == focus, "Enter hat das mehrzeilige Feld verlassen"
        controls.commit("Neue Zeile")
        expected = message + "\nNeue Zeile"
        assert controls.saved()["notes"] == expected
        controls.tap("home")
        controls.tap("delete")
        expected = message + "\neue Zeile"
        assert controls.saved()["notes"] == expected
        controls.commit("")
        assert controls.saved()["notes"] == expected
        controls.tap("escape")
        assert game.resource(FOCUS)["current_focus"] != focus
        controls.driver.command(TEXT_INPUT, {"text": "kein Fokus"}, expected_error="text_focus_unavailable")
        assert controls.saved()["notes"] == expected
        return {"field": field, "stored_text": expected, "commit_to_save_ticks": 2}

    ctx.steps.run("4.notes_unicode", notes_unicode, requires=("4.menu",))

    def name_limit():
        controls.open_menu()
        controls.click_label("Zurück zum Spiel")
        notes = controls.saved()["notes"]
        field = controls.focus_field("Spielername")
        focus = game.resource(FOCUS)["current_focus"]
        controls.select_all()
        name = "Ä界🙂" * 8
        assert len(name) == 24 and len(name.encode("utf-8")) == 72
        controls.commit(name)
        assert controls.saved()["player_name"] == name
        controls.commit("X")
        assert controls.saved()["player_name"] == name, "Namensfeld hat die 24-Zeichen-Grenze überschritten"
        assert controls.saved()["notes"] == notes
        controls.tap("enter")
        assert game.resource(FOCUS)["current_focus"] != focus
        assert not game.resource("flow")["menu"]
        controls.driver.command(TEXT_INPUT, {"text": "kein Fokus"}, expected_error="text_focus_unavailable")
        assert controls.saved()["player_name"] == name
        return {"field": field, "stored_name": name, "characters": 24, "utf8_bytes": 72}

    ctx.steps.run("4.name_limit", name_limit, requires=("4.menu",))

    def text_limit():
        controls.open_menu()
        controls.focus_field("Notizen")
        controls.select_all()
        controls.tap("backspace")
        prefix = "Präfix\n"
        controls.commit(prefix)
        line = "Grüße 世界🙂\n"
        boundary = line * (16384 // len(line.encode("utf-8")))
        boundary += "x" * (16384 - len(boundary.encode("utf-8")))
        assert len(boundary.encode("utf-8")) == 16384
        before, tick = controls.saved(), game.tick_count()
        controls.driver.command(TEXT_INPUT, {"text": boundary})
        assert controls.saved() == before
        game.advance(2)
        assert controls.saved()["notes"] == prefix + boundary
        assert game.tick_count() == tick + 2
        expected = prefix + boundary + "\nEnde🙂"
        controls.commit("\nEnde🙂")
        assert controls.saved()["notes"] == expected, "16 KiB begrenzen den Befehl, nicht das gesamte Feld"
        tick = game.tick_count()
        for rejected in (boundary + "x", "🙂" * 4097):
            controls.driver.command(TEXT_INPUT, {"text": rejected}, expected_error="text_too_large")
            assert controls.saved()["notes"] == expected
            assert game.tick_count() == tick
        game.advance(2)
        assert controls.saved()["notes"] == expected, "Abgelehnter Text wurde später doch angewandt"
        return {"accepted_command_bytes": 16384, "stored_bytes": len(expected.encode("utf-8")),
                "rejected_command_bytes": [16385, 16388], "stored_characters": len(expected)}

    ctx.steps.run("4.text_limit", text_limit, requires=("4.menu",))

    def notes_scroll():
        controls.open_menu()
        controls.focus_field("Notizen")
        controls.select_all()
        text = "ANFANG\n" + "\n".join(f"Zeile {i:03d}: Grüße aus der Signalstation" for i in range(120)) + "\nENDE"
        controls.commit(text)
        assert controls.saved()["notes"] == text
        field = controls.field("Notizen")
        assert 70 < field["size"][1] < 180, "Lange Notizen haben das Feld ausgedehnt"
        center, size = field["center"], field["size"]
        x0, y0 = [round(center[i] - size[i] / 2 + 3) for i in (0, 1)]
        x1, y1 = [round(center[i] + size[i] / 2 - 3) for i in (0, 1)]

        def capture(name):
            path = f"screenshots/notes-{name}.png"
            tick = game.tick_count()
            controls.driver.command("screenshot.capture", {"path": path})
            assert game.tick_count() == tick
            width, height, pixels = rgb_pixels(game.artifacts / path)
            assert 0 <= x0 < x1 <= width and 0 <= y0 < y1 <= height
            return [row[x0 * 3:x1 * 3] for row in pixels[y0:y1]]

        def difference(left, right):
            changed = sum(any(abs(a[i + c] - b[i + c]) > 2 for c in range(3))
                          for a, b in zip(left, right) for i in range(0, len(a), 3))
            return changed / ((x1 - x0) * (y1 - y0))

        bottom = capture("bottom")
        controls.driver.command("input.pointer.scroll", {"delta": [0, 10000]})
        assert difference(bottom, capture("queued-scroll")) <= 0.002, "Scrollen wirkte vor dem Tick"
        game.advance(1)
        top = capture("top")
        changed = difference(bottom, top)
        assert changed > 0.01, ("Sichtbarer Text hat sich beim Scrollen nicht geändert", changed)
        assert controls.saved()["notes"] == text
        # Click the first visible line after scrolling, then edit at its start.
        # This proves we reached the beginning without inspecting editor internals.
        controls.hover([v + 2 for v in field["content_origin"]])
        controls.click()
        controls.tap("home")
        controls.commit("TOP ")
        assert controls.saved()["notes"] == "TOP " + text, "Scrollen hat den Textanfang nicht erreichbar gemacht"
        controls.hover(field["center"])
        controls.driver.command("input.pointer.scroll", {"delta": [0, -10000]})
        game.advance(1)
        returned = capture("bottom-again")
        assert difference(top, returned) > 0.01, "Rückwärts scrollen verändert den sichtbaren Text nicht"
        assert controls.field("Notizen")["size"] == size
        assert controls.saved()["notes"] == "TOP " + text
        # Scrolling and caret-following may rasterize the same final lines at
        # different subpixel offsets. Verify the reachable end by editing it.
        controls.hover([field["content_origin"][0] + 2, center[1] + size[1] / 2 - 16])
        controls.click()
        controls.tap("end")
        controls.commit(" BOTTOM")
        assert controls.saved()["notes"] == "TOP " + text + " BOTTOM", "Scrollen hat das Textende nicht erreichbar gemacht"
        return {"field": field, "bottom_to_top_changed_fraction": changed,
                "images": ["screenshots/notes-bottom.png", "screenshots/notes-top.png",
                           "screenshots/notes-bottom-again.png"]}

    ctx.steps.run("4.notes_scroll", notes_scroll, requires=("4.text_limit",))

    def settings_effects():
        controls.open_menu()
        controls.click_label("Zurück zum Spiel")
        controls.reset()
        controls.open_menu()
        before = controls.saved()
        assert (before["sensitivity"], before["speed"], before["fov"]) == (1, 4.5, 75)
        assert before["show_path"] and before["shadows"] and not before["invert_y"]
        for label in ("Maus-Empfindlichkeit    1.0", "FPV-Sichtfeld    75°", "Lauftempo    4.5 m/s"):
            target = controls.label("+", row=label)
            controls.hover(target["center"])
            previous = controls.saved()
            controls.button()
            assert controls.saved() == previous, "Einstellung wurde vor dem Tick verändert"
            game.advance(1)
            controls.button(down=False)
            game.advance(1)
        for label in ("Laufroute anzeigen: An", "Schatten: An", "Maus Y-Achse umkehren: Aus"):
            controls.click_label(label)
        changed = controls.saved()
        assert math.isclose(changed["sensitivity"], 1.2, abs_tol=1e-6), changed
        assert (changed["speed"], changed["fov"]) == (5.0, 80.0), changed
        assert not changed["show_path"] and not changed["shadows"] and changed["invert_y"], changed
        for name in ("player_name", "notes"):
            assert changed[name] == before[name]
        for label in ("Maus-Empfindlichkeit    1.2", "FPV-Sichtfeld    80°", "Lauftempo    5.0 m/s",
                      "Laufroute anzeigen: Aus", "Schatten: Aus", "Maus Y-Achse umkehren: An"):
            controls.label(label)

        def shadows(expected):
            light = "bevy_light::directional_light::DirectionalLight"
            items = game.query(entities(components(light), with_types=(light,)))["items"]
            assert items, "Kein gerichtetes Licht vorhanden"
            for item in items:
                value = item["result"]["components"][0]["value"]
                assert value["status"] == "readable", value
                assert value["value"]["shadow_maps_enabled"] is expected, value

        shadows(False)
        controls.click_label("Zurück zum Spiel")
        controls.tap("v")
        controls.captured()
        from tests.session.controls import PROJECTION
        camera = controls.named("Spielkamera", PROJECTION)[PROJECTION]["Perspective"]
        assert abs(camera["fov"] - math.radians(80)) < 0.00001, camera
        controls.key("w")
        game.advance(6)
        controls.key("w", False)
        game.advance(1)
        assert math.dist(game.resource("player")["position"], [-32, 21.5]) < 0.0001
        controls.move_by([40, 20])
        game.advance(1)
        assert abs(game.resource("player")["yaw"] + 0.12) < 0.00001
        assert abs(game.resource("view")["pitch"] - 0.06) < 0.00001
        controls.camera_pose()

        controls.tap("m")
        assert game.resource("flow")["menu"]
        player, progress = game.resource("player"), controls.progress()
        practice = game.resource("practice")
        controls.hover([720, 870])
        controls.key("w")
        controls.key("e")
        controls.move_by([20, 10])
        controls.button()
        game.advance(3)
        controls.release_all()
        game.advance(1)
        assert game.resource("player") == player and controls.progress() == progress
        assert game.resource("practice")["shots"] == practice["shots"]
        assert game.resource("flow")["menu"] and not game.resource("capture")["active"]
        controls.click_label("Standardwerte")
        restored = controls.saved()
        assert (restored["sensitivity"], restored["speed"], restored["fov"]) == (1.0, 4.5, 75.0)
        assert restored["show_path"] and restored["shadows"] and not restored["invert_y"]
        assert restored["player_name"] == before["player_name"] and restored["notes"] == before["notes"]
        shadows(True)
        controls.click_label("Zurück zum Spiel")
        controls.captured()
        controls.aim(0)
        position = game.resource("player")["position"]
        controls.key("w")
        game.advance(6)
        controls.key("w", False)
        game.advance(1)
        assert math.dist(game.resource("player")["position"], [position[0], position[1] - 0.45]) < 0.0001
        controls.move_by([40, 20])
        game.advance(1)
        assert abs(game.resource("player")["yaw"] + 0.1) < 0.00001
        assert abs(game.resource("view")["pitch"] + 0.05) < 0.00001
        camera = controls.named("Spielkamera", PROJECTION)[PROJECTION]["Perspective"]
        assert abs(camera["fov"] - math.radians(75)) < 0.00001
        return {"changed": {k: v for k, v in changed.items() if k not in ("notes", "player_name")},
                "six_tick_distances": [0.5, 0.45], "mouse_delta": [40, 20],
                "yaw_pitch_changed": [-0.12, 0.06], "yaw_pitch_defaults": [-0.1, -0.05],
                "personal_text_preserved": True}

    ctx.steps.run("4.settings_effects", settings_effects, requires=("4.menu",))

    def focus_gameplay():
        route = controls.travel([-32.3, 0, -1])
        controls.face_station("Table part 0")
        controls.tap("escape")
        assert game.resource("view")["mode"] == "FirstPerson" and not game.resource("capture")["active"]
        controls.focus_field("Spielername")
        controls.select_all()
        controls.commit("Eingabe E")
        assert controls.saved()["player_name"] == "Eingabe E"
        focus = game.resource(FOCUS)["current_focus"]
        player, progress = game.resource("player"), controls.progress()
        assert progress["parts"] == [False, False]
        assert not game.resource("flow")["menu"]
        for key in ("w", "space", "e", "v", "r", "m"):
            controls.key(key)
        controls.move_by([90, 40])
        game.advance(3)
        controls.release_all()
        game.advance(1)
        assert game.resource("player") == player, "Texteingabe hat den Spieler bewegt"
        assert controls.progress() == progress, "E im Textfeld hat interagiert"
        assert not game.resource("flow")["menu"] and game.resource("view")["mode"] == "FirstPerson"
        assert game.resource(FOCUS)["current_focus"] == focus
        assert controls.saved()["player_name"] == "Eingabe E"
        controls.tap("enter")
        assert game.resource(FOCUS)["current_focus"] != focus
        controls.move_to([720, 450])
        controls.click()
        controls.captured()
        assert game.resource("practice")["shots"] == 0, "Fokus-Rückkehr hat geschossen"
        acquired = controls.interact()
        assert acquired["expedition"]["parts"] == [True, False], "E funktioniert nach Verlassen des Felds nicht"
        assert acquired["inventory"]["Bauteile"] == 1
        return {"route": route, "player_while_focused": player,
                "parts_while_focused": progress["parts"], "after_blur": acquired}

    ctx.steps.run("4.focus_gameplay", focus_gameplay, requires=("4.menu",),
                  precondition=controls.restore_settings)

    def restored():
        expected = json.loads(original_seed)
        assert controls.saved() == expected
        controls.assert_reset()
        controls.cursor_free()
        controls.driver.command(TEXT_INPUT, {"text": "kein Fokus"}, expected_error="text_focus_unavailable")
        assert (personal.read_bytes() if personal.exists() else None) == original_personal
        assert seed_path.read_bytes() == original_seed
        return {"snapshot": game.snapshot("after_settings"), "profile": "workspace/config/settings.json",
                "personal_settings_unchanged": True, "seed_unchanged": True}

    ctx.steps.run("4.restore", restored, requires=("3.restore",), precondition=controls.restore_settings)
