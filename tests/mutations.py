"""Bounded step-7 campaign over four existing windowless contracts; not acceptance."""
import argparse
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import time

from tests._builds import commands, executables
from tests._support import ROOT, force_owned_tree, wait_process_gone


@dataclass(frozen=True)
class Candidate:
    name: str
    plan: str
    source: str
    old: str
    new: str
    test_source: str
    function: str
    assertion: str
    signature: tuple[str, ...]
    target: str = ""
    equivalence: str = ""


HANDLE = dict(source="src/handle.rs", old="EntityGeneration::from_bits(self.generation)",
              test_source="tests/rust/test_handle_encoding.rs", function="handle_encoding_contract",
              assertion="                assert_eq!(entity.generation().to_bits(), generation);",
              signature=("assertion `left == right` failed", "left: 0", "right: 2147483648"),
              target="handle_encoding")
CANDIDATES = (
    Candidate("handle-generation-bit", "4.69", new="EntityGeneration::from_bits(self.generation & 0x7fff_ffff)", **HANDLE),
    Candidate("native-keyboard-leak", "4.71", "src/session/input.rs",
              "        clear::<KeyboardInput>(world);", "        // Mutant: retain native typed keyboard messages.",
              "src/session/input/tests/test_native_input.rs",
              "native_input_cannot_enter_controlled_ticks_or_release_virtual_holds",
              "    empty!(", ("KeyboardInput",)),
    Candidate("text-current-focus", "4.72", "src/session/input/text.rs",
              "            if let Some(mut editable) = _world.get_mut::<EditableText>(entity) {",
              "            let entity = _world.resource::<InputFocus>().get().unwrap_or(entity);\n"
              "            if let Some(mut editable) = _world.get_mut::<EditableText>(entity) {",
              "src/session/input/text/tests/test_target_binding.rs",
              "accepted_text_stays_bound_across_focus_despawn_and_editability_changes",
              '        assert_eq!(\n            app.world()\n                .get::<EditableText>(b)\n'
              '                .unwrap()\n                .value()\n                .to_string(),\n'
              '            "beta:",\n            "{scenario}"\n        );',
              ("assertion `left == right` failed: focus", 'left: "beta:Grüße 🦉"', 'right: "beta:"')),
    Candidate("request-id-reuse", "4.77", "src/session/coordinator.rs",
              "                    let id = next;\n                    next += 1;\n                    Ok(id)",
              "                    let id = next;\n                    next += 0;\n                    Ok(id)",
              "src/session/coordinator/tests/test_request_id_exhaustion.rs",
              "exhausted_ids_never_admit_work_or_lose_pending_and_reserved_shutdown",
              "                assert!(matches!(\n                    session.send(warp::Stop {}),",
              ("assertion failed: matches!(session.send(warp::Stop {}), Err(Error::RequestIdExhausted))",)),
    Candidate("handle-or-zero", "4.69 control", new="EntityGeneration::from_bits(self.generation | 0)",
              equivalence="Handle.generation is u32; bitwise OR with inferred u32 zero preserves every bit "
                          "for all u32 values. The same EntityGeneration::from_bits receives the same value; "
                          "no effects, evaluation-order or overflow change.", **HANDLE),
)
# Workspace members must remain resolvable, but are not built. No app assets needed.
ARCHIVE_PATHS = ("Cargo.toml", "Cargo.lock", "src", "tests/rust", "tests/faults/internal_peer.py",
                 "tests/app.toml", "tests/settings.json", "test_app/Cargo.toml", "test_app/src",
                 "test_zoo/Cargo.toml", "test_zoo/src")


def save(path, data):
    path.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def file_state(path):
    if path.is_symlink():
        return {"symlink": os.readlink(path)}
    return {"sha256": digest(path), "size": path.stat().st_size} if path.exists() else {"missing": True}


def git(root, *args):
    return subprocess.run(["git", "-C", str(root), *args], check=True, capture_output=True, timeout=30).stdout


def tracked_state(root):
    return {name: file_state(root / name) for name in git(root, "ls-files", "-z").decode().split("\0") if name}


def source_state(source):
    return {str(p.relative_to(source)): file_state(p) for p in sorted(source.rglob("*"))
            if p.is_file() and p.relative_to(source).parts[0] != "target"}


