"""Prepared products are reused through their manifest, never rebuilt by a test."""
import json
from pathlib import Path
import tempfile
import unittest

from tests import _artifacts


class ArtifactsTest(unittest.TestCase):
    def test_prepared_manifest_reuses_the_same_file_and_rejects_changed_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "src").mkdir()
            source = root / "src/main.rs"
            source.write_text("fn main() {}")
            binary = root / "woodpecker"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(0o700)
            log = root / "build.jsonl"
            log.write_text(json.dumps(dict(reason="compiler-artifact", executable=str(binary),
                target=dict(name="woodpecker", kind=["bin"]), profile=dict(test=False))) + "\n")
            manifest = root / "products.json"
            _artifacts.publish([("cli", log)], manifest, root=root)
            first = _artifacts.load(manifest, root=root)
            second = _artifacts.load(manifest, root=root)
            self.assertEqual(first["bin:woodpecker"], second["bin:woodpecker"])
            self.assertEqual(first["bin:woodpecker"].read_bytes(), binary.read_bytes())
            self.assertNotEqual(first["bin:woodpecker"], binary)
            prepared = first["bin:woodpecker"]
            prepared.write_text("changed product")
            with self.assertRaisesRegex(ValueError, "Missing or changed"):
                _artifacts.load(manifest, root=root)
            prepared.write_bytes(binary.read_bytes())
            source.write_text("fn main() { panic!(); }")
            with self.assertRaisesRegex(ValueError, "stale"):
                _artifacts.load(manifest, root=root)
