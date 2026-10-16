"""Create the demo's low-poly research station kit without touching the blaster.

Run: Blender --background --python-exit-code 1 --python create_station.py
All dimensions in the builders use glTF/game coordinates: X right, Y up, Z front.
Exports six .gltf + .bin models, an editable .blend and a rendered contact sheet.
"""
from pathlib import Path
import json
import math
import struct
import sys

import bpy
from mathutils import Matrix, Quaternion, Vector

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gltf_export import export_model

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "asset" / "custom" / "station"
OUT.mkdir(parents=True, exist_ok=True)
(ROOT / "artifacts").mkdir(exist_ok=True)

# This script owns a fresh file. Never load or overwrite training_blaster.blend.
bpy.ops.wm.read_factory_settings(use_empty=True)
source = bpy.context.scene
source.name = "Station source models"
current = None


def xyz(value):
    return Vector((value[0], -value[2], value[1]))


def material(name, rgb, metallic=0.0, emission=0.0):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*rgb, 1)
    mat.use_nodes = True
    node = mat.node_tree.nodes.get("Principled BSDF")
    node.inputs["Base Color"].default_value = (*rgb, 1)
    node.inputs["Metallic"].default_value = metallic
    node.inputs["Roughness"].default_value = 0.58
    node.inputs["Emission Color"].default_value = (*rgb, 1)
    node.inputs["Emission Strength"].default_value = emission
    return mat


SHELL = material("Station | warm porcelain", (0.78, 0.77, 0.66))
DARK = material("Station | deep petrol", (0.034, 0.095, 0.105), 0.25)
CORAL = material("Station | coral warning", (0.96, 0.205, 0.085))
ENERGY = material("Station | turquoise power", (0.055, 0.76, 0.64), emission=0.8)
SCREEN = material("Station | screen glass", (0.022, 0.15, 0.17), 0.15)
METAL = material("Station | worn metal", (0.20, 0.29, 0.29), 0.45)
SAND = material("Cliff | sandstone", (0.38, 0.32, 0.255))
ROCK = material("Cliff | shadow", (0.25, 0.25, 0.235))
LIGHT_ROCK = material("Cliff | pale stratum", (0.49, 0.43, 0.33))


def finish(obj, name, mat, bevel=0):
    obj.name = name
    for collection in list(obj.users_collection):
        collection.objects.unlink(obj)
    current.objects.link(obj)
    obj.data.materials.append(mat)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel:
        mod = obj.modifiers.new("Single cut corners", "BEVEL")
        mod.width = bevel
        mod.segments = 1
        bpy.ops.object.modifier_apply(modifier=mod.name)
    return obj


def box(name, position, size, mat, bevel=0.012):
    bpy.ops.mesh.primitive_cube_add(size=1, location=xyz(position))
    obj = bpy.context.object
    obj.dimensions = (size[0], size[2], size[1])
    return finish(obj, name, mat, bevel)


def cylinder(name, position, radius, height, mat, vertices=8):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=height,
                                      location=xyz(position))
    return finish(bpy.context.object, name, mat)


def beam(name, start, end, width, mat):
    delta = xyz(end) - xyz(start)
    bpy.ops.mesh.primitive_cube_add(size=1, location=(xyz(start) + xyz(end)) / 2)
    obj = bpy.context.object
    obj.dimensions = (width, width, delta.length)
    obj.rotation_euler = delta.to_track_quat("Z", "Y").to_euler()
    return finish(obj, name, mat, min(0.008, width / 8))


def energy_cell():
    cylinder("Cell base", (0, .055, 0), .175, .11, DARK)
    cylinder("Charged core", (0, .305, 0), .145, .42, ENERGY)
    cylinder("Cell cap", (0, .56, 0), .175, .09, SHELL)
    cylinder("Contact", (0, .6275, 0), .09, .045, CORAL)
    for x in (-.112, .112):
        for z in (-.112, .112):
            box("Protective cage", (x, .315, z), (.045, .43, .045), SHELL, .007)
    cylinder("Charge band", (0, .16, 0), .157, .035, CORAL)


