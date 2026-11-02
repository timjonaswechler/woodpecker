"""Lossless idle checkpoints for Cargo outputs; prepared executables remain directly runnable.

This changes storage, not compilation settings. A build needs its ordinary uncompressed
working set. All mtimes are restored exactly so unchanged dependencies stay fresh.
"""
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import stat
import tarfile
import uuid


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def sync_directory(path):
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def paths(root):
    target = Path(root).resolve() / "target"
    if target.is_symlink():
        raise ValueError("Refusing symlinked compiler target")
    target.mkdir(exist_ok=True)
    archive, state = target / "compiler-cache.tar.gz", target / "compiler-cache.json"
    if archive.is_symlink() or state.is_symlink():
        raise ValueError("Refusing symlinked compiler checkpoint")
    return target, archive, state


def packed(root):
    return paths(root)[2].is_file()


@contextlib.contextmanager
def locked(root):
    target, _, _ = paths(root)
    with contextlib.ExitStack() as scope:
        for path in (target / ".compiler-cache.lock", target / "debug/.cargo-lock",
                     target / "panic-abort/debug/.cargo-lock"):
            for parent in path.parents:
                if parent == target:
                    break
                if parent.is_symlink():
                    raise ValueError("Refusing symlinked compiler lock parent")
            if path.is_symlink():
                raise ValueError("Refusing symlinked compiler lock")
            path.parent.mkdir(parents=True, exist_ok=True)
            stream = scope.enter_context(path.open("a"))
            fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


def safe(target, name):
    relative = Path(name)
    if relative.is_absolute() or ".." in relative.parts or not relative.parts:
        raise ValueError("Unsafe compiler checkpoint path")
    if relative.parts[0] not in ("debug", "panic-abort"):
        raise ValueError("Compiler checkpoint escapes Cargo outputs")
    result = target / relative
    for parent in result.parents:
        if parent == target:
            break
        if parent.is_symlink():
            raise ValueError("Compiler checkpoint parent is a symlink")
    return result


def verify(archive, entries):
    seen = set()
    with tarfile.open(archive, "r:gz") as stream:
        for member in stream:
            if member.name not in entries or member.name in seen:
                raise ValueError("Unexpected compiler checkpoint member")
            seen.add(member.name)
            expected = entries[member.name]
            if expected["kind"] == "file":
                if not member.isfile() or member.size != expected["size"]:
                    raise ValueError("Compiler checkpoint size/type mismatch")
                content = stream.extractfile(member)
                if hashlib.file_digest(content, "sha256").hexdigest() != expected["sha256"]:
                    raise ValueError("Compiler checkpoint contents changed")
            elif not member.issym() or member.linkname != expected["link"]:
                raise ValueError("Compiler checkpoint link mismatch")
    if seen != entries.keys():
        raise ValueError("Incomplete compiler checkpoint")


