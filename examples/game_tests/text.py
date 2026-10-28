"""One app: discover the settings UI, focus Notes, enter Unicode, tick, inspect."""
import json
import math
import time

from _cli import CONFIG, LEVEL, SEED, SETTINGS, TICKS, Run, arguments

BUTTON = "bevy_ui_widgets::button::Button"
TEXT_INPUT = "bevy_ui_widgets::text_input::TextInput"
TEXT = "bevy_ui::widget::text::Text"
NODE = "bevy_ui::ui_node::ComputedNode"
TRANSFORM = "bevy_ui::ui_transform::UiGlobalTransform"


def components(game, session, with_types, selected):
    """Read-only component projection; no ticks, focus changes or cached handles."""
    items = game.call(session, "inspect.query", {
        "source": "entities", "entity": None, "with": with_types, "without": [],
        "projection": {"kind": "components", "selection": {
            "kind": "listed", "type_paths": selected,
        }},
    })["items"]
    result = []
    for item in items:
        values = {}
        for component in item["result"]["components"]:
            assert component["value"]["status"] == "readable", component
            values[component["component"]["type_path"]] = component["value"]["value"]
        result.append((item["entity"], values))
    return result


def widget_center(game, session, marker, *, name=None, label=None):
    """Discover an existing widget by public Name or child Text, then its layout."""
    roots = game.call(session, "inspect.query", {
        "source": "entities", "entity": None, "with": [marker], "without": [],
        "projection": {"kind": "summary"} if name else {"kind": "hierarchy", "depth": 1},
    })["items"]
    if name:
        matches = [item["entity"] for item in roots if item["result"]["name"] == name]
    else:
        texts = components(game, session, [TEXT], [TEXT])
        labels = {}
        for entity, values in texts:
            assert isinstance(values[TEXT], str), values
            labels[(entity["index"], entity["generation"])] = " ".join(values[TEXT].split())
        matches = [
            item["entity"] for item in roots
            if any(labels.get((child["entity"]["index"], child["entity"]["generation"]))
                   == " ".join(label.split())
                   for child in item["result"]["root"]["children"])
        ]
    assert len(matches) == 1, f"Expected one widget with name={name!r}, label={label!r}: {matches}"
    layouts = components(game, session, [marker, NODE, TRANSFORM], [NODE, TRANSFORM])
    matches = [(entity, values) for entity, values in layouts if entity == matches[0]]
    assert len(matches) == 1, "Selected widget needs a unique computed layout"
    entity, values = matches[0]
    node, affine = values[NODE], values[TRANSFORM]
    scale, size = node["inverse_scale_factor"], node["size"]
    assert len(affine) == 6 and len(size) == 2, values
    assert all(type(n) in (int, float) and math.isfinite(n) for n in [scale, *size, *affine]), values
    assert scale > 0 and min(size) > 0, "Widget is not laid out"
    center = [affine[4] * scale, affine[5] * scale]
    assert 0 <= center[0] < 1440 and 0 <= center[1] < 900, center
    return entity, center


def main():
    args = arguments(__doc__)
    with Run(args, "text") as game:
        session = game.own(game.cli("session", "create", "--config", str(CONFIG))["id"])
        game.wait_engine(session)
        startup_ticks = game.resource(session, TICKS)["ticks"]

        deadline = time.monotonic() + 10
        for extra_ticks in range(121):
            load = game.resource(session, LEVEL)
            if load == "Ready":
                break
            assert load == "Loading", load
            if extra_ticks == 120 or time.monotonic() >= deadline:
                raise TimeoutError("Level not ready within 120 explicit ticks / 10 s")
            assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
        game.note(preparation={"engine_ticks": startup_ticks, "level_ticks": extra_ticks})

        seed = json.loads(SEED.read_text(encoding="utf-8"))
        assert game.resource(session, SETTINGS) == seed
        before = game.resource(session, TICKS)["ticks"]
        # Existing settings button, then the existing multiline Notes input.
        # Each move/press/release gets its own explicit tick (six total).
        for marker, selector in (
            (BUTTON, {"label": "Einstellungen [M]"}),
            (TEXT_INPUT, {"name": "Notizen"}),
        ):
            entity, center = widget_center(game, session, marker, **selector)
            game.note(widget={"entity": entity, "center": center, **selector})
            for command, payload in (
                ("input.pointer.move_to", {"position": center}),
                ("input.pointer.press", {"button": "left"}),
                ("input.pointer.release", {"button": "left"}),
            ):
                assert game.call(session, command, payload) is None
                assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
                    "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
                }
        assert game.resource(session, "bevy_input_focus::InputFocus")["current_focus"] is not None
        assert game.resource(session, SETTINGS) == seed
        text = "Grüße 🦉\n東京"
        assert game.call(session, "input.text.input", {"text": text}) is None
        assert game.resource(session, SETTINGS) == seed, "Input must wait for an explicit tick"
        assert game.resource(session, TICKS)["ticks"] - before == 6

        # Bevy edits in PostUpdate; Signalstation mirrors the edit in the next Update.
        # Fixed edit + sync ticks, NOT a retry-until-the-text-matches loop.
        for _ in range(2):
            assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
        assert game.resource(session, SETTINGS) == {**seed, "notes": text}
        assert game.resource(session, TICKS)["ticks"] - before == 8
        game.note(observation={"notes": text, "action_ticks": 8})
        game.stop(session)
    print("PASS: exact Unicode Settings.notes after eight action ticks; owned session ended.")


if __name__ == "__main__":
    main()
