"""Checkpoints preserve Cargo bytes, modes and nanosecond mtimes without touching evidence."""
import fcntl
import json
import os
from pathlib import Path
import tempfile
import unittest

from tests import _artifacts, _compiler_cache


def fixture(root):
    binary = root / "program"
    binary.write_text("#!/bin/sh\nexit 0\n")
    binary.chmod(0o700)
    log = root / "build.log"
    log.write_text(json.dumps(dict(reason="compiler-artifact", executable=str(binary),
        target=dict(name="woodpecker", kind=["bin"]), profile=dict(test=False))) + "\n")
    manifest = root / "products.json"
    _artifacts.publish([("cli", log)], manifest, root=root)
    return manifest


class CompilerCacheTest(unittest.TestCase):
    def test_roundtrip_preserves_code_freshness_and_prepared_programs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = fixture(root)
            source = root / "target/debug/deps/lib.rs.rlib"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"compiled code" * 10000)
            source.chmod(0o744)
            os.utime(source, ns=(1234567890123456789, 1234567890123456789))
            original = source.read_bytes(), source.stat().st_mtime_ns, source.stat().st_mode
            link = source.with_name("pointer.rlib")
            link.symlink_to(source.name)
            os.utime(link, ns=(original[1], original[1]), follow_symlinks=False)
            public = root / "target/debug/woodpecker"
            public.write_text("public CLI remains runnable")
            evidence = root / "target/test-runs/evidence.log"
            evidence.parent.mkdir(parents=True)
            evidence.write_text("retained original evidence")
            _compiler_cache.compact(root, manifest)
            self.assertFalse(source.exists())
            self.assertFalse(link.is_symlink())
            self.assertTrue(public.is_file())
            self.assertEqual(evidence.read_text(), "retained original evidence")
            self.assertLess((root / "target/compiler-cache.tar.gz").stat().st_size, len(original[0]))
            _artifacts.load(manifest, root=root)
            self.assertTrue(_compiler_cache.restore(root))
            self.assertEqual((source.read_bytes(), source.stat().st_mtime_ns, source.stat().st_mode), original)
            self.assertEqual((os.readlink(link), link.lstat().st_mtime_ns), (source.name, original[1]))
            self.assertFalse(_compiler_cache.restore(root))

    def test_corrupt_checkpoint_is_not_restored_or_discarded(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = fixture(root)
            source = root / "target/debug/deps/code.rlib"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"compiled code" * 100)
            _compiler_cache.compact(root, manifest)
            (root / "target/compiler-cache.tar.gz").write_bytes(b"corrupt")
            with self.assertRaisesRegex(ValueError, "changed compiler checkpoint"):
                _compiler_cache.restore(root)
            self.assertTrue((root / "target/compiler-cache.json").is_file())
            self.assertFalse(source.exists())

    def test_active_build_lock_prevents_compaction(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = fixture(root)
            path = root / "target/debug/.cargo-lock"
            path.parent.mkdir(parents=True)
            with path.open("a") as stream:
                fcntl.flock(stream, fcntl.LOCK_EX)
                with self.assertRaises(BlockingIOError):
                    _compiler_cache.compact(root, manifest)
            with self.assertRaises(ValueError):
                _compiler_cache.safe(root / "target", "debug/../../src/main.rs")

    def test_symlinked_lock_ancestor_does_not_touch_foreign_directory(self):
        with tempfile.TemporaryDirectory() as directory, tempfile.TemporaryDirectory() as outside:
            root = Path(directory)
            manifest = fixture(root)
            (root / "target/panic-abort").symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "symlinked compiler lock parent"):
                _compiler_cache.compact(root, manifest)
            self.assertEqual(list(Path(outside).iterdir()), [])
