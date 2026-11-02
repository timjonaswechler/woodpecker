"""Published programs have bounded retention; active consumers and foreign data survive."""
import json
from pathlib import Path
import tempfile
import unittest

from tests import _artifacts
from tests._product_cache import lease


def publish(root, generation, *, source_generation=None):
    (root / "src").mkdir(exist_ok=True)
    source = generation if source_generation is None else source_generation
    (root / "src/main.rs").write_text(f"fn main() {{ let _ = {source}; }}")
    binary = root / "program"
    binary.write_text(f"#!/bin/sh\n# generation {generation}\nexit 0\n")
    binary.chmod(0o700)
    log = root / "build.jsonl"
    log.write_text(json.dumps(dict(reason="compiler-artifact", executable=str(binary),
        target=dict(name="woodpecker", kind=["bin"]), profile=dict(test=False))) + "\n")
    return _artifacts.publish([("cli", log)], root / f"products-{generation}.json", root=root)


class ProductCacheTest(unittest.TestCase):
    def test_current_snapshot_retained_but_a_running_old_program_is_not_deleted(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = publish(root, 0)
            path = first["bin:woodpecker"]
            foreign = root / "target/test-products" / ("f" * 64)
            foreign.mkdir()
            (foreign / "keep.txt").write_text("not a framework product")
            with lease(first):
                for generation in range(1, 5):
                    latest = publish(root, generation)
                self.assertTrue(path.is_file(), "live consumer must pin its old executable")
                self.assertEqual(len(list((root / "target/test-products").glob("*/program"))), 2)
                self.assertEqual(_artifacts.load(root / "products-4.json", root=root), latest)
            publish(root, 5)
            self.assertFalse(path.exists())
            self.assertEqual(len(list((root / "target/test-products").glob("*/program"))), 1)
            self.assertEqual((foreign / "keep.txt").read_text(), "not a framework product")
            self.assertTrue((root / "products-0.json").is_file(), "historical manifest/evidence is retained")

    def test_relinking_unchanged_sources_does_not_accumulate_unlimited_versions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for generation in range(6):
                publish(root, generation, source_generation=0)
            self.assertEqual(len(list((root / "target/test-products").glob("*/program"))), 1)
            index = json.loads((root / "target/test-products/index.json").read_text())
            self.assertEqual(len(index["contexts"]), 1)
            self.assertEqual(len(index["contexts"][0]["roles"]["bin:woodpecker"]), 1)

    def test_changed_inputs_during_preparation_do_not_publish_a_manifest(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            publish(root, 0)
            before = _artifacts.inputs(root)
            (root / "src/main.rs").write_text("fn main() { panic!(); }")
            destination = root / "unsafe.json"
            with self.assertRaisesRegex(ValueError, "changed during preparation"):
                _artifacts.publish([], destination, root=root, expected_inputs=before)
            self.assertFalse(destination.exists())

    def test_normal_app_cannot_satisfy_an_automation_product_role(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "program"
            log = root / "build.jsonl"
            log.write_text(json.dumps(dict(reason="compiler-artifact", executable=str(binary),
                target=dict(name="test_app", kind=["bin"]), profile=dict(test=False))) + "\n")
            self.assertEqual(_artifacts.collect([("app-normal", log)]), {})
