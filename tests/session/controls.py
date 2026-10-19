"""Normal virtual inputs and a verified reset for the shared gameplay session."""
from tests.session.signalstation import CAMERA, NAME, TRANSFORM, components, entities

KEYS = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
BUTTONS = "bevy_input::button_input::ButtonInput<bevy_input::mouse::MouseButton>"
WINDOW = "bevy_window::window::Window"
PRIMARY = "bevy_window::window::PrimaryWindow"
LOCATION = "bevy_picking::pointer::PointerLocation"
MOTION = "bevy_input::mouse::AccumulatedMouseMotion"
PROJECTION = "bevy_camera::projection::Projection"


class Controls:
    def __init__(self, game):
        self.game = game
        self.driver = game.driver
        self.keys = set()
        self.buttons = set()

    def key(self, key, down=True):
        self.driver.command("input.keyboard." + ("press" if down else "release"), {"key": key})
        (self.keys.add if down else self.keys.remove)(key)

    def button(self, button="left", down=True):
        self.driver.command("input.pointer." + ("press" if down else "release"), {"button": button})
        (self.buttons.add if down else self.buttons.remove)(button)

    def window(self):
        windows = self.game.query(entities(components(WINDOW), with_types=(PRIMARY,)))["items"]
        assert len(windows) == 1, windows
        value = windows[0]["result"]["components"][0]["value"]
        assert value["status"] == "readable", value
        return value["value"]

    def background(self):
        assert self.window()["focused"] is False, "Spiel hat unerwartet OS-Fokus"

    def named(self, name, *types):
        items = self.game.query(entities(components(NAME, *types), with_types=(NAME, *types)))["items"]
        matches = []
        for item in items:
            values = {c["component"]["type_path"]: c["value"] for c in item["result"]["components"]}
            if values[NAME] == {"status": "readable", "value": name}:
                assert all(v["status"] == "readable" for v in values.values()), values
                matches.append({path: values[path]["value"] for path in types})
        assert len(matches) == 1, (name, matches)
        return matches[0]

    def pointer(self):
        location = self.named("woodpecker-pointer", LOCATION)[LOCATION]["location"]
        return None if location is None else location["position"]

    def move_to(self, position):
        self.driver.command("input.pointer.move_to", {"position": position})

    def move_by(self, delta):
        self.driver.command("input.pointer.move_by", {"delta": delta})

    def screen_position(self, world):
        """Project a known floor point using the camera currently exposed by inspect."""
        camera = self.named("Spielkamera", CAMERA, TRANSFORM, PROJECTION)
        transform = camera[TRANSFORM]
        # Conjugate quaternion: transform a world displacement into camera space.
        qx, qy, qz, qw = transform["rotation"]
        qx, qy, qz = -qx, -qy, -qz
        x, y, z = [a - b for a, b in zip(world, transform["translation"])]
        tx, ty, tz = 2 * (qy * z - qz * y), 2 * (qz * x - qx * z), 2 * (qx * y - qy * x)
        x, y = x + qw * tx + qy * tz - qz * ty, y + qw * ty + qz * tx - qx * tz
        area = camera[PROJECTION]["Orthographic"]["area"]
        low, high = area["min"], area["max"]
        size = self.game.application.data["screenshot"]
        point = [(x - low[0]) / (high[0] - low[0]) * size["width"],
                 (high[1] - y) / (high[1] - low[1]) * size["height"]]
        assert 0 <= point[0] < size["width"] and 0 <= point[1] < size["height"], point
        return point

    def release_all(self):
        for key in sorted(self.keys):
            self.key(key, False)
        for button in sorted(self.buttons):
            self.button(button, False)

    def reset(self):
        self.driver.command("tick.warp.stop", {})
        self.release_all()
        # R is deliberately sent through the same route as user input.
        self.key("r")
        self.game.advance(1)
        self.key("r", False)
        self.game.advance(1)
        self.assert_reset()

    def assert_reset(self):
        player = self.game.resource("player")
        assert player["position"] == [-32.0, 22.0] and player["route"] == [], player
        assert player["walked"] == player["height"] == player["vertical_speed"] == 0, player
        view = self.game.resource("view")
        assert view["mode"] == "Overview" and view["pitch"] == 0.0, view
        assert view.get("eye") is None, view
        assert not self.game.resource("capture")["active"]
        assert not self.game.resource("flow")["menu"]
        assert self.game.resource(KEYS)["pressed"] == []
        assert self.game.resource(BUTTONS)["pressed"] == []
