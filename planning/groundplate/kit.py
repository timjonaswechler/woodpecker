"""Pilot module geometry and placement, shared by Blender and the game export.

Plan coordinates: X right, Y down the plan, Z up; distances are metres.
Only architecture and task-object layout anchors use the construction grid.
Decoration, model-local details, player movement and navigation are independent.
"""
from dataclasses import dataclass
from math import isfinite

BUILD_GRID = 1.0


def game_point(point):
    x, y, z = point
    return [round(x - 39, 5), round(z, 5), round(27 - y, 5)]


@dataclass(frozen=True)
class Placement:
    """Construction/task anchor, not a placement constraint for decoration."""
    origin: tuple[float, float, float]
    quarter_turns: int = 0

    def __post_init__(self):
        if (len(self.origin) != 3
                or not all(isfinite(v) and abs(v / BUILD_GRID - round(v / BUILD_GRID)) < 1e-6
                           for v in self.origin)
                or type(self.quarter_turns) is not int
                or not 0 <= self.quarter_turns < 4):
            raise ValueError("construction anchors need whole-metre coordinates and 0..3 quarter turns")

    def point(self, local):
        x, y, z = local
        for _ in range(self.quarter_turns):
            x, y = -y, x
        return (self.origin[0] + x, self.origin[1] + y, self.origin[2] + z)

    def anchor(self, local):
        """Validate layout sockets separately from unconstrained mesh details."""
        return Placement(self.point(local)).origin

    def box(self, name, center, half):
        hx, hy, hz = half
        if self.quarter_turns % 2:
            hx, hy = hy, hx
        return dict(name=name, center=game_point(self.point(center)), half=[hx, hz, hy])


@dataclass(frozen=True)
class StairModule:
    placement: Placement
    width: float = 3.0
    run: float = 8.0
    rise: float = 3.0
    steps: int = 12
    name: str = "Workshop stair module"

    asset = "meshes/environment/workshop_stair.gltf"

    def treads(self):
        """Local solid steps: center and half extents in plan coordinates."""
        for i in range(self.steps):
            height = (i + 1) * self.rise / self.steps
            yield ((0, (i + .5) * self.run / self.steps, height / 2),
                   (self.width / 2, self.run / self.steps / 2, height / 2))

    def surfaces(self):
        for center, half in self.treads():
            box = self.placement.box("", center, half)
            yield dict(center=box["center"][::2], half=box["half"][::2],
                       height=round(box["center"][1] + box["half"][1], 5),
                       bottom=self.placement.origin[2])

    def rails(self):
        # Separate narrow proxies follow the incline, not a tall invisible wall.
        for side in (-1, 1):
            for i in range(self.steps):
                height = (i + 1) * self.rise / self.steps
                yield self.placement.box(
                    f"{self.name} rail {side}/{i}",
                    (side * (self.width / 2 - .08), (i + .5) * self.run / self.steps,
                     height + .5),
                    (.06, self.run / self.steps / 2, .5))

    def metadata(self):
        return dict(name=self.name, asset=self.asset,
                    origin=game_point(self.placement.origin),
                    quarter_turns=self.placement.quarter_turns,
                    exit=game_point(self.placement.anchor((0, self.run, self.rise))))


@dataclass(frozen=True)
class WorkshopIsland:
    """An aligned input / press / output row with a shared free front aisle."""
    placement: Placement

    def __post_init__(self):
        if self.placement.quarter_turns:
            raise ValueError("the pilot island uses the unrotated workshop frontage")

    STATIONS = {
        "part": (0, (1.1, 1.6, 1.1)),
        "craft": (2, (1.3, 1.5, .9)),
        "output": (5, (2, 1.75, 1.5)),
    }

    def metadata(self):
        return {name: dict(base=game_point(self.point(name)), size=list(size),
                           approach=game_point(self.placement.anchor((x, -2, 0))))
                for name, (x, size) in self.STATIONS.items()}

    def point(self, station):
        x, _ = self.STATIONS[station]
        return self.placement.anchor((x, 0, 0))


WORKSHOP_STAIR = StairModule(Placement((12, 23, 0)))
WORKSHOP = WorkshopIsland(Placement((3, 31, 0)))
