"""Selective cleanup preserves the prepared compiler closure and unrelated files."""
import fcntl
import json
import os
from pathlib import Path
import tempfile
import unittest

from tests import _artifacts, _cargo_cache


class CargoCacheTest(unittest.TestCase):
    def fixture(self, root):
        (root / "Cargo.toml").write_text('[package]\nname = "package"\nversion = "0.1.0"\n')
        binary = root / "program"
        binary.write_text("#!/bin/sh\nexit 0\n")
        binary.chmod(0o700)
        product_log = root / "products.log"
        product_log.write_text(json.dumps(dict(reason="compiler-artifact", executable=str(binary),
            target=dict(name="woodpecker", kind=["bin"]), profile=dict(test=False))) + "\n")
        _artifacts.publish([("cli", product_log)], root / "products-0.json", root=root)
        builds = []
        for index, directory in enumerate((root / "target/debug", root / "target/panic-abort/debug")):
            deps = directory / "deps"
            deps.mkdir(parents=True)
            live = deps / "libalive-1111111111111111.rlib"
            for name in (live.name, "alive-1111111111111111.d", "alive-1111111111111111.a.rcgu.o",
                         "libdead-2222222222222222.rlib", "unrelated.txt", "private-3333333333333333"):
                (deps / name).write_text(name)
            incremental = directory / "incremental"
            for version in range(3):
                path = incremental / f"alive-abc{version}"
                path.mkdir(parents=True)
                (path / "cache.bin").write_text("cache")
                os.utime(path, (version + 1, version + 1))
            hashed = deps / "test_program-4444444444444444"
            hashed.write_text("current executable cache")
            hashed.chmod(0o700)
            fingerprint = directory / ".fingerprint/package-4444444444444444"
            fingerprint.mkdir(parents=True)
            (fingerprint / "bin-test-program.json").write_text("{}")
            public = directory / "test-program"
            public.write_bytes(hashed.read_bytes())
            log = root / f"build-{index}.log"
            log.write_text(json.dumps(dict(reason="compiler-artifact", filenames=[str(live)])) + "\n" +
                json.dumps(dict(reason="compiler-artifact", filenames=[str(public)], executable=str(public),
                                target=dict(name="test-program"), manifest_path=str(root / "Cargo.toml"))) + "\n")
            builds.append(dict(status="Bestanden", log=str(log)))
        summary = root / "summary.json"
        summary.write_text(json.dumps(dict(selection="full-local", errors=[], preflight=[], builds=builds,
                                          products_manifest=str(root / "products-0.json"))))
        return summary

    def test_dry_run_then_apply_only_removes_obsolete_rust_outputs_and_old_incremental_variants(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = self.fixture(root)
            planned = _cargo_cache.sweep(summary, root=root)
            self.assertEqual(planned["files"], 4)
            self.assertTrue(all(Path(name).exists() for name in planned["paths"]))
            applied = _cargo_cache.sweep(summary, root=root, apply=True)
            self.assertEqual(planned["paths"], applied["paths"])
            self.assertTrue(all(not Path(name).exists() for name in applied["paths"]))
            deps = root / "target/debug/deps"
            for name in ("libalive-1111111111111111.rlib", "alive-1111111111111111.d",
                         "alive-1111111111111111.a.rcgu.o", "test_program-4444444444444444",
                         "unrelated.txt", "private-3333333333333333"):
                self.assertTrue((deps / name).is_file(), name)
            self.assertTrue((root / "products-0.json").is_file())
            self.assertEqual(_cargo_cache.sweep(summary, root=root)["files"], 0)

    def test_incremental_caches_for_retired_crates_are_not_kept_forever(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = self.fixture(root)
            retired = []
            for output in (root / "target/debug", root / "target/panic-abort/debug"):
                path = output / "incremental/retired_graphical_game-abc0"
                path.mkdir()
                (path / "cache.bin").write_text("obsolete compiler output")
                retired.append(str(path.resolve()))
            planned = _cargo_cache.sweep(summary, root=root)
            self.assertTrue(set(retired) <= set(planned["paths"]))
            _cargo_cache.sweep(summary, root=root, apply=True)
            self.assertTrue(all(not Path(path).exists() for path in retired))
            self.assertTrue((root / "target/debug/incremental/alive-abc2/cache.bin").is_file())

    def test_active_cargo_lock_or_missing_compiler_output_prevents_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            summary = self.fixture(root)
            with (root / "target/debug/.cargo-lock").open("a") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX)
                with self.assertRaises(BlockingIOError):
                    _cargo_cache.sweep(summary, root=root, apply=True)
            (root / "target/debug/deps/libalive-1111111111111111.rlib").unlink()
            with self.assertRaisesRegex(ValueError, "no longer complete"):
                _cargo_cache.sweep(summary, root=root, apply=True)
            self.assertTrue((root / "target/debug/deps/libdead-2222222222222222.rlib").exists())
