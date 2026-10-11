"""Run in Blender to rebuild the demo's fictional low-poly training blaster.

Creates a separate scene, preserving any scene already open in Blender.
Exports only the model to GLB; the .blend keeps the lighting and preview camera.
"""
from pathlib import Path
import math
import bpy
from mathutils import Vector

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "asset" / "custom"
OUT.mkdir(parents=True, exist_ok=True)
(ROOT / "artifacts").mkdir(exist_ok=True)

scene = bpy.data.scenes.new("Demo_Blaster")
bpy.context.window.scene = scene
model = bpy.data.collections.new("Training Blaster")
scene.collection.children.link(model)


def material(name, color, metallic=0.0, emission=0.0):
    result = bpy.data.materials.new(name)
    result.diffuse_color = (*color, 1.0)
    result.use_nodes = True
    shader = result.node_tree.nodes.get("Principled BSDF")
    shader.inputs["Base Color"].default_value = (*color, 1.0)
    shader.inputs["Metallic"].default_value = metallic
    shader.inputs["Roughness"].default_value = 0.42
    shader.inputs["Emission Color"].default_value = (*color, 1.0)
    shader.inputs["Emission Strength"].default_value = emission
    return result


shell = material("Blaster | porcelain", (0.68, 0.77, 0.80))
dark = material("Blaster | graphite", (0.035, 0.065, 0.085), 0.25)
yellow = material("Blaster | training yellow", (1.0, 0.59, 0.07))
teal = material("Blaster | energy", (0.06, 0.85, 0.64), emission=2.0)
orange = material("Blaster | orange muzzle", (1.0, 0.18, 0.025))


def finish(obj, name, mat, bevel=0.0):
    obj.name = name
    for collection in list(obj.users_collection):
        collection.objects.unlink(obj)
    model.objects.link(obj)
    obj.data.materials.append(mat)
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if bevel:
        modifier = obj.modifiers.new("Soft edges", "BEVEL")
        modifier.width = bevel
        modifier.segments = 2
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    return obj


def box(name, position, size, mat, bevel=0.02, tilt=0.0):
    bpy.ops.mesh.primitive_cube_add(size=1, location=position)
    obj = bpy.context.object
    obj.dimensions = size
    obj.rotation_euler.x = tilt
    return finish(obj, name, mat, bevel)


def cylinder(name, position, radius, depth, mat, axis="Y", vertices=12):
    bpy.ops.mesh.primitive_cylinder_add(vertices=vertices, radius=radius, depth=depth, location=position)
    obj = bpy.context.object
    obj.rotation_euler = (math.pi / 2, 0, 0) if axis == "Y" else (0, math.pi / 2, 0)
    return finish(obj, name, mat, 0.008)


box("01 Receiver", (0, 0.18, 0.09), (0.25, 0.57, 0.25), shell, 0.04)
box("02 Rear housing", (0, -0.14, 0.09), (0.27, 0.17, 0.23), dark, 0.035)
box("03 Grip", (0, -0.08, -0.19), (0.16, 0.19, 0.39), dark, 0.025, -0.2)
box("04 Grip heel", (0, -0.12, -0.39), (0.20, 0.22, 0.075), yellow, 0.02)
box("05 Power cell", (0, 0.21, -0.11), (0.20, 0.24, 0.12), yellow, 0.025)
box("06 Trigger guard bottom", (0, 0.13, -0.25), (0.08, 0.24, 0.05), shell, 0.015)
box("07 Trigger guard front", (0, 0.25, -0.18), (0.08, 0.045, 0.15), shell, 0.012)
box("08 Trigger", (0, 0.05, -0.14), (0.045, 0.04, 0.13), orange, 0.01, -0.25)
cylinder("09 Barrel shroud", (0, 0.50, 0.09), 0.135, 0.21, dark)
cylinder("10 Orange muzzle", (0, 0.66, 0.09), 0.15, 0.14, orange)
cylinder("11 Muzzle inset", (0, 0.735, 0.09), 0.105, 0.02, dark)
cylinder("12 Energy emitter", (0, 0.748, 0.09), 0.065, 0.012, teal)
box("13 Top rail", (0, 0.13, 0.235), (0.10, 0.43, 0.045), dark, 0.01)
box("14 Front sight", (0, 0.40, 0.275), (0.065, 0.06, 0.085), yellow, 0.01)
box("15 Sight dot", (0, 0.365, 0.29), (0.025, 0.008, 0.025), teal, 0.003)
for side in [-1, 1]:
    box(f"16 Side panel {side}", (side * 0.13, 0.20, 0.10), (0.025, 0.33, 0.14), yellow, 0.012)
    box(f"17 Charge strip {side}", (side * 0.148, 0.19, 0.11), (0.012, 0.21, 0.035), teal, 0.005)
    cylinder(f"18 Rear fastener {side}", (side * 0.14, -0.14, 0.10), 0.035, 0.015, shell, "X", 8)
