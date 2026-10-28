"""Run the existing acceptance cases with Cargo/unittest; subsets never certify the whole suite."""
import argparse
import contextlib
from datetime import datetime, timezone
import importlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
import tomllib
import unittest

from tests._support import ROOT, force_owned_tree
from tests._builds import commands, executables, rust_source_tests

PILOT_MODULES = (
    "tests.session.server_lifecycle", "tests.session.session_lifecycle",
    "tests.session.session_isolation", "tests.session.unknown_session",
    "tests.session.invalid_session_id", "tests.session.invalid_launch_config",
    "tests.session.launch_failure", "tests.inspect.resource_value", "tests.startup.manual_startup",
    "tests.tick.exact_ticks", "tests.startup.engine_startup", "tests.keyboard.movement",
    "tests.screenshot.capture", "tests.replay.recording_roundtrip",
    "tests.reports.tracing_report", "tests.reports.panic_exit",
)
# Safety classification only, never scenario definitions. Unknown cases default to graphical.
NON_GRAPHICAL = frozenset({
    "tests.session.server_lifecycle", "tests.session.unknown_session",
    "tests.session.invalid_session_id", "tests.session.invalid_launch_config",
    "tests.session.launch_failure", "tests.startup.invalid_config", "tests.session.local_access",
    "tests.session.protocol_faults", "tests.replay.blocked", "tests.cli.config_errors",
    "tests.cli.command_errors", "tests.cli.script_unknown", "tests.reports.invalid_marker",
    "tests.reports.before_ready", "tests.shutdown.starting",
})
EXTERNAL = "tests.providers.github_live"
CONTROL = "control::3.18"


def verdict(statuses, *, missing=False):
    statuses = list(statuses)
    if "Fehlgeschlagen" in statuses:
        return "Fehlgeschlagen"
    if missing or not statuses or any(status != "Bestanden" for status in statuses):
        return "Unvollständig"
    return "Bestanden"


def cases_in(suite):
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            yield from cases_in(item)
        else:
            yield item


def inventory():
    loader = unittest.TestLoader()
    cases = [case for case in cases_in(loader.discover(str(ROOT / "tests"), pattern="*.py", top_level_dir=str(ROOT)))
             if not case.__class__.__module__.startswith("tests.runner_checks.")]
    errors = list(loader.errors)
    rows, objects, modules = {}, {}, {}
    for case in cases:
        module = case.__class__.__module__
        modules[module] = modules.get(module, 0) + 1
        objects[case.id()] = case
        rows[case.id()] = dict(kind="python", module=module, graphics=module not in NON_GRAPHICAL,
                               external=module == EXTERNAL, status="Nicht ausgeführt", seconds=None, artifacts=[])
    if modules.get(EXTERNAL) != 1 or not NON_GRAPHICAL.issubset(modules) or not set(PILOT_MODULES).issubset(modules):
        errors.append("Missing mandatory external, windowless or pilot module.")
    if len(cases) != 75:
        errors.append(f"Expected 75 Python cases, discovered {len(cases)}; reconcile coverage before execution.")
    if any(count != 1 for count in modules.values()):
        errors.append("Exactly one Python test per module is required.")
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    targets = manifest.get("test", [])
    if len(targets) != 10:
        errors.append("Expected ten explicit public Rust integration targets.")
    for target in targets:
        path = ROOT / target["path"]
        names = rust_source_tests(path) if path.is_file() else []
        if len(names) != 1:
            errors.append(f"{path}: expected one real Rust test, found {names}")
        key = "rust::" + target["name"]
        rows[key] = dict(kind="rust", target=target["name"], source=target["path"],
                         function=names[0] if names else "", graphics=target["name"] != "handle_encoding",
                         external=False, status="Nicht ausgeführt", seconds=None, artifacts=[])
    internal = []
    for path in sorted((ROOT / "src").rglob("test_*.rs")):
        names = rust_source_tests(path)
        if not names:
            continue  # test_support.rs is a helper, not a zero-test target.
        if len(names) != 1:
            errors.append(f"{path}: expected one internal test")
        internal += names
        rows["internal::" + names[0]] = dict(kind="internal", function=names[0],
            source=str(path.relative_to(ROOT)), graphics=False, external=False,
            status="Nicht ausgeführt", seconds=None, artifacts=[])
    if len(internal) != 9:
        errors.append("Expected nine internal Rust tests.")
    resource = next((case for case in cases if case.__class__.__module__ == "tests.inspect.resource_value"), None)
    if resource:
        control = type(resource)(resource._testMethodName)
        control.wrong_expected_player_name = "3.18 intentionally wrong player name"
        control._acceptance_id = CONTROL
        objects[CONTROL] = control
        rows[CONTROL] = dict(kind="control", module="tests.inspect.resource_value", graphics=True,
                             external=False, status="Nicht ausgeführt", seconds=None, artifacts=[])
    else:
        errors.append("Missing real resource test for mandatory control 3.18.")
    return rows, objects, [target["name"] for target in targets], errors


