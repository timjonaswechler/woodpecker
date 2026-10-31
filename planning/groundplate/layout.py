"""Whole-metre construction layout; local mesh detail and decoration stay free."""
from kit import Placement, WallModule, WORKSHOP, WORKSHOP_STAIR, STAIR_MODULES, station_metadata
from passages import PASSAGES

PLATE = (-2, -2, 80, 56)
UPPER = 3.0
STATIONS = station_metadata()


def station_point(key, field="base"):
    x, height, z = STATIONS[key][field]
    return (x+39, 27-z, height)

ROOMS = [
    ("A", "Eingang", (0, 0, 18, 18), "E9D7A8"),
    ("B", "Werkhalle", (0, 18, 18, 38), "BECBB3"),
    ("C", "Zentrale Halle", (18, 10, 50, 38), "BBD4CF"),
    ("D", "Lagerhalle", (50, 18, 78, 38), "DFB9A7"),
    ("E", "Gewächshaus", (50, 38, 78, 54), "BED5A8"),
    ("F", "Archiv", (0, 38, 24, 54), "C7BFDB"),
    ("G", "Übungsraum", (24, 38, 50, 54), "E5C8A0"),
    ("H", "Generatorraum", (50, 0, 78, 18), "B4CED2"),
]
MAINTENANCE = (18, 0, 50, 10)
DOORS = [
    ("h",18,7,11), ("v",18,13,17), ("v",18,25,29),
    ("v",50,25,29), ("v",50,12,16), ("h",38,34,38),
    ("h",38,66,70), ("h",18,65,69), ("v",24,49,52),
    ("v",50,49,52), ("v",18,3,6), ("v",50,3,6), ("h",38,5,8),
]
DECKS = [
    ("B gallery", (9,31,18,36), "main"),
    ("Bridge B C", (18,32,29,35), "main"),
    ("C gallery", (29,31,47,37), "main"),
    ("Bridge C D", (47,32,55,35), "main"),
    ("D gallery", (55,31,75,36), "main"),
    ("Bridge D E", (54,36,57,40), "main"),
    ("E gallery", (51,40,57,48), "main"),
    ("E stair landing", (57,40,62,44), "main"),
    ("F gallery", (4,40,22,44), "main"),
    ("G2 upper room", (17,44,24,49), "secret"),
    ("G2 bridge F E", (24,45,51,48), "secret"),
    ("H signal balcony", (64,10,76,15), "main"),
]
STAIRS = [(s.placement.origin[0], s.placement.origin[1],
           s.placement.anchor((0, s.run, s.rise))[1]) for s in STAIR_MODULES]
PARKOUR = [(26,16,.6),(27,19,1.2),(29,22,1.8),(31,25,2.4),(32,28,3.0)]
COINS = [(5,5,0),(11,9,0),(14,15,0), (4,23,0),(15,20,0),(15,34,3),
         (23,14,0),(38,18,0),(31,25,2.4), (56,21,0),(74,28,0),(63,33,3),
         (67,41,0),(73,51,0),(55,42,3), (4,47,0),(19,51,0),(13,42,3),
         (28,41,0),(45,42,0),(35,46,3), (57,5,0),(74,5,0),(66,12,3)]
BLUEPRINTS = [p["reward"] for p in PASSAGES] + [(34,34,3),(74,21,0),(46,51,0)]
GEARS = [station_point(key) for key in ("part0", "part1")]
CELLS = [(x,y,z+1.1) for x,y,z in (station_point(key) for key in ("cell0","cell1","cell2"))]
LABELS = [("A",8,8),("B",7,20.5),("C",35,15),("D",63,25),
          ("E",75,43),("F",13,49.5),("G",38,41),("H",59,13)]
SECRETS = [(p["id"], *p["reward"]) for p in PASSAGES]


def subtract_interval(pieces, lo, hi):
    result = []
    for a, b in pieces:
        if hi <= a or lo >= b:
            result.append((a, b))
        else:
            if a < lo: result.append((a, lo))
            if hi < b: result.append((hi, b))
    return result


def gallery_rails():
    """Exposed upper-floor edges only; joins and complete 3 m stair mouths are open."""
    result = []
    for _, (x0, y0, x1, y1), kind in DECKS:
        if kind != "main":
            continue  # Secret rooms have full-height walls instead of railings.
        for axis, coord, lo, hi in [("h", y0, x0, x1), ("h", y1, x0, x1),
                                     ("v", x0, y0, y1), ("v", x1, y0, y1)]:
            pieces = [(lo, hi)]
            for _, other, _ in DECKS:
                if other == (x0, y0, x1, y1): continue
                a, b, c, d = other
                touches = (axis == "h" and (b == coord or d == coord)
                           or axis == "v" and (a == coord or c == coord))
                if touches:
                    pieces = subtract_interval(pieces, a if axis == "h" else b,
                                                c if axis == "h" else d)
            for stair in STAIR_MODULES:
                sx, sy, _ = stair.placement.anchor((0, stair.run, stair.rise))
                if axis == "h" and sy == coord:
                    pieces = subtract_interval(pieces, sx-stair.width/2, sx+stair.width/2)
            if axis == "h" and coord == 31:
                pieces = subtract_interval(pieces, 30.5, 33.5)  # Optional parkour landing.
            if axis == 'h' and coord in (40, 44):
                pieces = [part for a,b in pieces for part in
                          ([(a,10),(10,b)] if a<10<b else [(a,b)])]
            result.extend((axis, coord, a, b) for a, b in pieces if b > a)
    return result


def walls():
    """Merge boundaries, cut openings and assemble reusable 1–3 m wall panels."""
    axes = {}
    for bounds in [room[2] for room in ROOMS] + [MAINTENANCE]:
        x0,y0,x1,y1 = bounds
        Placement((x0, y0, 0)); Placement((x1, y1, 0))
        for key,interval in [(('h',y0),(x0,x1)),(('h',y1),(x0,x1)),
                             (('v',x0),(y0,y1)),(('v',x1),(y0,y1))]:
            axes.setdefault(key,[]).append(interval)
    result = []
    for (axis,coordinate),intervals in axes.items():
        merged = []
        for lo,hi in sorted(intervals):
            if merged and lo <= merged[-1][1]:
                merged[-1] = (merged[-1][0],max(merged[-1][1],hi))
            else: merged.append((lo,hi))
        for da, dc, dlo, dhi in DOORS:
            if (da, dc) == (axis, coordinate):
                merged = subtract_interval(merged, dlo, dhi)
        # Split at secret boundaries so a cutaway cannot hide the neighbouring room.
        cuts = {v for p in PASSAGES for b in p["volumes"]
                for v in (b[0], b[2]) if axis == "h"}
        cuts |= {v for p in PASSAGES for b in p["volumes"]
                 for v in (b[1], b[3]) if axis == "v"}
        for lo, hi in merged:
            boundaries = [lo] + sorted(c for c in cuts if lo < c < hi) + [hi]
            for a, b in zip(boundaries, boundaries[1:]):
                while a < b:
                    length = int(min(3, b-a))
                    origin = (a, coordinate, 0) if axis == "h" else (coordinate, a, 0)
                    result.append(WallModule(Placement(origin, 0 if axis == "h" else 1), length).segment())
                    a += length
    return result