def replace_once(text, old, new):
    if not old or text.count(old) != 1 or old == new:
        raise ValueError("Mutation/assertion site must match exactly once and change text")
    return text.replace(old, new, 1)


def assertion_line(source, candidate):
    text = (source / candidate.test_source).read_text()
    if text.count(candidate.assertion) != 1:
        raise ValueError(f"Ambiguous/missing intended assertion: {candidate.name}")
    return text[:text.index(candidate.assertion)].count("\n") + 1


def extract(archive, destination):
    """Accept only ordinary archive files/directories, never links or traversal."""
    with tarfile.open(archive) as bundle:
        members = bundle.getmembers()
        for member in members:
            path = Path(member.name)
            if path.is_absolute() or ".." in path.parts or not (member.isdir() or member.isfile()):
                raise ValueError(f"Unsafe archive entry: {member.name}")
        for member in members:
            path = destination / member.name
            if member.isdir():
                path.mkdir(parents=True, exist_ok=True)
            else:
                path.parent.mkdir(parents=True, exist_ok=True)
                with bundle.extractfile(member) as stream, path.open("xb") as output:
                    shutil.copyfileobj(stream, output)


def verify_private_cache(parent, private):
    """Check every copied inode, not just directory names or sample binaries."""
    if parent.resolve() == private.resolve() or private.is_relative_to(parent.resolve()):
        raise ValueError("Private target overlaps parent cache")
    parent_inodes = set()
    for path in [parent, *parent.rglob("*")]:
        info = path.lstat()
        if stat.S_ISLNK(info.st_mode):
            raise ValueError(f"Symlink in parent cache: {path}")
        if stat.S_ISREG(info.st_mode):
            parent_inodes.add((info.st_dev, info.st_ino))
    count = 0
    for path in [private, *private.rglob("*")]:
        info = path.lstat()
        if stat.S_ISLNK(info.st_mode):
            raise ValueError(f"Symlink in private cache: {path}")
        if stat.S_ISREG(info.st_mode):
            if (info.st_dev, info.st_ino) in parent_inodes:
                raise ValueError(f"Hardlink to parent cache: {path}")
            count += 1
    return {"files_checked": count, "no_parent_hardlinks_or_symlinks": True}


def evidence_destination(parent, destination):
    parent = parent.resolve()
    # Only this explicitly named ignored evidence subtree may receive parent writes.
    relative = destination.absolute().relative_to(parent)
    if (".." in relative.parts or len(relative.parts) != 5 or relative.parts[:2] != ("target", "test-runs")
            or relative.parts[3] != "mutation-evidence" or relative.parts[4] in (".", "..")):
        raise ValueError("Evidence must be PARENT/target/test-runs/CAMPAIGN/mutation-evidence/RUN")
    current = parent
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            raise ValueError(f"Symlink evidence path: {current}")
    if destination.exists():
        raise ValueError("Evidence destination already exists; never overwrite a previous run")
    return destination


def logged(command, cwd, path, env, timeout):
    begin = time.monotonic()
    row = dict(command=[str(arg) for arg in command], cwd=str(cwd), log=str(path),
               returncode=None, forced=False, timed_out=False, cleanup_errors=[])
    try:
        with path.open("w") as stream:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdout=stream, stderr=subprocess.STDOUT,
                                       start_new_session=True)
            row["pid"] = process.pid
            try:
                process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                row["timed_out"] = True
            finally:
                if process.poll() is None:
                    row["forced"] = True
                    row["forced_processes"] = force_owned_tree(process)
                row["returncode"] = process.returncode
                wait_process_gone({"pid": process.pid, "group": process.pid})
    except Exception as error:
        row["cleanup_errors"].append(f"{type(error).__name__}: {error}")
    finally:
        row["seconds"] = round(time.monotonic() - begin, 6)
        save(Path(str(path) + ".process.json"), row)
    return row


def clean(row):
    return not (row.get("forced") or row.get("timed_out") or row.get("cleanup_errors"))