def relay_console():
    box("Weighted foot", (0, .06, 0), (.86, .12, .68), DARK, .025)
    box("Pedestal", (0, .415, 0), (.66, .59, .42), SHELL, .035)
    box("Head casing", (0, .945, 0), (1.1, .51, .7), SHELL, .045)
    box("Screen recess", (-.105, 1.015, .347), (.735, .285, .024), DARK)
    box("Display", (-.13, 1.02, .363), (.61, .21, .012), SCREEN, .003)
    for row, width in enumerate((.34, .46, .22)):
        box("Display status line", (-.375 + width / 2, 1.075 - row * .05, .371),
            (width, .015, .009), ENERGY, .002)
    box("Confirm key", (.39, .985, .358), (.12, .11, .035), CORAL, .016)
    box("Key indicator", (.39, 1.105, .361), (.085, .025, .025), ENERGY, .004)
    box("Cell socket frame", (0, .53, .216), (.30, .33, .035), DARK, .025)
    box("Cell socket interior", (0, .53, .239), (.18, .24, .014), SCREEN, .018)
    box("Socket light", (0, .345, .231), (.22, .025, .015), ENERGY, .003)
    for x in (-.46, .46):
        box("Warning corner", (x, .77, .352), (.095, .035, .015), CORAL, .004)
    for y in (.23, .285):
        box("Pedestal vent", (0, y, .217), (.36, .019, .016), METAL, .003)


def signal_tower():
    box("Tower foundation", (0, .08, 0), (1.5, .16, 1.5), DARK, .045)
    for x in (-.54, .54):
        for z in (-.54, .54):
            box("Anchor foot", (x, .20, z), (.27, .08, .27), CORAL, .016)
            beam("Splayed leg", (x, .22, z), (x * .58, 2.4, z * .58), .13, SHELL)
    for z in (-.51, .51):
        beam("Lower cross brace", (-.5, .4, z), (.36, 1.8, z * .7), .085, METAL)
        beam("Lower cross brace", (.5, .4, z), (-.36, 1.8, z * .7), .085, METAL)
    for x in (-.51, .51):
        beam("Side cross brace", (x, .4, -.5), (x * .7, 1.8, .36), .085, METAL)
    box("Equipment deck", (0, 2.40, 0), (1.22, .18, 1.22), DARK, .025)
    box("Transmitter cabinet", (0, 2.92, 0), (.66, .86, .60), SHELL, .06)
    box("Transmitter face", (0, 2.91, .305), (.47, .60, .028), DARK, .018)
    for i in range(3):
        box("Transmitter status", (0, 2.71 + i * .19, .326), (.30, .06, .014), ENERGY, .006)
    box("Mast base", (0, 3.42, 0), (.75, .15, .75), CORAL, .015)
    box("Main mast", (0, 4.3, 0), (.22, 1.62, .22), SHELL, .015)
    box("Antenna spar", (0, 4.63, 0), (1.4, .11, .11), METAL, .008)
    for x in (-.61, .61):
        box("Antenna panel", (x, 4.75, 0), (.18, .75, .29), SHELL, .025)
        box("Panel warning tip", (x, 5.08, .152), (.14, .08, .025), CORAL, .006)
    cylinder("Beacon base", (0, 5.17, 0), .24, .16, DARK)
    cylinder("Beacon lamp", (0, 5.375, 0), .17, .25, ENERGY)
    cylinder("Beacon cap", (0, 5.525, 0), .24, .05, CORAL)
    cylinder("Aerial", (0, 5.675, 0), .025, .25, METAL)


def bridge_module():
    box("Bridge deck", (0, -.135, 0), (2.6, .21, 4.0), DARK, .015)
    for i in range(10):
        box("Deck tread", (0, -.016, -1.8 + i * .4), (2.16, .032, .37), METAL, .008)
    for x in (-1.21, 1.21):
        box("Deck curb", (x, -.015, 0), (.18, .03, 4), SHELL, .005)
        box("Underside stringer", (x, -.3, 0), (.14, .18, 4), METAL, .008)
        for z in (-1.85, 0, 1.85):
            box("Rail stanchion", (x, .43, z), (.12, .86, .12), SHELL, .012)
            box("Rail warning", (x, .70, z), (.13, .12, .13), CORAL, .008)
        box("Hand rail", (x, .87, 0), (.15, .10, 4), SHELL, .015)
        box("Lower rail", (x, .37, 0), (.08, .065, 4), METAL, .009)
        for z in (-1.45, 1.45):
            box("Path beacon", (x, .075, z), (.10, .12, .20), ENERGY, .009)
    for z in (-1.95, 1.95):
        box("Threshold", (0, -.011, z), (2.13, .022, .08), CORAL, .006)


def energy_gate():
    for x in (-1.52, 1.52):
        box("Gate foot", (x, .065, 0), (.36, .13, .40), DARK, .012)
        box("Gate upright", (x, 1.47, 0), (.30, 2.68, .34), SHELL, .025)
        box("Energy conductor", (x - math.copysign(.158, x), 1.54, 0),
            (.02, 2.15, .14), ENERGY, .003)
        for y in (.3, 2.55):
            box("Upright warning", (x, y, .18), (.30, .12, .03), CORAL, .005)
    box("Gate lintel", (0, 2.825, 0), (3.4, .35, .40), DARK, .025)
    box("Lintel shell", (0, 2.85, .18), (2.7, .20, .04), SHELL, .008)
    box("Access indicator", (0, 2.84, .206), (.50, .08, .018), ENERGY, .008)
    for x in (-.72, .72):
        box("Lintel warning", (x, 2.84, .206), (.20, .08, .018), CORAL, .005)


