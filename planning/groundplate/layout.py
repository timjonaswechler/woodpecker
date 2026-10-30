"""Shared geometry for the expanded planning model and PDF. Units are metres."""

from kit import WORKSHOP, WORKSHOP_STAIR

PLATE = (-2, -2, 80, 56)
UPPER = 3.0
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
# Horizontal walls use their y coordinate, vertical walls their x coordinate.
DOORS = [
    ("h",18,7,11), ("v",18,13,16.5), ("v",18,25,29),
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
    ("E gallery", (51,40,62,47.5), "main"),
    ("F gallery", (4,40,22,44), "main"),
    ("G2 upper room", (17,44,24,49), "secret"),
    ("G2 bridge F E", (24,44.5,51,47.5), "secret"),
    ("H signal balcony", (64,10,76,15), "main"),
]
# x centre, y at ground, y at upper landing.
STAIRS = [(WORKSHOP_STAIR.placement.origin[0], WORKSHOP_STAIR.placement.origin[1],
           WORKSHOP_STAIR.placement.origin[1] + WORKSHOP_STAIR.run),
          (42,23,31),(71,23,31),
          (59,53.5,47.5),(8,51,44),(69,3,10)]
PARKOUR = [(26,16,.6),(27,19,1.2),(29,22,1.8),(31,25,2.4),(32,28,3.0)]
COINS = [(5,5,0),(11,9,0),(14,15,0),
         (4,23,0),(15,20,0),(15,34,3),
         (23,14,0),(38,18,0),(31,25,2.4),
         (56,21,0),(74,28,0),(63,33,3),
         (67,41,0),(73,51,0),(55,42,3),
         (4,47,0),(19,51,0),(13,42,3),
         (28,41,0),(45,42,0),(35,46,3),
         (57,5,0),(74,5,0),(66,12,3)]
BLUEPRINTS = [(34,5,0),(21,47,3),(6.5,41,0),
              (34,34,3),(74,21,0),(46,51,0)]
GEARS = [WORKSHOP.point("part"),(66,26,0)]
CELLS = [(*WORKSHOP.point("output")[:2], 1.1),(71,42,1.1),(17,48,1.1)]
LABELS = [("A",8,8),("B",7,20.5),("C",35,15),("D",63,25),
          ("E",75,43),("F",13,49.5),("G",38,41),("H",59,13)]
SECRETS = [("G1",34,5,0),("G2",21,47,3),("G3",6.5,41,0)]


def walls():
    """Merge shared room boundaries and cut real openings for every doorway."""
    axes = {}
    for bounds in [room[2] for room in ROOMS] + [MAINTENANCE]:
        x0,y0,x1,y1 = bounds
        for key,interval in [(('h',y0),(x0,x1)),(('h',y1),(x0,x1)),
                             (('v',x0),(y0,y1)),(('v',x1),(y0,y1))]:
            axes.setdefault(key,[]).append(interval)
    result = []
    for (axis,coordinate),intervals in axes.items():
        merged = []
        for lo,hi in sorted(intervals):
            if merged and lo <= merged[-1][1]:
                merged[-1] = (merged[-1][0],max(merged[-1][1],hi))
            else:
                merged.append((lo,hi))
        for door_axis,door_coord,dlo,dhi in DOORS:
            if (door_axis,door_coord) != (axis,coordinate):
                continue
            pieces = []
            for lo,hi in merged:
                if dhi <= lo or dlo >= hi: pieces.append((lo,hi))
                else:
                    if lo < dlo: pieces.append((lo,dlo))
                    if dhi < hi: pieces.append((dhi,hi))
            merged = pieces
        result.extend((axis,coordinate,lo,hi) for lo,hi in merged)
    return result
