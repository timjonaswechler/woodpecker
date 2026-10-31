"""Closed secret architecture, two independent portals and physical reward alcoves."""
from kit import game_point

# Bounds and portal construction sockets use the same whole-metre plan as rooms.
# Portals: axis, plane, low/high opening, inward sign, slide sign, outside operation.
PASSAGES = [
    dict(id="G1", floor=0, volumes=[(18, 0, 50, 10)],
         portals=[("v", 18, 3, 6, 1, 1, True), ("v", 50, 3, 6, -1, 1, True)],
         partitions=[("h", 3, 30, 35), ("v", 30, 0, 3), ("v", 38, 0, 3)],
         reward=(34, 2, 0)),
    dict(id="G2", floor=3, volumes=[(17, 44, 24, 49), (24, 45, 51, 48)],
         portals=[("h", 44, 18, 21, 1, 1, True), ("v", 51, 45, 48, -1, -1, False)],
         partitions=[("v", 21, 44, 47), ("h", 46, 17, 19)], reward=(18, 48, 3)),
    dict(id="G3", floor=0, volumes=[(3, 38, 10, 43)],
         portals=[("h", 38, 5, 8, 1, 1, True), ("h", 43, 4, 7, -1, 1, False)],
         partitions=[("h", 40, 5, 10)], reward=(8, 42, 0)),
]


def subtract_rect(rect, hole):
    x0, y0, x1, y1 = rect
    a, b, c, d = max(x0, hole[0]), max(y0, hole[1]), min(x1, hole[2]), min(y1, hole[3])
    if a >= c or b >= d:
        return [rect]
    return [r for r in [(x0, y0, a, y1), (c, y0, x1, y1),
                        (a, y0, c, b), (a, d, c, y1)] if r[0] < r[2] and r[1] < r[3]]


def vestibule(portal):
    axis, plane, lo, hi, inward, _, _ = portal
    if axis == "v":
        return (min(plane, plane + inward), lo, max(plane, plane + inward), hi)
    return (lo, min(plane, plane + inward), hi, max(plane, plane + inward))


def roofs(secret):
    pieces = list(secret["volumes"])
    for portal in secret["portals"]:
        pieces = [r for piece in pieces for r in subtract_rect(piece, vestibule(portal))]
    yield from ((r, None) for r in pieces)
    yield from ((vestibule(p), i) for i, p in enumerate(secret["portals"]))


def shells(secret):
    for x0, y0, x1, y1 in secret["volumes"]:
        for axis, coord, lo, hi in [("h", y0, x0, x1), ("h", y1, x0, x1),
                                     ("v", x0, y0, y1), ("v", x1, y0, y1)]:
            holes = [(p[2], p[3]) for p in secret["portals"] if p[:2] == (axis, coord)]
            for a, b, c, d in secret["volumes"]:
                if (a, b, c, d) == (x0, y0, x1, y1): continue
                if axis == "h" and coord in (b, d): holes.append((a, c))
                if axis == "v" and coord in (a, c): holes.append((b, d))
            pieces = [(lo, hi)]
            for a, b in holes:
                pieces = [part for l, h in pieces for part in
                          ([(l, h)] if b <= l or a >= h else
                           ([(l, a)] if l < a else []) + ([(b, h)] if b < h else []))]
            yield from ((axis, coord, a, b) for a, b in pieces)


def on_boundary(segment, secret):
    axis, coord, lo, hi = segment
    return any((axis == "h" and coord in (y0, y1) and x0 <= lo < hi <= x1)
               or (axis == "v" and coord in (x0, x1) and y0 <= lo < hi <= y1)
               for x0, y0, x1, y1 in secret["volumes"])


def metadata():
    result = []
    for i, secret in enumerate(PASSAGES):
        portals = []
        for j, (axis, plane, lo, hi, inward, slide, outside) in enumerate(secret["portals"]):
            x, y = (plane, (lo + hi) / 2) if axis == "v" else ((lo + hi) / 2, plane)
            direction = [inward, 0, 0] if axis == "v" else [0, 0, -inward]
            portals.append(dict(name=f"Secret portal {i}/{j}",
                anchor=game_point((plane, lo, secret["floor"]) if axis == "v" else (lo, plane, secret["floor"])),
                center=game_point((x, y, secret["floor"] + 1.25)),
                half=[.35, 1.25, (hi-lo)/2] if axis == "v" else [(hi-lo)/2, 1.25, .35],
                inward=direction, slide=[0, 0, -slide*4] if axis == "v" else [slide*4, 0, 0],
                outside=outside))
        result.append(dict(id=secret["id"], floor=secret["floor"],
                           volumes=[list(v) for v in secret["volumes"]], portals=portals))
    return result
