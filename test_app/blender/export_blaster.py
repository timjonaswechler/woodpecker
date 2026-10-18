"""Export the edited training_blaster.blend without rebuilding the model.

Run with Blender:
  blender --background test_app/blender/training_blaster.blend \
      --python test_app/blender/export_blaster.py
Append -- --preview to also render artifacts/blaster.png.
"""
from pathlib import Path
import sys
import bpy

sys.path.insert(0, str(Path(__file__).resolve().parent))
from gltf_export import export_model


root = Path(__file__).resolve().parents[1]
scene = bpy.data.scenes.get("Demo_Blaster")
model = bpy.data.collections.get("Training Blaster")
if scene is None or model is None or not any(o.type == "MESH" for o in model.all_objects):
    raise RuntimeError("Open training_blaster.blend with its Training Blaster collection first.")

previous_scene = bpy.context.window.scene
bpy.context.window.scene = scene
previous_selection = list(bpy.context.selected_objects)
previous_active = bpy.context.view_layer.objects.active
previous_mode = previous_active.mode if previous_active else "OBJECT"
try:
    if previous_mode != "OBJECT":
        bpy.ops.object.mode_set(mode="OBJECT")
    bpy.ops.object.select_all(action="DESELECT")
    for obj in model.all_objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = next(o for o in model.all_objects if o.type == "MESH")
    destination = root / "asset" / "meshes" / "blaster" / "training_blaster.gltf"
    destination.parent.mkdir(parents=True, exist_ok=True)
    export_model(
        destination,
        use_selection=True,
        use_active_scene=True,
        export_apply=True,
        export_animations=False,
        export_cameras=False,
        export_lights=False,
    )
    print(f"Exported {len(model.all_objects)} objects to {destination}")
    if "--" in sys.argv and "--preview" in sys.argv[sys.argv.index("--") + 1:]:
        preview = root / "artifacts" / "blaster.png"
        preview.parent.mkdir(exist_ok=True)
        previous_path = scene.render.filepath
        try:
            scene.render.filepath = str(preview)
            bpy.ops.render.render(write_still=True)
        finally:
            scene.render.filepath = previous_path
finally:
    bpy.ops.object.select_all(action="DESELECT")
    for obj in previous_selection:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = previous_active
    if previous_mode != "OBJECT":
        bpy.ops.object.mode_set(mode=previous_mode)
    bpy.context.window.scene = previous_scene