class RecordingResult(unittest.TextTestResult):
    def startTest(self, test):
        super().startTest(test)
        self.started_at = time.monotonic()
        self.succeeded = False
        self.failure_types = []

    def addSuccess(self, test):
        self.succeeded = True
        super().addSuccess(test)

    def addFailure(self, test, error):
        self.failure_types.append(error[0])
        super().addFailure(test, error)

    def stopTest(self, test):
        status = "Fehlgeschlagen" if self.failures or self.errors or self.unexpectedSuccesses else (
            "Bestanden" if self.succeeded and not (self.skipped or self.expectedFailures) else "Unvollständig")
        self.row.update(status=status, seconds=round(time.monotonic() - self.started_at, 6),
                        artifacts=getattr(test, "artifact_dirs", []))
        super().stopTest(test)


def logged(command, path, *, timeout=900, env=None):
    with Path(path).open("w", encoding="utf-8") as stream:
        process = subprocess.Popen(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT,
                                   start_new_session=True, env=env)
        forced = False
        try:
            code = process.wait(timeout=timeout)
        finally:
            if process.poll() is None:
                forced = True
                force_owned_tree(process)
            Path(str(path) + ".process.json").write_text(json.dumps({
                "pid": process.pid, "returncode": process.returncode, "forced": forced}) + "\n")
    return code


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    for flag in ("non-graphical", "pilot", "negative-control", "external-github"):
        group.add_argument("--" + flag, action="store_true")
    parser.add_argument("--list", action="store_true", help="planning only; no processes, builds or tests")
    parser.add_argument("--build-only", action="store_true")
    parser.add_argument("--feature-matrix", action="store_true",
                        help="also check all library feature variants and the normal app; automatic for full-local")
    parser.add_argument("--rendered", action="store_true", help="technical confirmation after fresh scoped graphics approval")
    args = parser.parse_args(argv)
    rows, objects, _, errors = inventory()
    selection = "non-graphical" if args.non_graphical else "pilot" if args.pilot else (
        "negative-control" if args.negative_control else "external-github" if args.external_github else "full-local")
    def selected(key, row):
        if args.negative_control:
            return key == CONTROL
        if args.external_github:
            return row["external"]
        if args.pilot:
            return row["kind"] == "python" and row["module"] in PILOT_MODULES
        return not row["external"] and (not args.non_graphical or not row["graphics"])
    keys = [key for key, row in rows.items() if selected(key, row)]
    # Windowless Rust first, then independent windowless Python, then graphics.
    # Keep command-shape rejection checks after the other windowless cases.
    keys.sort(key=lambda key: (rows[key]["graphics"], rows[key]["kind"] in ("python", "control"),
                               rows[key].get("module") == "tests.cli.command_errors", key))
    feature_matrix = args.feature_matrix or selection == "full-local"
    build_commands = commands([rows[key] for key in keys], feature_matrix=feature_matrix)
    if args.list:
        print(json.dumps({"action": "planning only; not acceptance", "selection": selection,
                          "selected": {key: rows[key] for key in keys}, "inventory_errors": errors,
                          "feature_matrix": feature_matrix,
                          "builds": [dict(name=name, command=command) for name, command in build_commands]}, indent=2))
        return 1 if errors else 0
    started = time.monotonic()
    (ROOT / "target/test-runs").mkdir(parents=True, exist_ok=True)
    artifacts = Path(tempfile.mkdtemp(prefix="acceptance-", dir=ROOT / "target/test-runs"))
    report = dict(started_at=datetime.now(timezone.utc).isoformat(), selection=selection,
                  scope="93 local cases plus control 3.18; real Github case separate", artifacts=str(artifacts),
                  metadata={"python": platform.python_version()}, feature_matrix=feature_matrix, builds=[], tests=rows,
                  preflight=[], errors=errors, selected=keys)
    preflight = report["preflight"]
    if os.name != "posix":
        preflight.append("Unix process supervision required.")
    for tool in ("cargo", "rustc", "git"):
        if not shutil.which(tool):
            preflight.append(f"Missing tool: {tool}")
    if not args.build_only and any(rows[key]["graphics"] for key in keys):
        if not args.rendered or os.environ.get("WOODPECKER_TEST_RENDERED") != "1":
            preflight.append("Fresh scoped graphics approval required, then --rendered and WOODPECKER_TEST_RENDERED=1. Flags alone grant no permission.")
        if not args.negative_control and not args.external_github:
            try:
                importlib.import_module("PIL.Image")
            except ImportError:
                preflight.append("Pillow required for rendered PNG cases.")
    if args.external_github and not args.build_only:
        repo = os.environ.get("WOODPECKER_TEST_GITHUB_REPO", "")
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo) or os.environ.get("WOODPECKER_TEST_GITHUB_CLEANUP") != "close":
            preflight.append("Separate approval for named GitHub test repository, one issue and closing it required; set repository/cleanup guards afterward.")
    old_target = os.environ.get("CARGO_TARGET_DIR")
    try:
        if not preflight and not errors:
            os.environ["CARGO_TARGET_DIR"] = str(ROOT / "target")
            for name, command in (("commit", ["git", "rev-parse", "HEAD"]),
                                  ("dirty", ["git", "status", "--porcelain=v1"]),
                                  ("cargo", ["cargo", "--version"]), ("rustc", ["rustc", "--version"])):
                output = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=True, timeout=15).stdout.strip()
                report["metadata"][name] = bool(output) if name == "dirty" else output
            report["metadata"]["app_config"] = tomllib.loads((ROOT / "tests/app.toml").read_text())
            report["metadata"]["settings_seed"] = json.loads((ROOT / "tests/settings.json").read_text())
            binaries = {}
            environment = {**os.environ, "CARGO_TARGET_DIR": str(ROOT / "target")}
            for index, (name, command) in enumerate(build_commands):
                path = artifacts / f"build-{index:02}-{name}.log"
                row = dict(name=name, command=command, log=str(path), status="Fehlgeschlagen")
                report["builds"].append(row)
                code = logged(command, path, env=environment)
                if code:
                    raise RuntimeError(f"Build {name} exited {code}: {path}")
                row["status"] = "Bestanden"
                if name == "rust-tests":
                    binaries = executables(path)
            # Enumerate only selected test binaries; absent, unselected targets are not required.
            if any(rows[key]["kind"] == "internal" for key in keys):
                lib = binaries[("woodpecker", ("lib",))]
                output = subprocess.run([lib, "--list"], capture_output=True, text=True, check=True, timeout=15).stdout
                lib_names = re.findall(r"^(.+): test$", output, re.M)
                expected_internal = sorted(row["function"] for row in rows.values() if row["kind"] == "internal")
                if sorted(name.rsplit("::", 1)[-1] for name in lib_names) != expected_internal:
                    raise RuntimeError(f"Compiled internal tests differ: {lib_names}")
            for key in keys:
                row = rows[key]
                if row["kind"] == "internal":
                    matches = [name for name in lib_names if name.endswith("::" + row["function"])]
                    if len(matches) != 1:
                        raise RuntimeError(f"No unique compiled test for {key}")
                    row["executable"], row["rust_name"] = lib, matches[0]
                elif row["kind"] == "rust":
                    binary = binaries[(row["target"], ("test",))]
                    output = subprocess.run([binary, "--list"], capture_output=True, text=True, check=True, timeout=15).stdout
                    names = re.findall(r"^(.+): test$", output, re.M)
                    if names != [row["function"]]:
                        raise RuntimeError(f"Compiled target {key} must have exactly its declared test: {names}")
                    row["executable"], row["rust_name"] = binary, names[0]
            if not args.build_only:
                unittest.installHandler()
                try:
                    for index, key in enumerate(keys):
                        row = rows[key]
                        if row["kind"] in ("python", "control"):
                            case = objects[key]
                            def make_result(*arguments):
                                result = RecordingResult(*arguments)
                                result.row = row
                                return result
                            with (artifacts / "unittest.log").open("a") as stream:
                                with contextlib.redirect_stdout(stream), contextlib.redirect_stderr(stream):
                                    # The typed negative control intentionally fails once.
                                    # Do not confuse that failfast stop with a real interrupt.
                                    result = unittest.TextTestRunner(stream=stream, verbosity=2, failfast=key != CONTROL,
                                        resultclass=make_result).run(unittest.TestSuite([case]))
                            if key == CONTROL:
                                from tests.inspect.resource_value import ExpectationControlFailure
                                confirmed = (result.testsRun == 1 and len(result.failures) == 1 and
                                    not (result.errors or result.skipped or result.expectedFailures or result.unexpectedSuccesses)
                                    and result.failure_types == [ExpectationControlFailure])
                                row["typed_control_confirmed"] = confirmed
                                row["status"] = "Bestanden" if confirmed else "Fehlgeschlagen"
                        else:
                            path = artifacts / f"rust-{index:02}.log"
                            begin = time.monotonic()
                            code = logged([row["executable"], "--exact", row["rust_name"], "--nocapture", "--test-threads=1"],
                                          path, timeout=1800, env=environment)
                            text = path.read_text()
                            row.update(status="Bestanden" if code == 0 and "1 passed; 0 failed; 0 ignored" in text else "Fehlgeschlagen",
                                       seconds=round(time.monotonic() - begin, 6), log=str(path),
                                       artifacts=re.findall(r"Artefakte: (.+)", text))
                        if row["kind"] in ("python", "control") and result.shouldStop and row["status"] == "Bestanden":
                            preflight.append("Interrupted after active test cleanup.")
                            break
                        if row["status"] != "Bestanden":
                            break  # Never start another game after failure/unsafe cleanup/skip.
                finally:
                    unittest.removeHandler()
    except KeyboardInterrupt:
        preflight.append("Interrupted; no automatic retry.")
    except Exception as error:
        errors.append(f"{type(error).__name__}: {error}")
    finally:
        for key in keys:
            if key in objects:
                try:
                    if not objects[key].doCleanups():
                        raise RuntimeError("Registered cleanup failed")
                except BaseException as error:
                    rows[key]["status"] = "Fehlgeschlagen"
                    errors.append(f"Cleanup {key}: {error}")
    if old_target is None:
        os.environ.pop("CARGO_TARGET_DIR", None)
    else:
        os.environ["CARGO_TARGET_DIR"] = old_target
    local = [row["status"] for row in rows.values() if not row["external"]]
    report["selection_status"] = verdict(rows[key]["status"] for key in keys)
    report["external_github"] = verdict(row["status"] for row in rows.values() if row["external"])
    report["status"] = verdict([*local, *[row["status"] for row in report["builds"]],
                                 *(["Fehlgeschlagen"] if errors else [])],
                                missing=bool(preflight or args.build_only or not report["builds"]))
    report["all_features_status"] = verdict([report["status"], report["external_github"]])
    # An external-only failure still exits 1, even with unexecuted local cases.
    report["exit_status"] = verdict([report["status"], report["selection_status"]])
    report["missing_local_evidence"] = [key for key, row in rows.items() if not row["external"] and row["status"] != "Bestanden"]
    report["features"] = {}
    for key, row in rows.items():
        feature = row.get("module", row.get("source", "rust")).split(".")[1] if row["kind"] in ("python", "control") else row["kind"]
        report["features"].setdefault(feature, []).append(key)
    report["features"] = {feature: {
        "tests": names, "status": verdict(rows[name]["status"] for name in names),
        "artifacts": [path for name in names for path in rows[name]["artifacts"]],
    } for feature, names in report["features"].items()}
    report["seconds"] = round(time.monotonic() - started, 6)
    path = artifacts / "summary.json"
    path.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")
    print(f"Auswahl {selection}: {report['selection_status']}; lokale Gesamtabnahme: {report['status']}; GitHub: {report['external_github']}")
    print(f"Artefakte: {path}")
    for message in [*preflight, *errors]:
        print(message)
    return {"Bestanden": 0, "Fehlgeschlagen": 1, "Unvollständig": 2}[report["exit_status"]]


if __name__ == "__main__":
    sys.exit(main())
