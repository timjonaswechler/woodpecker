"""Content-checked executable manifests. Preparation writes; test execution only reads."""
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

from tests._support import ROOT

VERSION = 1


def digest(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def inputs(root=ROOT):
    """Include dirty compiled sources and runtime assets, not Git HEAD or mtimes alone."""
    root = Path(root).resolve()
    result = {}
    for directory, directories, files in os.walk(root):
        directories[:] = sorted(name for name in directories if name not in ("target", ".git", "__pycache__"))
        for name in sorted(files):
            path = Path(directory) / name
            relative = path.relative_to(root)
            if ((path.suffix == ".rs" and relative.parts[0] in ("src", "crates", "test_app", "test_zoo", "tests"))
                    or name in ("Cargo.toml", "Cargo.lock", "build.rs")
                    or str(relative) in ("tests/_builds.py", "tests/_artifacts.py", "tests/_audits.py")
                    or relative.parts[:2] == ("test_app", "asset") or relative.parts[0] == ".cargo"):
                result[str(relative)] = digest(path)
    configs = [Path.home() / ".cargo/config", Path.home() / ".cargo/config.toml"]
    configs += [directory / ".cargo" / name for directory in root.parents for name in ("config", "config.toml")]
    for path in configs:
        if path.is_file():
            result["config:" + str(path)] = digest(path)
    result["environment"] = {key: value for key, value in os.environ.items()
        if key.startswith("CARGO_PROFILE_") or key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS",
            "CARGO_BUILD_TARGET", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTUP_TOOLCHAIN")}
    result["platform"] = platform.machine() + ":" + platform.system()
    result["toolchain"] = {name: subprocess.run([os.environ.get(variable, name), "--version"],
        check=True, capture_output=True, text=True, timeout=15).stdout.strip()
        for name, variable in (("cargo", "CARGO"), ("rustc", "RUSTC"))}
    return result


def collect(logs):
    products = {}
    for name, path in logs:
        for line in Path(path).read_text().splitlines():
            try:
                item = json.loads(line)
            except ValueError:
                continue
            if item.get("reason") != "compiler-artifact" or not item.get("executable"):
                continue
            # Normal-app compilation is an audit only; automation overwrites its top-level output.
            if name == "app-normal":
                continue
            target = item["target"]
            role = ("test:" + target["name"] + ":" + ",".join(target["kind"]) if item["profile"]["test"]
                    else "bin:" + target["name"])
            if name == "app-abort" and target["name"] == "test_app" and not item["profile"]["test"]:
                role += ":abort"
            products[role] = Path(item["executable"]).resolve()
    return products


def publish(logs, destination, *, root=ROOT, feature_matrix=False, expected_inputs=None, roles=None):
    from tests import _product_cache
    with _product_cache.locked(root):
        products = _publish(logs, destination, root=root, feature_matrix=feature_matrix,
                            expected_inputs=expected_inputs, roles=roles)
        _product_cache.maintain(root, destination)
        return products


def _publish(logs, destination, *, root, feature_matrix, expected_inputs, roles):
    root = Path(root).resolve()
    prepared_inputs = inputs(root)
    if expected_inputs is not None and prepared_inputs != expected_inputs:
        raise ValueError("Build inputs changed during preparation; no manifest published")
    products = {}
    for role, source in collect(logs).items():
        if roles is not None and role not in roles:
            continue
        checksum = digest(source)
        directory = root / "target/test-products" / checksum
        if directory.is_symlink():
            raise ValueError(f"Refusing symlinked prepared product: {directory}")
        directory.mkdir(parents=True, exist_ok=True)
        prepared = directory / source.name
        if prepared.is_symlink():
            raise ValueError(f"Refusing symlinked prepared artifact: {prepared}")
        if not prepared.exists():
            temporary = prepared.with_name(prepared.name + ".preparing")
            if temporary.is_symlink():
                raise ValueError(f"Refusing symlinked staging file: {temporary}")
            shutil.copy2(source, temporary)
            temporary.replace(prepared)
        if digest(prepared) != checksum:
            raise ValueError(f"Prepared product changed: {prepared}")
        # macOS unpacked debuginfo otherwise points back into the disposable Cargo cache.
        with source.open("rb") as stream:
            magic = stream.read(4)
        if platform.system() == "Darwin" and magic in (b"\xcf\xfa\xed\xfe", b"\xfe\xed\xfa\xcf", b"\xca\xfe\xba\xbe"):
            symbols = prepared.with_name(prepared.name + ".dSYM")
            if not symbols.exists():
                with tempfile.TemporaryDirectory(prefix=".symbols-", dir=directory) as staging:
                    staged = Path(staging) / symbols.name
                    subprocess.run(["dsymutil", str(source), "-o", str(staged)], check=True,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=300)
                    staged.replace(symbols)
            if symbols.is_symlink() or not (symbols / "Contents/Resources/DWARF" / source.name).is_file():
                raise ValueError(f"Incomplete prepared debug symbols: {symbols}")
        products[role] = dict(path=str(prepared), sha256=checksum)
    if roles is not None and roles - products.keys():
        raise ValueError(f"Missing prepared product roles: {sorted(roles - products.keys())}")
    if not products:
        raise ValueError("No executable Cargo products found")
    if inputs(root) != prepared_inputs:
        raise ValueError("Build inputs changed during publication; no manifest published")
    data = dict(version=VERSION, root=str(root), inputs=prepared_inputs,
                feature_matrix=feature_matrix, products=products)
    temporary = Path(destination).with_suffix(".preparing")
    temporary.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
    temporary.replace(destination)
    return load(destination, root=root)


def load(path, *, root=ROOT):
    data = json.loads(Path(path).read_text())
    if data.get("version") != VERSION or data.get("root") != str(Path(root).resolve()):
        raise ValueError("Unsupported or foreign prepared artifact manifest")
    if data.get("inputs") != inputs(root):
        raise ValueError("Prepared artifact manifest is stale; prepare the build again")
    products = {}
    for role, product in data["products"].items():
        executable = Path(product["path"])
        if executable.is_symlink() or not executable.is_file() or digest(executable) != product["sha256"]:
            raise ValueError(f"Missing or changed prepared artifact: {role}")
        products[role] = executable
    return products


def required(rows):
    rows = list(rows)
    roles = set()
    if any(row["kind"] in ("python", "control") and row.get("module") != "tests.providers.github_live" for row in rows):
        roles.add("bin:woodpecker")
    if any(row["graphics"] and row.get("module") != "tests.reports.panic_abort" for row in rows):
        roles.add("bin:test_app")
    if any(row.get("module") == "tests.reports.panic_abort" for row in rows):
        roles.add("bin:woodpecker-abort-fixture")
    if any(row.get("module") == "tests.providers.github_live" for row in rows):
        roles.add("bin:github-live-driver")
    for row in rows:
        if row["kind"] == "internal":
            roles.add("test:woodpecker:lib")
        elif row["kind"] == "rust":
            roles.add("test:" + row["target"] + ":test")
    return roles
