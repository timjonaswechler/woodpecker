"""Blender geometry for the pilot stair; dimensions come from groundplate/kit.py."""
import bpy
from mathutils import Vector


def build_stair(module, tread_material, rail_material):
    """One mesh with two materials, pivot at the lower entrance, open at both ends."""
    parts = []

    def box(center, size, material):
        bpy.ops.mesh.primitive_cube_add(size=1, location=center)
        obj = bpy.context.object
        obj.dimensions = size
        bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
        obj.data.materials.append(material)
        parts.append(obj)
        return obj

    def beam(start, end, width, material):
        start, end = Vector(start), Vector(end)
        obj = box((start + end) / 2, (width, width, (end - start).length), material)
        obj.rotation_euler = (end - start).to_track_quat("Z", "Y").to_euler()
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)

    for center, half in module.treads():
        box(center, tuple(2 * v for v in half), tread_material)
    for side in (-1, 1):
        x = side * (module.width / 2 - .08)
        # The handrail stays about one metre above the tread, within its footprint.
        beam((x, .08, 1.05), (x, module.run - .08, module.rise + .95), .12,
             rail_material)
        for i in (0, 3, 6, 9, 11):
            y = (i + .5) * module.run / module.steps
            floor = (i + 1) * module.rise / module.steps
            top = 1.05 + (y - .08) / (module.run - .16) * (module.rise - .1)
            box((x, y, (floor + top) / 2), (.12, .12, top - floor), rail_material)

    bpy.ops.object.select_all(action="DESELECT")
    for obj in parts:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = parts[0]
    bpy.ops.object.join()
    obj = bpy.context.object
    obj.name = module.name
    cursor = bpy.context.scene.cursor.location.copy()
    bpy.context.scene.cursor.location = (0, 0, 0)
    bpy.ops.object.origin_set(type="ORIGIN_CURSOR")
    bpy.context.scene.cursor.location = cursor
    obj["signalstation_module"] = "workshop_stair"
    return obj
