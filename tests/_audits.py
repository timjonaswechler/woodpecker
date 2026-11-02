"""Feature audits retain content-checked evidence, not another renderer compilation tree."""
import contextlib
import hashlib
import json
from pathlib import Path
import shutil
import uuid


@contextlib.contextmanager
def workspace(root):
    directory = Path(root) / "target" / ("feature-audit-" + uuid.uuid4().hex)
    directory.mkdir(parents=True, exist_ok=False)
    try:
        yield str(directory)
    finally:
        shutil.rmtree(directory)


def path(root, inputs):
    key = hashlib.sha256(json.dumps(inputs, sort_keys=True).encode()).hexdigest()
    return Path(root) / "target/test-audits" / (key + ".json")


def load(root, inputs):
    record = path(root, inputs)
    if not record.is_file():
        return {}
    data = json.loads(record.read_text())
    if data.get("inputs") != inputs:
        return {}
    result = {}
    for name, row in data["builds"].items():
        log = Path(row["log"])
        if log.is_file() and hashlib.sha256(log.read_bytes()).hexdigest() == row["sha256"]:
            result[name] = row
    return result


def publish(root, inputs, builds):
    """Only successful complete audit logs become reusable proof; no compile outputs survive."""
    if not builds or any(not Path(row["log"]).is_file() for row in builds.values()):
        return
    data = dict(inputs=inputs, builds={name: dict(row,
        sha256=hashlib.sha256(Path(row["log"]).read_bytes()).hexdigest()) for name, row in builds.items()})
    destination = path(root, inputs)
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_suffix(".preparing")
    temporary.write_text(json.dumps(data, indent=2) + "\n")
    temporary.replace(destination)
