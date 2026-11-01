"""Offline build-selection/cache regressions; never build or start a game."""
import contextlib
import io
import json
import os
import subprocess
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

from tests import acceptance
from tests import _builds
from tests._support import ROOT
from tests.reports import panic_abort


def python_row(module, *, graphics=True, external=False, kind="python"):
    return dict(kind=kind, module=module, graphics=graphics, external=external)


class BuildSelectionTest(unittest.TestCase):
    def test_only_selected_dependencies_are_built(self):
        examples = [
            ([python_row("tests.cli.command_errors", graphics=False)], ["cli"], [], False),
            ([python_row("tests.pointer.ui_click"), python_row("tests.text.input_limits")],
             ["cli", "app-automation"], [], False),
            ([python_row("tests.inspect.resource_value", kind="control")],
             ["cli", "app-automation"], [], False),
            ([python_row("tests.reports.panic_abort")], ["cli"], [], False),
            ([python_row("tests.providers.github_live", external=True)],
             ["app-automation", "github-live-driver"], [], False),
            ([dict(kind="rust", target="handle_encoding", graphics=False)],
             ["rust-tests"], ["handle_encoding"], False),
            ([dict(kind="internal", graphics=False)], ["rust-tests"], [], True),
            ([python_row("tests.cli.command_errors", graphics=False),
              dict(kind="internal", graphics=False),
              dict(kind="rust", target="handle_encoding", graphics=False)],
             ["cli", "rust-tests"], ["handle_encoding"], True),
            ([], [], [], False),
        ]
        for selected, expected, targets, library in examples:
            with self.subTest(selected=selected):
                commands = _builds.commands(selected)
                self.assertEqual([name for name, _ in commands], expected)
                for name, command in commands:
                    if name == "rust-tests":
                        self.assertEqual("--lib" in command, library)
                        self.assertEqual([command[i + 1] for i, arg in enumerate(command) if arg == "--test"], targets)
                        self.assertIn("--no-run", command)

    def test_explicit_feature_matrix_preserves_checks_and_app_order(self):
        commands = _builds.commands([python_row("tests.pointer.ui_click")], feature_matrix=True)
        names = [name for name, _ in commands]
        self.assertEqual(names[:7], ["library-" + name for name in (
            "default", "ui", "screenshot", "ui,screenshot", "server", "client", "cli")])
        self.assertLess(names.index("app-normal"), names.index("app-automation"))
        self.assertNotIn("rust-tests", names)
        self.assertNotIn("github-live-driver", names)

    def test_unknown_graphical_case_gets_automation_build(self):
        self.assertEqual([name for name, _ in _builds.commands([python_row("tests.future.case")])],
                         ["cli", "app-automation"])

    def test_product_inventory_excludes_runner_self_checks(self):
        rows, _, targets, errors = acceptance.inventory()
        self.assertEqual(errors, [])
        self.assertEqual(len(rows), 95)
        self.assertEqual(sum(row["kind"] == "python" for row in rows.values()), 75)
        self.assertEqual(len(targets), 10)
        self.assertFalse(any(row.get("module", "").startswith("tests.runner_checks.") for row in rows.values()))

    def test_list_shows_builds_without_processes_or_creating_artifacts(self):
        examples = [
            (["--non-graphical"], ["cli", "rust-tests"], False),
            (["--pilot"], ["cli", "app-automation"], False),
            (["--negative-control"], ["cli", "app-automation"], False),
            (["--external-github"], ["app-automation", "github-live-driver"], False),
            ([], None, True),
            (["--non-graphical", "--feature-matrix"], None, True),
        ]
        for flags, expected, matrix in examples:
            with self.subTest(flags=flags), patch.object(acceptance.subprocess, "run") as run, \
                    patch.object(acceptance, "logged") as logged, patch.object(Path, "mkdir") as mkdir:
                output = io.StringIO()
                with contextlib.redirect_stdout(output):
                    self.assertEqual(acceptance.main([*flags, "--list"]), 0)
                plan = json.loads(output.getvalue())
                names = [row["name"] for row in plan["builds"]]
                self.assertEqual(plan["feature_matrix"], matrix)
                if expected is not None:
                    self.assertEqual(names, expected)
                if not flags:
                    self.assertEqual(len(names), 11)
                    self.assertNotIn("github-live-driver", names)
                run.assert_not_called()
                logged.assert_not_called()
                mkdir.assert_not_called()

    def run_subset(self, rows, *, library=False, listing=None, expected_code=2):
        """Exercise main/report/binary binding, replacing all foreign processes."""
        rows = {**rows, "not-selected": dict(kind="python", module="tests.future.window",
            graphics=True, external=False, status="Nicht ausgeführt", artifacts=[])}
        with tempfile.TemporaryDirectory() as directory:
            output = io.StringIO()
            calls = []
            def logged(command, path, **kwargs):
                calls.append(command)
                if "--no-run" in command:
                    artifacts = []
                    if library:
                        artifacts.append(dict(reason="compiler-artifact", executable="fake-library",
                            target=dict(name="woodpecker", kind=["lib"]), profile=dict(test=True)))
                    if "--test" in command:
                        artifacts.append(dict(reason="compiler-artifact", executable="fake-handle",
                            target=dict(name="handle_encoding", kind=["test"]), profile=dict(test=True)))
                    Path(path).write_text("\n".join(json.dumps(row) for row in artifacts))
                else:
                    Path(path).write_text("test result: ok. 1 passed; 0 failed; 0 ignored\n")
                return 0
            def run(command, **kwargs):
                if "--list" in command and listing is not None:
                    text = listing
                elif command == ["fake-library", "--list"]:
                    text = "tests::chosen: test"
                elif command == ["fake-handle", "--list"]:
                    text = "handle_encoding_contract: test\n"
                else:
                    text = "test-version"
                return Mock(stdout=text)
            with patch.object(acceptance, "inventory", return_value=(rows, {}, [], [])), \
                    patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(acceptance.shutil, "which", side_effect=lambda name: name), \
                    patch.object(acceptance.subprocess, "run", side_effect=run), \
                    patch.object(acceptance, "logged", side_effect=logged), contextlib.redirect_stdout(output):
                code = acceptance.main(["--non-graphical"])
            report = json.loads((Path(directory) / "summary.json").read_text())
            self.assertEqual(code, expected_code)  # A passing subset is still not full acceptance.
            if expected_code == 2:
                self.assertEqual(report["errors"], [])
                self.assertEqual(report["selection_status"], "Bestanden")
                self.assertEqual(report["status"], "Unvollständig")
            else:
                self.assertTrue(report["errors"])
                self.assertEqual(report["status"], "Fehlgeschlagen")
            return calls, report

    def test_rust_only_subset_does_not_require_unbuilt_library_or_other_targets(self):
        row = dict(kind="rust", target="handle_encoding", function="handle_encoding_contract",
                   graphics=False, external=False, status="Nicht ausgeführt", artifacts=[])
        calls, report = self.run_subset({"rust::handle_encoding": row})
        self.assertEqual([row["name"] for row in report["builds"]], ["rust-tests"])
        self.assertNotIn("--lib", calls[0])
        self.assertEqual(row["executable"], "fake-handle")
        self.assertEqual(calls[1][1:3], ["--exact", "handle_encoding_contract"])

    def test_internal_only_subset_does_not_require_integration_binaries(self):
        row = dict(kind="internal", function="chosen", graphics=False, external=False,
                   status="Nicht ausgeführt", artifacts=[])
        calls, report = self.run_subset({"internal::chosen": row}, library=True)
        self.assertEqual([row["name"] for row in report["builds"]], ["rust-tests"])
        self.assertIn("--lib", calls[0])
        self.assertNotIn("--test", calls[0])
        self.assertEqual(row["rust_name"], "tests::chosen")

    def test_wrong_compiled_lists_still_fail_before_test_execution(self):
        for kind, target, function in [("internal", None, "chosen"), ("rust", "handle_encoding", "handle_encoding_contract")]:
            with self.subTest(kind=kind):
                row = dict(kind=kind, target=target, function=function, graphics=False, external=False,
                           status="Nicht ausgeführt", artifacts=[])
                calls, report = self.run_subset({"selected": row}, library=kind == "internal",
                                               listing="wrong_test: test\n", expected_code=1)
                self.assertIn("Compiled", report["errors"][0])
                self.assertEqual(len(calls), 1)
                self.assertEqual(row["status"], "Nicht ausgeführt")

    def test_python_build_only_never_enumerates_rust_binaries(self):
        with tempfile.TemporaryDirectory() as directory:
            row = python_row("tests.session.server_lifecycle", graphics=False)
            row.update(status="Nicht ausgeführt", artifacts=[])
            with patch.object(acceptance, "inventory", return_value=({"case": row}, {}, [], [])), \
                    patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(acceptance.shutil, "which", side_effect=lambda name: name), \
                    patch.object(acceptance.subprocess, "run", return_value=Mock(stdout="version")) as run, \
                    patch.object(acceptance, "logged", return_value=0) as logged, \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(acceptance.main(["--non-graphical", "--build-only"]), 2)
            report = json.loads((Path(directory) / "summary.json").read_text())
            self.assertEqual(report["errors"], [])
            self.assertEqual([row["name"] for row in report["builds"]], ["cli"])
            self.assertEqual(logged.call_count, 1)
            self.assertFalse(any("--list" in call.args[0] for call in run.call_args_list))

    def test_ci_shell_requires_passing_subset_and_expected_exit(self):
        workflow = (ROOT / ".github/workflows/contracts.yml").read_text()
        block = workflow.split("      - name: Common acceptance entry (no games or real GitHub)\n", 1)[1]
        script_lines = block.split("        run: |\n", 1)[1].splitlines()
        script = []
        for line in script_lines:
            if not line.startswith("          "):
                break
            script.append(line[10:])
        self.assertTrue(script)
        passed = "Auswahl non-graphical: Bestanden; lokale Gesamtabnahme: Unvollständig; GitHub: Unvollständig"
        for status, output, expected in ((2, passed, 0), (1, passed, 1), (0, passed, 1),
                                         (2, passed.replace("Auswahl non-graphical: Bestanden",
                                                            "Auswahl non-graphical: Unvollständig"), 1)):
            with self.subTest(status=status, output=output), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                binary = root / "python3"
                binary.write_text('#!/bin/sh\nprintf "%s\\n" "$CASE_OUTPUT"\nexit "$CASE_STATUS"\n')
                binary.chmod(0o700)
                environment = {**os.environ, "PATH": str(root) + os.pathsep + os.environ["PATH"],
                               "CASE_OUTPUT": output, "CASE_STATUS": str(status)}
                result = subprocess.run(["bash", "-e", "-o", "pipefail", "-c", "\n".join(script)],
                                        cwd=root, env=environment, capture_output=True, text=True, timeout=5)
                self.assertEqual(result.returncode, expected, result.stdout + result.stderr)

    def test_expected_control_continues_but_wrong_failure_cleanup_or_interrupt_stops(self):
        from tests.inspect.resource_value import ExpectationControlFailure

        for mode in ("expected", "wrong-failure", "cleanup-error", "interrupt"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                visited = []

                class Control(unittest.TestCase):
                    def runTest(self):
                        if mode == "cleanup-error":
                            self.addCleanup(lambda: (_ for _ in ()).throw(RuntimeError("cleanup fault")))
                        if mode == "interrupt":
                            self._outcome.result.stop()  # Same public stop signal as unittest's handler.
                        if mode == "wrong-failure":
                            raise AssertionError("not the intended control")
                        raise ExpectationControlFailure("intended wrong expectation")

                class Later(unittest.TestCase):
                    def runTest(self):
                        visited.append("later")

                control = dict(kind="control", module="tests.inspect.resource_value", graphics=True,
                               external=False, status="Nicht ausgeführt", artifacts=[])
                later = dict(kind="python", module="tests.session.session_lifecycle", graphics=True,
                             external=False, status="Nicht ausgeführt", artifacts=[])
                rows = {acceptance.CONTROL: control, "later": later}
                objects = {acceptance.CONTROL: Control(), "later": Later()}
                with patch.dict(os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                        patch.object(acceptance, "inventory", return_value=(rows, objects, [], [])), \
                        patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                        patch.object(acceptance.shutil, "which", side_effect=lambda name: name), \
                        patch.object(acceptance.subprocess, "run", return_value=Mock(stdout="version")), \
                        patch.object(acceptance.importlib, "import_module", return_value=Mock()), \
                        patch.object(acceptance, "logged", return_value=0), contextlib.redirect_stdout(io.StringIO()):
                    code = acceptance.main(["--rendered"])
                report = json.loads((Path(directory) / "summary.json").read_text())
                self.assertEqual(code, 0 if mode == "expected" else 2 if mode == "interrupt" else 1)
                self.assertEqual(visited, ["later"] if mode == "expected" else [])
                self.assertEqual(control["typed_control_confirmed"], mode in ("expected", "interrupt"))
                self.assertEqual(bool(report["preflight"]), mode == "interrupt")

    def test_graphics_guard_still_blocks_before_builds(self):
        with tempfile.TemporaryDirectory() as directory:
            with patch.dict(os.environ, {"WOODPECKER_TEST_RENDERED": "0"}), \
                    patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(acceptance, "logged") as logged, \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(acceptance.main(["--pilot"]), 2)
            report = json.loads((Path(directory) / "summary.json").read_text())
            self.assertEqual(report["errors"], [])
            self.assertTrue(any("Fresh scoped graphics approval" in item for item in report["preflight"]))
            logged.assert_not_called()


class AbortCacheTest(unittest.TestCase):
    def test_repeated_runs_share_only_build_cache_not_logs_or_server_environment(self):
        class BuildFinished(Exception):
            pass
        environments = []
        original_target = os.environ.get("CARGO_TARGET_DIR")
        original_panic = os.environ.get("CARGO_PROFILE_DEV_PANIC")
        with tempfile.TemporaryDirectory() as directory:
            for index in range(2):
                artifacts = Path(directory) / str(index)
                artifacts.mkdir()
                server = Mock(artifacts=artifacts)
                server.process.wait.return_value = 0
                process = Mock()
                process.wait.return_value = 0
                process.poll.return_value = 0
                case = panic_abort.PanicAbortTest()
                try:
                    with patch.dict(os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                            patch.object(panic_abort, "start_server", side_effect=[server, BuildFinished]) as start, \
                            patch.object(panic_abort.subprocess, "Popen", return_value=process) as popen, \
                            patch.object(Path, "is_file", return_value=True):
                        with self.assertRaises(BuildFinished):
                            case.test_abort_build_reports_panic_once_and_not_process_exit()
                        environments.append(popen.call_args.kwargs["env"])
                        metadata = json.loads((artifacts / "abort-build.json").read_text())
                        self.assertEqual(metadata["CARGO_TARGET_DIR"], str(ROOT / "target/panic-abort"))
                        self.assertEqual(start.call_args.kwargs["env"], {
                            "CARGO_TARGET_DIR": str(ROOT / "target/panic-abort"), "CARGO_PROFILE_DEV_PANIC": "abort"})
                finally:
                    self.assertTrue(case.doCleanups())
                self.assertFalse((artifacts / "abort-target").exists())
        self.assertEqual(environments[0]["CARGO_TARGET_DIR"], environments[1]["CARGO_TARGET_DIR"])
        self.assertEqual(environments[0]["CARGO_PROFILE_DEV_PANIC"], "abort")
        self.assertEqual(os.environ.get("CARGO_TARGET_DIR"), original_target)
        self.assertEqual(os.environ.get("CARGO_PROFILE_DEV_PANIC"), original_panic)

    def test_abort_graphics_guard_precedes_server_and_build(self):
        case = panic_abort.PanicAbortTest()
        with patch.dict(os.environ, {"WOODPECKER_TEST_RENDERED": "0"}), \
                patch.object(panic_abort, "start_server") as start, \
                patch.object(panic_abort.subprocess, "Popen") as popen:
            with self.assertRaises(AssertionError):
                case.test_abort_build_reports_panic_once_and_not_process_exit()
        start.assert_not_called()
        popen.assert_not_called()
