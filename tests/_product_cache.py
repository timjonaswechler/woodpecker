"""Keep one current prepared product per role; running consumers hold read leases."""
import contextlib
import fcntl
import hashlib
import json
from pathlib import Path
import shutil


def store(root):
    directory = Path(root).resolve() / "target/test-products"
    if directory.is_symlink() or directory.parent.is_symlink():
        raise ValueError("Refusing maintenance of a symlinked product cache")
    directory.mkdir(parents=True, exist_ok=True)
    return directory


@contextlib.contextmanager
def locked(root, *, shared=False):
    directory = store(root)
    with (directory / ".lock").open("a") as stream:
        fcntl.flock(stream, fcntl.LOCK_SH if shared else fcntl.LOCK_EX)
        yield directory


@contextlib.contextmanager
def lease(products):
    # Locks are on the programs, not PID files: exits release them even after crashes.
    groups = {}
    for value in products.values():
        path = Path(value)
        if path.parent.parent.name == "test-products" and path.parent.parent.parent.name == "target":
            groups.setdefault(path.parent.parent.parent.parent, set()).add(path)
    with contextlib.ExitStack() as consumers:
        for root, paths in sorted(groups.items()):
            with locked(root, shared=True):
                for path in sorted(paths):
                    if path.parent.is_symlink():
                        raise ValueError("Refusing symlinked prepared product")
                    stream = consumers.enter_context(path.open("rb"))
                    fcntl.flock(stream, fcntl.LOCK_SH)
        yield


def _hashes(data, root, directory):
    if data.get("version") != 1 or data.get("root") != str(Path(root).resolve()):
        return set()
    result = set()
    for product in data.get("products", {}).values():
        path = Path(product["path"])
        if (path.parent.parent == directory and path.parent.name == product["sha256"]
                and len(path.parent.name) == 64 and all(c in "0123456789abcdef" for c in path.parent.name)):
            result.add(path.parent.name)
    return result


def maintain(root, manifest):
    """Caller holds the exclusive store lock. Preserve logs/manifests and foreign entries."""
    directory = store(root)
    index_path = directory / "index.json"
    index = json.loads(index_path.read_text()) if index_path.exists() else {}
    if index.get("version") != 2:
        index = dict(version=2, contexts=[], known=index.get("known", []))
        # Adopt only manifests produced by this checkout, never arbitrary hash directories.
        for path in sorted((Path(root) / "target/test-runs").glob("acceptance-*/products.json"),
                           key=lambda path: path.stat().st_mtime):
            try:
                _include(index, json.loads(path.read_text()), root, directory)
            except (ValueError, KeyError, TypeError):
                continue
    _include(index, json.loads(Path(manifest).read_text()), root, directory)
    index["contexts"] = index["contexts"][-1:]
    retained = {checksum for entry in index["contexts"] for checksum in entry["hashes"]}
    removed = []
    for checksum in sorted(set(index["known"]) - retained):
        candidate = directory / checksum
        if not candidate.exists():
            removed.append(checksum)
            continue
        if candidate.is_symlink() or not candidate.is_dir():
            continue
        files = [path for path in candidate.iterdir() if path.is_file() and not path.is_symlink()]
        if not files or any(path.is_symlink() or (path.is_dir() and path.name not in
                {program.name + ".dSYM" for program in files}) for path in candidate.iterdir()):
            continue
        try:
            with contextlib.ExitStack() as handles:
                for path in files:
                    stream = handles.enter_context(path.open("rb"))
                    fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    if hashlib.file_digest(stream, "sha256").hexdigest() != checksum:
                        raise ValueError("Unrecognized product cache contents")
                shutil.rmtree(candidate)
                removed.append(checksum)
        except (BlockingIOError, ValueError):
            continue
    index["known"] = sorted(set(index["known"]) - set(removed))
    temporary = index_path.with_suffix(".preparing")
    temporary.write_text(json.dumps(index, indent=2) + "\n")
    temporary.replace(index_path)
    return removed


def _include(index, data, root, directory):
    hashes = _hashes(data, root, directory)
    if not hashes:
        return
    key = hashlib.sha256(json.dumps(data["inputs"], sort_keys=True).encode()).hexdigest()
    previous = next((entry for entry in index["contexts"] if entry["key"] == key), None)
    roles = {} if previous is None else previous["roles"]
    if previous:
        index["contexts"].remove(previous)
    for role, product in data["products"].items():
        checksum = product["sha256"]
        if checksum in hashes:
            versions = [value for value in roles.get(role, []) if value != checksum]
            # Mach-O link timestamps can change bytes without changing source inputs.
            roles[role] = [*versions, checksum][-1:]
    retained = {value for versions in roles.values() for value in versions}
    index["contexts"].append(dict(key=key, roles=roles, hashes=sorted(retained)))
    index["known"] = sorted(set(index["known"]) | hashes)
