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

from tests._support import ROOT, force_owned_tree, use_products
from tests import _artifacts, _audits, _compiler_cache, _shared
from tests._builds import ABORT_TARGET, app_build, commands, executables, rust_source_tests, validate_app_build

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
    "tests.reports.before_ready", "tests.shutdown.starting", "tests.reports.panic_abort",
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
    if (modules.get(EXTERNAL) != 1 or modules.get("tests.headless.basis") != 1
            or not NON_GRAPHICAL.issubset(modules) or not set(PILOT_MODULES).issubset(modules)
            or not _shared.MODULES.issubset(modules)):
        errors.append("Missing mandatory external, windowless, pilot or L1 input/basis module.")
    if len(cases) != 78:
        errors.append(f"Expected 78 Python cases, discovered {len(cases)}; reconcile coverage before execution.")
    if any(count != 1 for count in modules.values()):
        errors.append("Exactly one Python test per module is required.")
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    targets = manifest.get("test", [])
    if len(targets) != 1 or targets[0]["name"] != "contracts":
        errors.append("Expected one shared public Rust contract target.")
    suite = ROOT / "tests/rust/contracts.rs"
    targets = [dict(name=name, path="tests/rust/" + path) for path, name in
               re.findall(r'#\[path = "([^"]+)"\]\s*mod (\w+);', suite.read_text())]
    if len(targets) != 10:
        errors.append("Expected ten isolated public Rust contract cases.")
    for target in targets:
        path = ROOT / target["path"]
        names = rust_source_tests(path) if path.is_file() else []
        if len(names) != 1:
            errors.append(f"{path}: expected one real Rust test, found {names}")
        key = "rust::" + target["name"]
        rows[key] = dict(kind="rust", target="contracts", case=target["name"], source=target["path"],
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
    if len(internal) != 14:
        errors.append("Expected fourteen internal Rust tests.")
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


def main(argv=None, *, shared_game=None, should_stop=None):
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group()
    for flag in ("non-graphical", "pilot", "negative-control", "external-github", "headless"):
        group.add_argument("--" + flag, action="store_true",
                           **({"help": "scoped L1 headless input/basis run; requires GPU approval"} if flag == "headless" else {}))
    group.add_argument("--test", action="append", dest="tests", metavar="ID",
                       help="select an exact inventory ID or Python test module; repeatable")
    group.add_argument("--category", action="append", dest="categories", metavar="NAME",
                       help="select a Python category, rust, or internal; repeatable")
    parser.add_argument("--list", action="store_true", help="planning only; no processes, builds or tests")
    parser.add_argument("--build-only", action="store_true", help="prepare products and write an executable manifest")
    parser.add_argument("--artifacts", type=Path, metavar="MANIFEST", help="reuse prepared executables; never build")
    parser.add_argument("--compact", action="store_true", help="checkpoint compiler outputs after --build-only; 5 GB idle budget")
    parser.add_argument("--feature-matrix", action="store_true",
                        help="also check all library feature variants and the normal app; automatic for full-local")
    parser.add_argument("--rendered", action="store_true", help="technical confirmation after fresh scoped graphics approval")
    args = parser.parse_args(argv)
    if args.build_only and args.artifacts:
        parser.error("--build-only and --artifacts are mutually exclusive")
    if args.compact and not args.build_only:
        parser.error("--compact requires --build-only")
    if args.headless and args.feature_matrix:
        parser.error("--headless is the scoped L1 run, not a full feature-matrix audit")
    rows, objects, _, errors = inventory()
    selection = "non-graphical" if args.non_graphical else "pilot" if args.pilot else (
        "negative-control" if args.negative_control else "external-github" if args.external_github else
        "headless-l1" if args.headless else "tests" if args.tests else "categories" if args.categories else "full-local")
    def category(row):
        return row["module"].split(".")[1] if row["kind"] in ("python", "control") else row["kind"]
    for value in args.tests or []:
        if not any(value in (key, row.get("module")) for key, row in rows.items()):
            errors.append(f"Unknown test: {value}")
    for value in args.categories or []:
        if value not in {category(row) for row in rows.values()}:
            errors.append(f"Unknown category: {value}")
    def headless_only(row):
        return row.get("module", "").startswith("tests.headless.")
    if not args.headless and (any(headless_only(row) and (key in (args.tests or []) or row.get("module") in (args.tests or []))
                                 for key, row in rows.items()) or "headless" in (args.categories or [])):
        errors.append("Headless-only owners require the explicit --headless L1 mode.")
    def selected(key, row):
        if args.headless:
            return row.get("module") in _shared.MODULES or headless_only(row)
        if headless_only(row):
            return False
        if args.tests:
            return (key in args.tests or (row["kind"] != "control" and row.get("module") in args.tests)) and not row["external"]
        if args.categories:
            return category(row) in args.categories and row["kind"] != "control" and not row["external"]
        if args.negative_control:
            return key == CONTROL
        if args.external_github:
            return row["external"]
        if args.pilot:
            return row["kind"] == "python" and row["module"] in PILOT_MODULES
        return not row["external"] and (not args.non_graphical or not row["graphics"])
    keys = [key for key, row in rows.items() if selected(key, row)]
    # Keep compatible games together so the warm process ends before isolated cases.
    keys.sort(key=lambda key: (rows[key]["graphics"],
                               rows[key].get("module") not in _shared.MODULES,
                               rows[key]["kind"] in ("python", "control"),
                               rows[key].get("module") == "tests.cli.command_errors", key))
    if not keys and not errors:
        errors.append("Selection contains no runnable tests; external GitHub cases require --external-github")
    feature_matrix = args.feature_matrix or selection == "full-local"
    build_commands = [] if args.artifacts else commands([rows[key] for key in keys], feature_matrix=feature_matrix, headless=args.headless)
    if args.list:
        print(json.dumps({"action": "planning only; not acceptance", "selection": selection,
                          "selected": {key: rows[key] for key in keys}, "inventory_errors": errors,
                          "feature_matrix": feature_matrix,
                          "prepared_manifest": str(args.artifacts) if args.artifacts else None,
                          "shared_application": [key for key in keys if rows[key].get("module") in _shared.MODULES],
                          "builds": [dict(name=name, command=command) for name, command in build_commands]}, indent=2))
        return 1 if errors else 0
    started = time.monotonic()
    (ROOT / "target/test-runs").mkdir(parents=True, exist_ok=True)
    artifacts = Path(tempfile.mkdtemp(prefix="acceptance-", dir=ROOT / "target/test-runs"))
    report = dict(started_at=datetime.now(timezone.utc).isoformat(), selection=selection,
                  scope=("L1 headless subset; not full-suite acceptance" if args.headless else
                         "Native local cases plus control 3.18; headless and real GitHub separate"), artifacts=str(artifacts),
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
        if any(rows[key].get("module") in ("tests.screenshot.capture", "tests.screenshot.overwrite", "tests.headless.basis") for key in keys):
            try:
                importlib.import_module("PIL.Image")
            except ImportError:
                preflight.append("Pillow required for rendered PNG cases.")
    if args.external_github and not args.build_only:
        repo = os.environ.get("WOODPECKER_TEST_GITHUB_REPO", "")
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repo) or os.environ.get("WOODPECKER_TEST_GITHUB_CLEANUP") != "close":
            preflight.append("Separate approval for named GitHub test repository, one issue and closing it required; set repository/cleanup guards afterward.")
    old_target = os.environ.get("CARGO_TARGET_DIR")
    product_scope = contextlib.ExitStack()
    audit_scope = contextlib.ExitStack()
    owns_game = shared_game is None
    if shared_game is not None and shared_game.headless != args.headless:
        preflight.append("Shared application profile does not match the requested runner mode.")
    shared_game = shared_game or _shared.SharedGame(headless=args.headless)
    shared_remaining = {key for key in keys if rows[key].get("module") in _shared.MODULES}
    report["shared_application"] = []
    restored_for_fixture = False
    try:
        if not preflight and not errors:
            os.environ["CARGO_TARGET_DIR"] = str(ROOT / "target")
            if not args.artifacts and _compiler_cache.packed(ROOT):
                _compiler_cache.restore(ROOT)
            for name, command in (("commit", ["git", "rev-parse", "HEAD"]),
                                  ("dirty", ["git", "status", "--porcelain=v1"]),
                                  ("cargo", ["cargo", "--version"]), ("rustc", ["rustc", "--version"])):
                output = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, check=True, timeout=15).stdout.strip()
                report["metadata"][name] = bool(output) if name == "dirty" else output
            report["metadata"]["app_config"] = tomllib.loads((ROOT / "tests/app.toml").read_text())
            report["metadata"]["settings_seed"] = json.loads((ROOT / "tests/settings.json").read_text())
            binaries = {}
            environment = {**os.environ, "CARGO_TARGET_DIR": str(ROOT / "target")}
            products = {}
            build_logs = []
            build_inputs = _artifacts.inputs(root=ROOT) if args.build_only or args.headless or feature_matrix else None
            cached_audits = _audits.load(ROOT, build_inputs) if feature_matrix else {}
            audit_builds = {}
            audit_target = None
            if args.artifacts:
                products = _artifacts.load(args.artifacts, root=ROOT)
                missing = _artifacts.required(rows[key] for key in keys) - products.keys()
                if missing:
                    raise ValueError(f"Prepared manifest is missing products: {sorted(missing)}")
                manifest_data = json.loads(args.artifacts.read_text())
                if "bin:test_app" in _artifacts.required(rows[key] for key in keys):
                    validate_app_build(manifest_data.get("app_build"), headless=args.headless)
                if feature_matrix and not manifest_data.get("feature_matrix"):
                    raise ValueError("Prepared manifest does not prove the requested feature matrix")
                report["products_manifest"] = str(args.artifacts.resolve())
                report["builds"].append(dict(name="prepared-artifacts", status="Bestanden", compiled=False))
                binaries = {(role.split(":")[1], tuple(role.split(":")[2].split(","))): str(path)
                            for role, path in products.items() if role.startswith("test:")}
            for index, (name, command) in enumerate(build_commands):
                if "--message-format=json" not in command:
                    command = [*command, "--message-format=json"]
                path = artifacts / f"build-{index:02}-{name}.log"
                row = dict(name=name, command=command, log=str(path), status="Fehlgeschlagen")
                report["builds"].append(row)
                is_audit = name.startswith("library-") or name == "app-normal"
                cached = cached_audits.get(name)
                if is_audit and cached and cached["command"] == command:
                    row.update(status="Bestanden", compiled=False, log=cached["log"], reused_audit=True)
                    audit_builds[name] = cached
                    build_logs.append((name, Path(cached["log"])))
                    continue
                if is_audit and audit_target is None:
                    audit_target = audit_scope.enter_context(_audits.workspace(ROOT))
                build_environment = ({**environment, "CARGO_TARGET_DIR": str(ABORT_TARGET),
                                      "CARGO_PROFILE_DEV_PANIC": "abort"} if name == "app-abort" else
                                     {**environment, "CARGO_TARGET_DIR": audit_target} if is_audit else environment)
                code = logged(command, path, env=build_environment, timeout=1800 if name in ("app-abort", "app-normal") else 900)
                if code:
                    raise RuntimeError(f"Build {name} exited {code}: {path}")
                row["status"] = "Bestanden"
                if name == "rust-tests":
                    binaries = executables(path)
                build_logs.append((name, path))
                if is_audit:
                    audit_builds[name] = dict(command=command, log=str(path))
            if feature_matrix and not args.artifacts:
                _audits.publish(ROOT, build_inputs, audit_builds)
            audit_scope.close()
            if (args.build_only or args.headless) and not args.artifacts:
                manifest = artifacts / "products.json"
                roles = _artifacts.required(rows[key] for key in keys)
                proof = app_build(build_logs, headless=args.headless) if "bin:test_app" in roles else None
                products = _artifacts.publish(build_logs, manifest, root=ROOT, feature_matrix=feature_matrix,
                                             expected_inputs=build_inputs, roles=roles)
                if proof is not None:
                    data = json.loads(manifest.read_text())
                    data["app_build"] = proof
                    temporary = manifest.with_suffix(".preparing")
                    temporary.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
                    temporary.replace(manifest)
                report["products_manifest"] = str(manifest)
            product_scope.enter_context(use_products(products))
            environment.update({key: value for key, value in os.environ.items() if key in
                                ("WOODPECKER_TEST_APP", "WOODPECKER_TEST_ABORT_APP")})
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
                    expected = sorted(f"{item['case']}::{item['function']}" for item in rows.values()
                                      if item["kind"] == "rust") if row.get("case") else [row["function"]]
                    if sorted(names) != expected:
                        raise RuntimeError(f"Compiled target {key} must have exactly its declared tests: {names}")
                    row["executable"] = binary
                    row["rust_name"] = f"{row['case']}::{row['function']}" if row.get("case") else names[0]
            if not args.build_only:
                unittest.installHandler()
                try:
                    for index, key in enumerate(keys):
                        if should_stop is not None and should_stop():
                            preflight.append("Client disconnected; no further cases started.")
                            break
                        row = rows[key]
                        if (row.get("case") == "report_snapshot" or row.get("module") == "tests.recording.io_failure"):
                            if _compiler_cache.packed(ROOT):
                                _compiler_cache.restore(ROOT)
                                restored_for_fixture = True
                        if row["kind"] in ("python", "control"):
                            case = objects[key]
                            if key in shared_remaining:
                                case.shared_game = shared_game
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
                            if hasattr(case, "shared_app_observation"):
                                row["application"] = case.shared_app_observation
                                report["shared_application"].append(row["application"])
                            if key in shared_remaining:
                                shared_remaining.remove(key)
                                if not shared_remaining and owns_game:
                                    shared_game.close()
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
                            if row.get("module") in _shared.MODULES:
                                shared_game.failed = True
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
    if owns_game:
        try:
            shared_game.close()
        except BaseException as error:
            errors.append(f"Shared application cleanup: {type(error).__name__}: {error}")
    audit_scope.close()
    product_scope.close()
    if old_target is None:
        os.environ.pop("CARGO_TARGET_DIR", None)
    else:
        os.environ["CARGO_TARGET_DIR"] = old_target
    if (args.compact or restored_for_fixture) and not errors and not preflight and report.get("products_manifest"):
        try:
            preparation = artifacts / "preparation.json"
            preparation.write_text(json.dumps(report, ensure_ascii=False) + "\n")
            if args.compact and selection == "full-local":
                from tests import _cargo_cache
                _cargo_cache.sweep(preparation, apply=True, root=ROOT)
            _compiler_cache.compact(ROOT, Path(report["products_manifest"]))
            report["compiler_cache"] = "lossless idle checkpoint"
            if args.compact:
                kib = int(subprocess.check_output(["du", "-sk", str(ROOT / "target")], text=True).split()[0])
                report["target_bytes"] = kib * 1024
                if kib * 1024 > 5_000_000_000:
                    raise RuntimeError(f"Idle target exceeds 5 GB: {kib * 1024 / 1e9:.3f} GB")
        except Exception as error:
            errors.append(f"Compiler checkpoint: {type(error).__name__}: {error}")
    local_keys = [key for key, row in rows.items() if not row["external"] and (args.headless or not headless_only(row))]
    local = [rows[key]["status"] for key in local_keys]
    report["selection_status"] = verdict(rows[key]["status"] for key in keys)
    report["external_github"] = verdict(row["status"] for row in rows.values() if row["external"])
    report["status"] = verdict([*local, *[row["status"] for row in report["builds"]],
                                 *(["Fehlgeschlagen"] if errors else [])],
                                missing=bool(preflight or args.build_only or args.headless or not report["builds"]))
    report["headless_status"] = report["selection_status"] if args.headless else "Unvollständig"
    report["all_features_status"] = verdict([report["status"], report["external_github"], report["headless_status"]])
    # An external-only failure still exits 1, even with unexecuted local cases.
    report["exit_status"] = verdict([report["status"], report["selection_status"]])
    report["missing_local_evidence"] = [key for key in local_keys if rows[key]["status"] != "Bestanden"]
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
    if report.get("products_manifest"):
        print(f"Produkte: {report['products_manifest']}")
    for message in [*preflight, *errors]:
        print(message)
    return {"Bestanden": 0, "Fehlgeschlagen": 1, "Unvollständig": 2}[report["exit_status"]]


if __name__ == "__main__":
    sys.exit(main())
