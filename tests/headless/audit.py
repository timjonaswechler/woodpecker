#!/usr/bin/env python3
"""CPU-only verification of committed L1 images/source hashes; never starts an app."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = Path(__file__).with_name("evidence")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sources():
    paths = {ROOT / "Cargo.toml", ROOT / "Cargo.lock"}
    for directory in ("src", "crates", "test_app"):
        paths.update(path for path in (ROOT / directory).rglob("*")
                     if path.is_file() and (path.suffix == ".rs" or path.name in ("Cargo.toml", "Cargo.lock")))
    paths.update(path for path in (ROOT / "test_app/asset").rglob("*") if path.is_file())
    return {str(path.relative_to(ROOT)): sha(path) for path in sorted(paths)}


def main():
    from PIL import Image
    manifest = json.loads((EVIDENCE / "recorded.json").read_text())
    assert manifest["schema"] == 1
    assert manifest["runtime_sources"] == sources(), "Runtime sources/assets differ from the recorded proof"
    assert manifest["counts"] == {"headless": 13, "native_with_control": 99,
                                  "internal": 14, "app_unit": 21, "harness": 93, "offscreen": 8}
    for group in ("headless", "native"):
        assert all(row["status"] == "Bestanden" for row in manifest["cases"][group])
    assert len(manifest["cases"]["headless"]) == 13
    assert len(manifest["cases"]["native"]) == 99
    real = manifest["observations"]["real_clock"]
    assert real["paused_unchanged"] and real["after_counted_ticks"]["elapsed"] > real["before"]["elapsed"]
    native = manifest["native_qualification"]
    assert native["source_tree_equivalent_to"] == "8e7de92"
    assert set(native["later_runtime_delta"]) == {"test_app/src/diagnostics.rs", "test_app/src/diagnostics/clock.rs"}
    assert all(row["current_sha256"] == manifest["runtime_sources"][name]
               for name, row in native["later_runtime_delta"].items())
    assert native["additional_attempt"]["status"] == "Fehlgeschlagen"
    assert manifest["observations"]["native_windows"] == []
    assert manifest["observations"]["paused_sha256"] == manifest["images"]["a.png"]["sha256"]
    for name, record in manifest["images"].items():
        assert name in ("a.png", "map.png", "notes-before.png", "notes.png", "overlay.png")
        path = EVIDENCE / name
        assert sha(path) == record["sha256"], name
        with Image.open(path) as image:
            assert image.mode == "RGB" and image.size == (1440, 900), name
    box = tuple(manifest["observations"]["glyph_box"])
    with Image.open(EVIDENCE / "notes-before.png") as a, Image.open(EVIDENCE / "notes.png") as b:
        changed = sum(x != y for x, y in zip(a.crop(box).getdata(), b.crop(box).getdata()))
    assert changed == manifest["observations"]["glyph_changed_pixels"] and changed > 50
    with Image.open(EVIDENCE / "a.png") as a, Image.open(EVIDENCE / "map.png") as b:
        assert a.tobytes() != b.tobytes()
    print(json.dumps({"status": "verified recorded evidence", "new_gpu_execution": False,
                      "images": len(manifest["images"]), "runtime_files": len(manifest["runtime_sources"]),
                      "glyph_changed_pixels": changed}, indent=2))


if __name__ == "__main__":
    main()