def classify(build, baseline, run, output, candidate, name, line):
    """A semantic kill requires the one intended panic, not merely nonzero exit."""
    if not baseline:
        return "invalid"
    if build.get("timed_out") or (run and run.get("timed_out")):
        return "incomplete"
    if build.get("returncode") != 0 or not clean(build) or not run or not clean(run):
        return "invalid"
    summaries = re.findall(r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured;", output, re.M)
    if len(summaries) != 1 or output.count("running 1 test\n") != 1:
        return "invalid"
    status, passed, failed, ignored, measured = summaries[0]
    if ignored != "0" or measured != "0" or not re.search(rf"^test {re.escape(name)} \.\.\.", output, re.M):
        return "invalid"
    if (status, passed, failed, run["returncode"]) == ("ok", "1", "0", 0) and "panicked at" not in output:
        return "known-equivalent" if candidate.equivalence else "survived"
    if candidate.equivalence or (status, passed, failed, run["returncode"]) != ("FAILED", "0", "1", 101):
        return "invalid"
    panics = re.findall(r"thread '([^']+)'(?: \(\d+\))? panicked at ([^\n]+?):(\d+):\d+:\n", output)
    if len(panics) != 1 or output.count("panicked at") != 1:
        return "invalid"
    function, location, actual_line = panics[0]
    if function != name or not (location == candidate.test_source or location.endswith("/" + candidate.test_source)) or int(actual_line) != line:
        return "invalid"
    normalized = " ".join(output.split())
    if not all(" ".join(part.split()) in normalized for part in candidate.signature):
        return "invalid"
    return "killed"


def build(source, cache, evidence, label, rows, env):
    [(unused, command)] = commands([dict(row, graphics=False) for row in rows])
    result = logged(command, source, evidence / f"{label}-build.log", env, 900)
    return result, executables(result["log"])


def run_test(source, cache, evidence, label, artifacts, candidate, env):
    key = (candidate.target, ("test",)) if candidate.target else ("woodpecker", ("lib",))
    executable = Path(artifacts[key])
    if executable.is_symlink() or not executable.resolve().is_relative_to(cache.resolve()):
        raise ValueError(f"Test binary not private: {executable}")
    listing = logged([executable, "--list"], source, evidence / f"{label}-list.log", env, 30)
    names = [line[:-6] for line in Path(listing["log"]).read_text().splitlines() if line.endswith(": test")]
    matches = [name for name in names if name.split("::")[-1] == candidate.function]
    if listing["returncode"] != 0 or not clean(listing) or len(matches) != 1:
        raise ValueError(f"Compiled test binding failed: {listing['log']}")
    name = matches[0]
    before = set((source / "target/test-runs").glob("**/peer.json"))
    result = logged([executable, "--exact", name, "--nocapture", "--test-threads=1"],
                    source, evidence / f"{label}-test.log", env, 60)
    result.update(function=name, executable=str(executable), executable_sha256=digest(executable),
                  listing=listing, peers=[])
    for path in sorted(set((source / "target/test-runs").glob("**/peer.json")) - before):
        peer = json.loads(path.read_text())
        try:
            wait_process_gone(peer)
            result["peers"].append(dict(path=str(path), **peer, absent=True))
        except Exception as error:
            result["cleanup_errors"].append(str(error))
    save(evidence / f"{label}-runtime.json", result)
    return result, Path(result["log"]).read_text()


def campaign(args):
    parent = Path(args.parent_root).resolve()
    run_root = ROOT / "target/test-runs"
    run_root.mkdir(parents=True, exist_ok=True)
    owned = Path(tempfile.mkdtemp(prefix="mutations-", dir=run_root)).resolve()
    evidence = owned / "evidence"
    evidence.mkdir()
    source, cache = owned / "source", owned / "cargo-target"
    destination = evidence_destination(parent, Path(args.evidence_dir) / owned.name)
    report = dict(scope="Bounded windowless sample; no universal sensitivity or historical score claim",
                  started_at=datetime.now(timezone.utc).isoformat(), authoring_worktree=str(ROOT),
                  owned=str(owned), source_copy=str(source), private_target=str(cache), evidence=str(destination),
                  requested_ref=args.source_ref, candidates=[dict(**asdict(c), outcome="incomplete") for c in CANDIDATES],
                  errors=[], baselines=[], builds=[])
    original = None
    parent_before = None
    products = None
    try:
        if ROOT.resolve() == parent or owned.is_relative_to(parent):
            raise ValueError("Authoring/cache must be outside parent checkout")
        report["tooling_sha256"] = {}
        for name in ("tests/mutations.py", "tests/runner_checks/mutations.py", "tests/mutations.md"):
            report["tooling_sha256"][name] = digest(ROOT / name)
            snapshot = evidence / "tooling" / name
            snapshot.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, snapshot)
        report["source_commit"] = git(ROOT, "rev-parse", "--verify", args.source_ref + "^{commit}").decode().strip()
        report["authoring_head"] = git(ROOT, "rev-parse", "HEAD").decode().strip()
        report["authoring_status_before"] = git(ROOT, "status", "--porcelain=v1").decode()
        report["authoring_sites_before"] = {c.source: file_state(ROOT / c.source) for c in CANDIDATES}
        report["parent_root"] = str(parent)
        parent_before = tracked_state(parent)
        save(evidence / "parent-tracked-before.json", parent_before)
        products = json.loads(Path(args.parent_products).read_text())
        if set(products) != {"target/debug/woodpecker", "target/debug/test_app"}:
            raise ValueError("Expected parent woodpecker and test_app hash manifest")
        actual = {p: file_state(parent / p) for p in products}
        save(evidence / "parent-products-before.json", actual)
        if actual != products:
            raise ValueError("Parent binaries differ from supplied pre-mutation hashes")
        archive = owned / "source.tar"
        git(ROOT, "archive", "--format=tar", "-o", str(archive), report["source_commit"], *ARCHIVE_PATHS)
        report["archive_sha256"] = digest(archive)
        extract(archive, source)
        original = source_state(source)
        save(evidence / "source-before.json", original)
        report["source_manifest_sha256"] = digest(evidence / "source-before.json")
        # Validate all declared sites/oracles before any copy is changed or built.
        for candidate in CANDIDATES:
            replace_once((source / candidate.source).read_text(), candidate.old, candidate.new)
            assertion_line(source, candidate)
        cache.mkdir()
        if args.clone_parent_cache:
            row = logged(["cp", "-cR", parent / "target/debug", cache / "debug"], ROOT,
                         evidence / "cache-clone.log", os.environ.copy(), 180)
            report["cache_clone"] = row
            if row["returncode"] != 0 or not clean(row):
                raise RuntimeError("Safe COW cache preparation unavailable; no fallback")
        report["cache_isolation"] = verify_private_cache(parent / "target/debug", cache)
        env = {**os.environ, "CARGO_NET_OFFLINE": "true", "CARGO_TARGET_DIR": str(cache), "RUST_BACKTRACE": "0"}
        env.pop("WOODPECKER_TEST_RENDERED", None)
        report["environment"] = {key: env.get(key) for key in
                                 ("CARGO_NET_OFFLINE", "CARGO_TARGET_DIR", "WOODPECKER_TEST_RENDERED", "RUST_BACKTRACE", "RUSTFLAGS")}
        save(evidence / "summary.json", report)  # Paths/hashes durable before any mutation.
        baseline_build, artifacts = build(source, cache, evidence, "baseline",
            [dict(kind="internal"), dict(kind="rust", target="handle_encoding")], env)
        report["builds"].append(baseline_build)
        if not clean(baseline_build) or baseline_build["returncode"] != 0:
            raise RuntimeError(f"Baseline build infrastructure failure: {baseline_build['log']}")
        baselines = {}
        for candidate in CANDIDATES:
            if candidate.function in baselines:
                continue
            result, output = run_test(source, cache, evidence, "baseline-" + candidate.name, artifacts, candidate, env)
            good = classify(baseline_build, True, result, output, candidate, result["function"], assertion_line(source, candidate)) == "survived"
            report["baselines"].append(dict(owner=candidate.function, passed=good, runtime=result))
            if not good:
                raise RuntimeError(f"Real baseline failed: {result['log']}")
            baselines[candidate.function] = True
        for candidate, row in zip(CANDIDATES, report["candidates"]):
            path = source / candidate.source
            pristine = path.read_bytes()
            changed = replace_once(pristine.decode(), candidate.old, candidate.new)
            line = assertion_line(source, candidate)
            row.update(expected_assertion_line=line, baseline_owner=candidate.function,
                       original_sha256=digest(path), patch=str(evidence / (candidate.name + ".patch")))
            Path(row["patch"]).write_text("".join(difflib.unified_diff(pristine.decode().splitlines(True),
                changed.splitlines(True), fromfile="a/" + candidate.source, tofile="b/" + candidate.source)))
            row["patch_sha256"] = digest(Path(row["patch"]))
            try:
                if source_state(source) != original:
                    raise ValueError("Source/oracle changed before mutation")
                path.write_text(changed)
                row["mutant_sha256"] = digest(path)
                row["outcome"] = "invalid"  # A build/setup error cannot be a kill.
                save(evidence / "summary.json", report)
                built, artifacts = build(source, cache, evidence, candidate.name,
                    [dict(kind="rust", target=candidate.target)] if candidate.target else [dict(kind="internal")], env)
                row["build"] = built
                if not clean(built) or built["returncode"] != 0:
                    row["outcome"] = "incomplete" if built["timed_out"] else "invalid"
                    raise RuntimeError(f"Build infrastructure failure: {built['log']}")
                result, output = run_test(source, cache, evidence, candidate.name, artifacts, candidate, env)
                row["runtime"] = result
                row["outcome"] = classify(built, baselines[candidate.function], result, output, candidate, result["function"], line)
                if row["outcome"] in ("invalid", "incomplete"):
                    raise RuntimeError(f"Not semantic detection; stop: {candidate.name}: {row['outcome']}")
            except KeyboardInterrupt:
                row["outcome"] = "incomplete"
                raise
            finally:
                path.write_bytes(pristine)
                row["restored_sha256"] = digest(path)
                row["source_and_oracles_restored"] = source_state(source) == original
                save(evidence / "summary.json", report)
                if not row["source_and_oracles_restored"]:
                    raise RuntimeError("Source/oracle restoration failed; stop")
    except (Exception, KeyboardInterrupt) as error:
        report["errors"].append(f"{type(error).__name__}: {error}")
    finally:
        if "authoring_sites_before" in report:
            report["authoring_sites_after"] = {c.source: file_state(ROOT / c.source) for c in CANDIDATES}
            report["authoring_sites_unchanged"] = report["authoring_sites_before"] == report["authoring_sites_after"]
        if original is not None:
            report["final_source_restored"] = source_state(source) == original
            save(evidence / "source-after.json", source_state(source))
        if parent_before is not None:
            after = tracked_state(parent)
            save(evidence / "parent-tracked-after.json", after)
            report["parent_tracked_unchanged"] = after == parent_before
        if products is not None:
            after = {p: file_state(parent / p) for p in products}
            save(evidence / "parent-products-after.json", after)
            report["parent_products_unchanged"] = after == products
        runtime_artifacts = source / "target/test-runs"
        if runtime_artifacts.exists():
            shutil.copytree(runtime_artifacts, evidence / "runtime-artifacts")
        report["counts"] = {key: sum(row["outcome"] == key for row in report["candidates"])
                            for key in ("killed", "survived", "known-equivalent", "invalid", "incomplete")}
        report["complete"] = not report["errors"] and all(report.get(key) for key in
            ("final_source_restored", "authoring_sites_unchanged", "parent_tracked_unchanged", "parent_products_unchanged"))
        save(evidence / "summary.json", report)
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(evidence, destination)
        print(json.dumps(dict(report=str(destination / "summary.json"), counts=report["counts"],
                              complete=report["complete"], errors=report["errors"]), indent=2))
    return 2 if not report["complete"] else 1 if report["counts"]["survived"] else 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--list", action="store_true", help="catalog only; no processes, builds or writes")
    parser.add_argument("--source-ref", default="HEAD", help="explicit source revision (default: HEAD, resolved once)")
    parser.add_argument("--parent-root", help="read-only original checkout to protect")
    parser.add_argument("--parent-products", help="pre-mutation JSON hashes of the two original binaries")
    parser.add_argument("--evidence-dir", help="exclusive PARENT/target/test-runs/CAMPAIGN/mutation-evidence directory")
    parser.add_argument("--clone-parent-cache", action="store_true", help="macOS/APFS cp -cR only; no fallback")
    args = parser.parse_args(argv)
    if args.list:
        print(json.dumps([asdict(c) for c in CANDIDATES], indent=2, ensure_ascii=False))
        return 0
    if not all((args.parent_root, args.parent_products, args.evidence_dir)):
        parser.error("run requires --parent-root, --parent-products and --evidence-dir")
    return campaign(args)


if __name__ == "__main__":
    sys.exit(main())
