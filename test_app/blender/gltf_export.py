"""Shared runtime-model export: external glTF, BIN buffers and PNG textures."""
import json
from pathlib import Path
from urllib.parse import unquote

import bpy


def export_model(filepath, **options):
    """Export without changing the saved Blender source; reject other formats."""
    path = Path(filepath)
    if path.suffix != ".gltf":
        raise ValueError("Runtime models must use a .gltf destination")
    path.parent.mkdir(parents=True, exist_ok=True)
    # Blender 5.2 has no explicit PNG enum. AUTO preserves PNG source textures;
    # check the emitted files so JPEG/WebP inputs cannot silently mix formats.
    options.update(filepath=str(path), export_format="GLTF_SEPARATE",
                   export_image_format="AUTO", export_image_add_webp=False)
    result = bpy.ops.export_scene.gltf(**options)
    if result != {"FINISHED"}:
        raise RuntimeError(f"glTF export failed: {path}")
    document = json.loads(path.read_text())
    for buffer in document.get("buffers", []):
        external = path.parent / unquote(buffer.get("uri", ""))
        if external.suffix != ".bin" or not external.is_file():
            raise RuntimeError(f"Export requires an external BIN buffer: {path}")
        if external.stat().st_size < buffer["byteLength"]:
            raise RuntimeError(f"Truncated BIN buffer: {external}")
    for image in document.get("images", []):
        external = path.parent / unquote(image.get("uri", ""))
        if external.suffix.lower() != ".png" or not external.is_file():
            raise RuntimeError(f"Convert source textures to PNG before exporting: {external}")
        if not external.read_bytes().startswith(b"\x89PNG\r\n\x1a\n"):
            raise RuntimeError(f"Not a PNG texture: {external}")
    return path