def compact(root, manifest):
    from tests import _artifacts
    _artifacts.load(manifest, root=root)
    target, archive, state = paths(root)
    with locked(root):
        if state.exists():
            data = json.loads(state.read_text())
            if data["root"] != str(Path(root).resolve()) or digest(archive) != data["archive_sha256"]:
                raise ValueError("Foreign or changed compiler checkpoint")
            return data
        if archive.exists():
            raise ValueError("Orphan compiler archive requires explicit inspection")
        entries = {}
        temporary = archive.with_name(archive.name + ".preparing-" + uuid.uuid4().hex)
        pending = state.with_name(state.name + ".preparing-" + uuid.uuid4().hex)
        try:
            with tarfile.open(temporary, "x:gz", compresslevel=6, format=tarfile.PAX_FORMAT) as output:
                for base in (target / "debug", target / "panic-abort/debug"):
                    for path in sorted(base.rglob("*")):
                        inner = path.relative_to(base)
                        if (inner.parts[0] not in ("deps", ".fingerprint", "build", "incremental", "examples")
                                and not inner.parts[0].endswith(".dSYM")):
                            continue
                        relative = path.relative_to(target).as_posix()
                        if path.name == ".cargo-lock" or path.parent == base:
                            continue  # Keep public CLI/app paths and the Cargo lock in place.
                        if path.is_dir() and not path.is_symlink():
                            continue
                        safe(target, relative)
                        info = path.lstat()
                        common = dict(mode=stat.S_IMODE(info.st_mode), mtime_ns=info.st_mtime_ns,
                                      size=info.st_size, inode=info.st_ino)
                        member = tarfile.TarInfo(relative)
                        member.mode = common["mode"]
                        if path.is_symlink():
                            link = os.readlink(path)
                            if not (path.parent / link).resolve().is_relative_to(target):
                                raise ValueError("External link in compiler outputs")
                            member.type = tarfile.SYMTYPE
                            member.linkname = link
                            entries[relative] = dict(common, kind="link", link=link)
                            output.addfile(member)
                        elif path.is_file():
                            member.size = info.st_size
                            checksum = digest(path)
                            entries[relative] = dict(common, kind="file", sha256=checksum)
                            with path.open("rb") as content:
                                output.addfile(member, content)
                        else:
                            raise ValueError("Special file in compiler outputs")
            with temporary.open("rb") as stream:
                os.fsync(stream.fileno())
            verify(temporary, entries)
            for name, entry in entries.items():
                info = safe(target, name).lstat()
                if (info.st_ino, info.st_size, info.st_mtime_ns) != (entry["inode"], entry["size"], entry["mtime_ns"]):
                    raise ValueError("Compiler output changed while checkpointing")
            temporary.replace(archive)
            data = dict(root=str(Path(root).resolve()), archive_sha256=digest(archive), entries=entries,
                        manifest=str(Path(manifest).resolve()))
            with pending.open("x") as stream:
                stream.write(json.dumps(data) + "\n")
                stream.flush()
                os.fsync(stream.fileno())
            pending.replace(state)
            sync_directory(target)  # Recovery is durable before removing any uncompressed file.
            for name in entries:
                safe(target, name).unlink()
            return data
        finally:
            temporary.unlink(missing_ok=True)
            pending.unlink(missing_ok=True)


def restore(root):
    target, archive, state = paths(root)
    with locked(root):
        if not state.exists():
            return False
        data = json.loads(state.read_text())
        if data["root"] != str(Path(root).resolve()) or digest(archive) != data["archive_sha256"]:
            raise ValueError("Foreign or changed compiler checkpoint")
        verify(archive, data["entries"])
        with tarfile.open(archive, "r:gz") as stream:
            for member in stream:
                path = safe(target, member.name)
                path.parent.mkdir(parents=True, exist_ok=True)
                entry = data["entries"][member.name]
                temporary = path.with_name(path.name + ".restoring-" + uuid.uuid4().hex)
                try:
                    if entry["kind"] == "link":
                        if not (path.parent / entry["link"]).resolve().is_relative_to(target):
                            raise ValueError("External checkpoint link")
                        temporary.symlink_to(entry["link"])
                        os.utime(temporary, ns=(entry["mtime_ns"], entry["mtime_ns"]), follow_symlinks=False)
                    else:
                        with stream.extractfile(member) as source, temporary.open("xb") as output:
                            while chunk := source.read(1024 * 1024):
                                output.write(chunk)
                            output.flush()
                            os.fchmod(output.fileno(), entry["mode"] & 0o777)
                            os.utime(output.fileno(), ns=(entry["mtime_ns"], entry["mtime_ns"]))
                            os.fsync(output.fileno())
                    temporary.replace(path)
                finally:
                    temporary.unlink(missing_ok=True)
        for parent in {safe(target, name).parent for name in data["entries"]}:
            sync_directory(parent)
        state.unlink()
        archive.unlink()
        sync_directory(target)
        return True


if __name__ == "__main__":
    import argparse
    from tests._support import ROOT
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--restore", action="store_true", required=True)
    parser.parse_args()
    print("Compiler cache restored" if restore(ROOT) else "Compiler cache is already unpacked")