def cliff_module():
    # Eight points per horizontal band keep the top exactly square and tileable.
    outline = [(-2, -2), (0, -2), (2, -2), (2, 0), (2, 2), (0, 2), (-2, 2), (-2, 0)]
    middle = [(-1.93, -1.84), (-.17, -1.96), (1.81, -1.77), (1.97, .2),
              (1.83, 1.94), (.21, 1.79), (-1.85, 1.82), (-1.96, -.12)]
    bottom = [(-1.35, -1.42), (-.12, -1.55), (1.44, -1.32), (1.61, -.04),
              (1.32, 1.46), (.02, 1.59), (-1.52, 1.28), (-1.59, .07)]
    verts = [xyz((x, 0, z)) for x, z in outline]
    verts += [xyz((x, -.78 - (i % 3) * .09, z)) for i, (x, z) in enumerate(middle)]
    verts += [xyz((x, -3.5 + (i % 3) * .09, z)) for i, (x, z) in enumerate(bottom)]
    faces = [tuple(reversed(range(8))), tuple(range(16, 24))]
    for layer in range(2):
        for i in range(8):
            a, b = layer * 8 + i, layer * 8 + (i + 1) % 8
            faces.extend(((a, b, b + 8), (a, b + 8, a + 8)))
    mesh = bpy.data.meshes.new("Cliff faceted geometry")
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new("Cliff module", mesh)
    current.objects.link(obj)
    for mat in (SAND, ROCK, LIGHT_ROCK):
        mesh.materials.append(mat)
    for i, poly in enumerate(mesh.polygons):
        poly.material_index = 2 if i == 0 or 2 <= i < 18 else (1 if i % 3 else 0)
    bpy.context.view_layer.objects.active = obj
    obj.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT")
    bpy.ops.mesh.select_all(action="SELECT")
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode="OBJECT")


BUILDERS = {
    "energy_cell": energy_cell,
    "relay_console": relay_console,
    "signal_tower": signal_tower,
    "bridge_module": bridge_module,
    "energy_gate": energy_gate,
    "cliff_module": cliff_module,
}


def exported_bounds(path):
    """Read exported vertex data and transforms, not approximate object bounds."""
    raw = path.read_bytes()
    doc = json.loads(raw)
    buffers = [(path.parent / buffer["uri"]).read_bytes() for buffer in doc["buffers"]]
    minimum, maximum = Vector((math.inf,) * 3), Vector((-math.inf,) * 3)
    triangles = 0
    for scene in doc["scenes"]:
        pending = [(idx, Matrix.Identity(4)) for idx in scene["nodes"]]
        while pending:
            idx, parent = pending.pop()
            node = doc["nodes"][idx]
            if "matrix" in node:
                local = Matrix([node["matrix"][i:i + 4] for i in range(0, 16, 4)]).transposed()
            else:
                q = node.get("rotation", (0, 0, 0, 1))
                local = Matrix.LocRotScale(Vector(node.get("translation", (0, 0, 0))),
                                          Quaternion((q[3], q[0], q[1], q[2])),
                                          Vector(node.get("scale", (1, 1, 1))))
            transform = parent @ local
            pending.extend((child, transform) for child in node.get("children", []))
            if "mesh" not in node:
                continue
            for primitive in doc["meshes"][node["mesh"]]["primitives"]:
                accessor = doc["accessors"][primitive["attributes"]["POSITION"]]
                view = doc["bufferViews"][accessor["bufferView"]]
                binary = buffers[view["buffer"]]
                assert accessor["componentType"] == 5126 and accessor["type"] == "VEC3"
                start = view.get("byteOffset", 0) + accessor.get("byteOffset", 0)
                stride = view.get("byteStride", 12)
                for i in range(accessor["count"]):
                    point = transform @ Vector(struct.unpack_from("<3f", binary, start + i * stride))
                    for axis in range(3):
                        minimum[axis] = min(minimum[axis], point[axis])
                        maximum[axis] = max(maximum[axis], point[axis])
                triangles += doc["accessors"][primitive["indices"]]["count"] // 3
                assert "NORMAL" in primitive["attributes"]
    assert not doc.get("cameras") and not doc.get("images")
    assert not doc.get("extensions", {}).get("KHR_lights_punctual")
    return {
        "minimum": [round(x, 5) for x in minimum],
        "maximum": [round(x, 5) for x in maximum],
        "dimensions": [round(x, 5) for x in maximum - minimum],
        "triangles": triangles,
        "bytes": len(raw) + sum(len(buffer) for buffer in buffers),
    }


