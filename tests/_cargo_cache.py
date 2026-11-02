"""Dry-run selective Cargo cache pruning from a successful full build preparation."""
import argparse
import contextlib
import fcntl
import json
from pathlib import Path
import re
import shutil
import tomllib

from tests import _artifacts
from tests._support import ROOT

# Only Rust-generated dependency outputs, never arbitrary files in target/.
ARTIFACT = re.compile(r"(.+-[0-9a-f]{16})(?:\.(?:rlib|rmeta|dylib|so|dll|a|d|exe)|\.[\w.-]+\.rcgu\.o)?$")


def allocated(path):
    if path.is_file():
        return path.stat().st_blocks * 512
    return sum(item.lstat().st_blocks * 512 for item in path.rglob("*") if item.is_file() and not item.is_symlink())


def sweep(preparation, *, apply=False, root=ROOT):
    root = Path(root).resolve()
    report = json.loads(Path(preparation).read_text())
    if (report.get("selection") != "full-local" or report.get("errors") or report.get("preflight")
            or not report.get("builds") or any(row["status"] != "Bestanden" for row in report["builds"])
            or any("log" not in row for row in report["builds"])):
        raise ValueError("Need a successful, complete full-local build preparation with compiler logs")
    manifest = Path(report["products_manifest"])
    _artifacts.load(manifest, root=root)
    directories = [root / "target/debug", root / "target/panic-abort/debug"]
    needed = {directory: set() for directory in directories}
    current_crates = {directory: set() for directory in directories}
    build_directories = {directory: set() for directory in directories}
    with contextlib.ExitStack() as locks:
        for directory in directories:
            if directory.is_symlink() or directory.parent.is_symlink():
                raise ValueError("Refusing a symlinked Cargo output directory")
            lock = locks.enter_context((directory / ".cargo-lock").open("a"))
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        for row in report["builds"]:
            for line in Path(row["log"]).read_text().splitlines():
                try:
                    item = json.loads(line)
                except ValueError:
                    continue
                if item.get("reason") == "build-script-executed":
                    output = Path(item["out_dir"]).resolve()
                    for directory in directories:
                        if output.parent.parent == directory / "build":
                            build_directories[directory].add(output.parent.name)
                if item.get("reason") != "compiler-artifact":
                    continue
                executable = Path(item["executable"]).resolve() if item.get("executable") else None
                for directory in directories:
                    if executable is not None and executable.parent == directory:
                        # Cargo JSON names the public binary, but its reusable cache copy
                        # lives in deps/<target>-<fingerprint>. Keep those counterparts too.
                        target = item["target"]["name"]
                        current_crates[directory].add(target.replace("-", "_"))
                        package = (tomllib.loads(Path(item["manifest_path"]).read_text())["package"]["name"]
                                   if "manifest_path" in item else None)
                        pattern = f"{package}-*/bin-{target}.json" if package else f"*/bin-{target}.json"
                        for fingerprint in (directory / ".fingerprint").glob(pattern):
                            suffix = fingerprint.parent.name.rsplit("-", 1)[-1]
                            if re.fullmatch(r"[0-9a-f]{16}", suffix):
                                needed[directory].add(f"{target.replace('-', '_')}-{suffix}")
                for name in item["filenames"]:
                    path = Path(name).resolve()
                    for directory in directories:
                        if path.parent.parent == directory / "build":
                            build_directories[directory].add(path.parent.name)
                            current_crates[directory].add(item.get("target", {}).get("name", "").replace("-", "_"))
                        if path.parent != directory / "deps":
                            continue
                        if not path.is_file():
                            raise ValueError(f"Preparation is no longer complete: {path}")
                        match = ARTIFACT.fullmatch(path.name)
                        if match:
                            prefix = match[1]
                            crate = item.get("target", {}).get("name", prefix.rsplit("-", 1)[0].removeprefix("lib"))
                            current_crates[directory].add(crate.replace("-", "_"))
                            needed[directory].update((prefix, prefix[3:] if prefix.startswith("lib") else "lib" + prefix))
        if any(not prefixes for prefixes in needed.values()):
            raise ValueError("Missing runtime/abort compiler dependency graph; refusing partial pruning")
        candidates = []
        for directory, prefixes in needed.items():
            builds = directory / "build"
            if builds.is_symlink():
                raise ValueError("Refusing symlinked Cargo build-script cache")
            if build_directories[directory] and builds.is_dir():
                candidates.extend(path for path in builds.iterdir() if path.is_dir() and not path.is_symlink()
                    and re.fullmatch(r".+-[0-9a-f]{16}", path.name) and path.name not in build_directories[directory])
            deps = directory / "deps"
            if deps.is_symlink():
                raise ValueError("Refusing symlinked Cargo dependencies")
            for path in deps.iterdir():
                match = ARTIFACT.fullmatch(path.name)
                if path.is_symlink() or not path.is_file() or not match or match[1] in prefixes:
                    continue
                if not path.suffix and not path.stat().st_mode & 0o111:
                    continue
                candidates.append(path)
            incremental = directory / "incremental"
            if incremental.is_symlink():
                raise ValueError("Refusing symlinked incremental cache")
            groups = {}
            for path in incremental.iterdir() if incremental.is_dir() else ():
                if path.is_dir() and not path.is_symlink() and re.fullmatch(r"[a-zA-Z0-9_]+-[a-z0-9]+", path.name):
                    groups.setdefault(path.name.rsplit("-", 1)[0], []).append(path)
            for crate, paths in groups.items():
                # Retired crates have no current outputs in this complete compiler graph.
                # Otherwise retain two configurations for subsequent source edits.
                retained = 2 if crate in current_crates[directory] else 0
                candidates.extend(sorted(paths, key=lambda path: path.stat().st_mtime, reverse=True)[retained:])
        result = dict(apply=apply, files=len(candidates), bytes=sum(allocated(path) for path in candidates),
                      paths=[str(path) for path in sorted(candidates)])
        if apply:
            for path in candidates:
                if path.is_dir():
                    shutil.rmtree(path)
                else:
                    path.unlink()
            _artifacts.load(manifest, root=root)
        return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--preparation", type=Path, required=True, help="full --build-only summary.json")
    parser.add_argument("--apply", action="store_true", help="delete only obsolete Rust outputs; default is dry-run")
    args = parser.parse_args()
    print(json.dumps(sweep(args.preparation, apply=args.apply), indent=2))


if __name__ == "__main__":
    main()
