"""Screenshot tests and read-only snapshots of test-owned paths; no capture or ticks."""
import hashlib
import os


def snapshot_tree(root):
    """Record names, bytes and links without following links out of the given tree."""
    snapshot = {}
    for path in sorted(root.rglob("*")):
        relative = str(path.relative_to(root))
        if path.is_symlink():
            snapshot[relative] = {"symlink": os.readlink(path)}
        elif path.is_file():
            snapshot[relative] = {"sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        elif path.is_dir():
            snapshot[relative] = {"directory": True}
        else:
            raise AssertionError(f"Unexpected filesystem entry: {path}")
    return snapshot