manifest = {}
models = {}
for name, build in BUILDERS.items():
    current = bpy.data.collections.new(name)
    source.collection.children.link(current)
    bpy.ops.object.select_all(action="DESELECT")
    build()
    bpy.ops.object.select_all(action="DESELECT")
    for obj in current.objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = next(iter(current.objects))
    if len(current.objects) > 1:
        bpy.ops.object.join()
    joined = bpy.context.object
    joined.name = name
    source.cursor.location = (0, 0, 0)
    bpy.ops.object.origin_set(type="ORIGIN_CURSOR")
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    path = OUT / f"{name}.gltf"
    export_model(path, use_selection=True, use_active_scene=True, export_apply=True,
                 export_animations=False, export_cameras=False, export_lights=False)
    manifest[name] = exported_bounds(path)
    models[name] = current

(OUT / "dimensions.json").write_text(json.dumps(manifest, indent=2) + "\n")

# A separate preview scene shows collection instances. Source geometry stays at
# its export origin, and no preview furniture enters the glTF exports.
preview = bpy.data.scenes.new("Station kit overview")
bpy.context.window.scene = preview
world = bpy.data.worlds.new("Station preview studio")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (.13, .18, .20, 1)
world.node_tree.nodes["Background"].inputs[1].default_value = .55
preview.world = world

placements = {
    "signal_tower": (-5.4, 0, -2.5),
    "energy_gate": (-.9, 0, -2.5),
    "cliff_module": (4.6, 3.5, -2.8),
    "energy_cell": (-5.4, 0, 3.2),
    "relay_console": (-1.65, 0, 3.0),
    "bridge_module": (3.5, .40, 3.0),
}
for name, location in placements.items():
    instance = bpy.data.objects.new(name + " preview", None)
    instance.instance_type = "COLLECTION"
    instance.instance_collection = models[name]
    preview.collection.objects.link(instance)
    instance.location = xyz(location)
    if name == "energy_cell":
        instance.scale = (2.6, 2.6, 2.6)

current = bpy.data.collections.new("Preview studio only")
preview.collection.children.link(current)
studio_floor = material("Preview | floor", (.075, .12, .135))
box("Studio floor", (0, -.10, 0), (200, .10, 200), studio_floor, 0)

def label(value, position, size=.27):
    curve = bpy.data.curves.new("Preview label", "FONT")
    curve.body = value
    curve.align_x = "CENTER"
    curve.size = size
    curve.extrude = 0
    obj = bpy.data.objects.new(value, curve)
    current.objects.link(obj)
    obj.location = xyz(position)
    curve.materials.append(SHELL)

for name, position in placements.items():
    label(name.replace("_", " ").upper(), (position[0], .016, position[2] + (2.45 if name in ("bridge_module", "cliff_module") else 1.1)), .23)
label("CELL SHOWN AT 2.6x  /  OTHER MODELS AT GAME SCALE", (0, .016, 6.35), .20)

for name, location, power, size in (
    ("Key", (-4, 13, 8), 2200, 9),
    ("Fill", (9, 9, 4), 1200, 8),
    ("Rim", (0, 10, -8), 2100, 7),
):
    data = bpy.data.lights.new(name, "AREA")
    data.energy = power
    data.shape = "DISK"
    data.size = size
    obj = bpy.data.objects.new(name, data)
    current.objects.link(obj)
    obj.location = xyz(location)
    obj.rotation_euler = (xyz((0, 1, 0)) - obj.location).to_track_quat("-Z", "Y").to_euler()

camera_data = bpy.data.cameras.new("Overview camera")
camera = bpy.data.objects.new("Overview camera", camera_data)
current.objects.link(camera)
camera.location = xyz((11, 17, 19))
camera.rotation_euler = (xyz((0, 1.2, -.4)) - camera.location).to_track_quat("-Z", "Y").to_euler()
camera_data.type = "ORTHO"
camera_data.ortho_scale = 20.6
preview.camera = camera
preview.render.engine = "CYCLES"
preview.cycles.samples = 40
preview.cycles.use_denoising = True
preview.render.resolution_x = 1500
preview.render.resolution_y = 1150
preview.render.resolution_percentage = 100
preview.render.image_settings.file_format = "PNG"
preview.render.filepath = str(ROOT / "artifacts" / "station-assets.png")
preview.view_settings.view_transform = "AgX"
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type == "VIEW_3D":
            area.spaces.active.region_3d.view_perspective = "CAMERA"
            area.spaces.active.shading.type = "MATERIAL"
            area.spaces.active.overlay.show_overlays = False
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT / "blender" / "station_kit.blend"))
bpy.ops.render.render(write_still=True)
print("STATION ASSETS VERIFIED " + json.dumps(manifest, sort_keys=True))