for i in range(3):
    box(f"19 Grip rib {i}", (0, -0.195 - i * 0.014, -0.13 - i * 0.075), (0.17, 0.025, 0.022), shell, 0.004)

# +Y in Blender exports as -Z in glTF, matching the game's camera-forward axis.
bpy.ops.object.select_all(action="DESELECT")
for obj in model.objects:
    obj.select_set(True)
bpy.context.view_layer.objects.active = next(iter(model.objects))
bpy.ops.export_scene.gltf(filepath=str(OUT / "training_blaster.glb"), export_format="GLB",
                          use_selection=True, use_active_scene=True, export_apply=True, export_animations=False,
                          export_cameras=False, export_lights=False)

# Preview setup belongs to the Blender file, not the exported game model.
world = bpy.data.worlds.new("Blaster studio")
world.use_nodes = True
world.node_tree.nodes["Background"].inputs[0].default_value = (0.045, 0.065, 0.085, 1)
world.node_tree.nodes["Background"].inputs[1].default_value = 0.45
scene.world = world


def area_light(name, position, power, size):
    data = bpy.data.lights.new(name, "AREA")
    data.energy = power
    data.shape = "DISK"
    data.size = size
    obj = bpy.data.objects.new(name, data)
    scene.collection.objects.link(obj)
    obj.location = position
    obj.rotation_euler = (Vector((0, 0.15, 0)) - obj.location).to_track_quat("-Z", "Y").to_euler()


area_light("Studio key", (2, 1, 3), 180, 3)
area_light("Studio fill", (-2, -1, 1), 110, 2)
area_light("Studio rim", (0, 2, 2), 220, 2)
camera_data = bpy.data.cameras.new("Blaster preview")
camera = bpy.data.objects.new("Blaster preview", camera_data)
scene.collection.objects.link(camera)
camera.location = (1.65, 1.8, 0.9)
camera.rotation_euler = (Vector((0, 0.15, -0.02)) - camera.location).to_track_quat("-Z", "Y").to_euler()
camera_data.type = "ORTHO"
camera_data.ortho_scale = 1.6
scene.camera = camera
scene.render.engine = "CYCLES"
scene.cycles.samples = 24
scene.render.resolution_x = 1000
scene.render.resolution_y = 800
scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = "PNG"
scene.render.filepath = str(ROOT / "artifacts" / "blaster.png")
for area in bpy.context.screen.areas:
    if area.type == "VIEW_3D":
        area.spaces.active.region_3d.view_perspective = "CAMERA"
        area.spaces.active.shading.type = "MATERIAL"
        area.spaces.active.overlay.show_overlays = False
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT / "blender" / "training_blaster.blend"))
print({"model": str(OUT / "training_blaster.glb"), "objects": len(model.objects),
       "triangles": sum(len(p.vertices) - 2 for o in model.objects for p in o.data.polygons)})
