"""Pointer tests and pure public Inspect queries; no implicit setup or inputs."""
import math

LOCATION = "bevy_picking::pointer::PointerLocation"
BUTTONS = "bevy_picking::pointer::PointerPressState"
MOUSE = "bevy_input::button_input::ButtonInput<bevy_input::mouse::MouseButton>"
TICKS = "test_app::woodpecker::SimulationTicks"


def read_components(server, session_id, with_types, type_paths=None):
    """Query and decode exactly the requested reflected components, without actions."""
    paths = list(with_types if type_paths is None else type_paths)
    items = server.call(session_id, {"command": "inspect.query", "arguments": {
        "source": "entities", "entity": None, "with": list(with_types), "without": [],
        "projection": {"kind": "components", "selection": {"kind": "listed", "type_paths": paths}},
    }})["items"]
    result = []
    for item in items:
        values = {}
        for component in item["result"]["components"]:
            if component["value"]["status"] != "readable":
                raise AssertionError(f"Component is not readable: {component}; see {server.artifacts}")
            values[component["component"]["type_path"]] = component["value"]["value"]
        result.append({"entity": item["entity"], "values": values})
    return result


def pointer_states(server, session_id):
    """Discover pointers through their public components, never a private UUID/name."""
    return [{"entity": item["entity"], "location": item["values"][LOCATION]["location"],
             "buttons": item["values"][BUTTONS]}
            for item in read_components(server, session_id, [LOCATION, BUTTONS])]


def layout_target(server, session_id, marker, *, name=None, label=None):
    """Find an existing widget by Name or direct child Text, then read its layout.

    Pure public queries only: no focus changes, inputs, ticks or cached handles.
    Coordinates are logical pixels; inverse_scale_factor converts physical UI layout.
    """
    if (name is None) == (label is None):
        raise ValueError("Choose exactly one widget Name or visible button label")
    projection = {"kind": "summary"} if name is not None else {"kind": "hierarchy", "depth": 1}
    roots = server.call(session_id, {"command": "inspect.query", "arguments": {
        "source": "entities", "entity": None, "with": [marker], "without": [],
        "projection": projection,
    }})["items"]
    if name is not None:
        matches = [item["entity"] for item in roots if item["result"]["name"] == name]
    else:
        text_type = "bevy_ui::widget::text::Text"
        texts = read_components(server, session_id, [text_type])
        by_handle = {}
        for item in texts:
            value = item["values"][text_type]
            if not isinstance(value, str):
                raise AssertionError(f"Expected reflected UI Text string: {item}")
            by_handle[(item["entity"]["index"], item["entity"]["generation"])] = " ".join(value.split())
        expected = " ".join(label.split())
        matches = [item["entity"] for item in roots
                   if any(by_handle.get((child["entity"]["index"], child["entity"]["generation"])) == expected
                          for child in item["result"]["root"]["children"])]
    if len(matches) != 1:
        raise AssertionError(f"Expected one {marker} with name={name!r}, label={label!r}: {matches}")
    node = "bevy_ui::ui_node::ComputedNode"
    transform = "bevy_ui::ui_transform::UiGlobalTransform"
    layouts = read_components(server, session_id, [marker, node, transform], [node, transform])
    selected = [item for item in layouts if item["entity"] == matches[0]]
    if len(selected) != 1:
        raise AssertionError(f"Selected widget has no unique computed layout: {matches[0]}")
    target = selected[0]
    computed, affine = target["values"][node], target["values"][transform]
    scale = computed["inverse_scale_factor"]
    numbers = [scale, *computed["size"], *affine]
    if (len(affine) != 6 or len(computed["size"]) != 2
            or not all(type(value) in (int, float) and math.isfinite(value) for value in numbers)
            or scale <= 0 or min(computed["size"]) <= 0):
        raise AssertionError(f"Expected finite visible widget layout: {target}")
    center = [affine[4] * scale, affine[5] * scale]
    if not (0 <= center[0] < 1440 and 0 <= center[1] < 900):
        raise AssertionError(f"Widget center outside the configured logical window: {target}")
    return {"entity": target["entity"], "position": center, "name": name, "label": label}
