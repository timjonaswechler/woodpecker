"""CPU-only checks of the opt-in L1 runner; no Cargo or GPU/app execution."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
from unittest.mock import Mock, patch

from tests import acceptance, _builds, _shared, _support


class HeadlessRunnerTest(unittest.TestCase):
    def test_headless_plan_reuses_all_original_input_owners_without_side_effects(self):
        with patch.object(acceptance.subprocess, "run") as run, \
                patch.object(acceptance.subprocess, "Popen") as popen, \
                patch.object(Path, "mkdir") as mkdir, \
                contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--headless", "--list"]), 0)
        plan = json.loads(output.getvalue())
        modules = {row["module"] for row in plan["selected"].values()}
        self.assertEqual(modules, _shared.MODULES | {"tests.headless.basis", "tests.headless.startup", "tests.headless.terminal"})
        self.assertEqual(len(plan["shared_application"]), 10)
        self.assertEqual(plan["selection"], "headless-l1")
        self.assertFalse(plan["feature_matrix"])
        self.assertTrue(all(row["graphics"] for row in plan["selected"].values()))
        run.assert_not_called()
        popen.assert_not_called()
        mkdir.assert_not_called()

    def test_persistent_native_host_rejects_a_headless_manifest_before_starting(self):
        from tests import interactive
        with tempfile.TemporaryDirectory() as directory:
            manifest = Path(directory) / "products.json"
            manifest.write_text(json.dumps({"app_build": {"mode": "headless", "features": ["automation"]}}))
            with patch.object(interactive, "STATE", Path(directory) / "state.json"), \
                    patch.object(interactive._artifacts, "load", return_value={"bin:woodpecker": Path("/never/cli"), "bin:test_app": Path("/never/app")}), \
                    patch.object(interactive._shared, "SharedGame") as game:
                with self.assertRaisesRegex(ValueError, "requested native variant"):
                    interactive.host(manifest)
                game.assert_not_called()

    def test_headless_builds_isolate_the_app_from_native_feature_unification(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--headless", "--list"]), 0)
        builds = json.loads(output.getvalue())["builds"]
        self.assertEqual(builds, [
            {"name": "cli", "command": ["cargo", "build", "--locked", "-p", "woodpecker_cli", "--bin", "woodpecker"]},
            {"name": "app-headless", "command": ["cargo", "build", "--locked", "-p", "test_app",
                "--no-default-features", "--features", "automation", "--bin", "test_app"]},
        ])
        normal = dict(_builds.commands([], feature_matrix=True))["app-normal"]
        self.assertNotIn("--no-default-features", normal, "Normal app audit must retain the native default")

    def test_headless_cargo_proof_rejects_native_features_or_missing_automation(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory) / "app.jsonl"
            for features in (["automation", "default", "native"], ["automation", "native"], [], ["default"]):
                with self.subTest(features=features):
                    log.write_text(json.dumps({"reason": "compiler-artifact", "executable": "/not-executed/test_app",
                        "target": {"name": "test_app", "kind": ["bin"]}, "profile": {"test": False},
                        "features": features}) + "\n")
                    with self.assertRaisesRegex(ValueError, "does not prove the requested headless"):
                        _builds.app_build([("app-headless", log)], headless=True)
            with self.assertRaisesRegex(ValueError, "Missing Cargo feature proof"):
                _builds.app_build([("app-automation", log)], headless=True)

    def test_headless_cannot_be_combined_with_native_selection_or_feature_audits(self):
        for flags in (["--non-graphical"], ["--pilot"], ["--test", "tests.pointer.ui_click"],
                      ["--category", "keyboard"], ["--feature-matrix"]):
            with self.subTest(flags=flags), patch.object(acceptance.subprocess, "Popen") as popen, \
                    patch.object(Path, "mkdir") as mkdir, contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    acceptance.main(["--headless", "--list", *flags])
                self.assertEqual(error.exception.code, 2)
                popen.assert_not_called()
                mkdir.assert_not_called()
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--list", "--category", "headless"]), 1)
        self.assertIn("explicit --headless", json.loads(output.getvalue())["inventory_errors"][0])

    def test_list_with_headless_prepared_manifest_does_not_open_or_validate_it(self):
        with patch.object(acceptance._artifacts, "load") as load, \
                patch.object(acceptance.subprocess, "run") as run, \
                patch.object(Path, "mkdir") as mkdir, contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--headless", "--list", "--artifacts", "/missing/products.json"]), 0)
        plan = json.loads(output.getvalue())
        self.assertEqual(plan["builds"], [])
        self.assertEqual(plan["prepared_manifest"], "/missing/products.json")
        load.assert_not_called()
        run.assert_not_called()
        mkdir.assert_not_called()

    def test_shared_headless_config_keeps_the_original_preparation_and_seed(self):
        original = (_support.ROOT / "tests/app.toml").read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            prepared = Path(directory) / "prepared-test-app"
            prepared.touch()
            with _support.use_products({"bin:test_app": prepared}):
                headless = _shared.SharedGame(headless=True).configuration()
                native = _shared.SharedGame().configuration()
            launch = headless["launch"]
            self.assertEqual(launch["no_default_features"], True)
            self.assertEqual(launch["headless"], {"width": 1440, "height": 900, "scale_factor": 1.0})
            self.assertEqual(launch["executable"], str(prepared.resolve()))
            self.assertEqual(launch["features"], ["automation"])
            self.assertEqual(headless["startup"], {"kind": "manual"})
            self.assertEqual(launch["arguments"], native["launch"]["arguments"])
            self.assertEqual(headless["tick"], native["tick"])
            self.assertNotIn("headless", native["launch"])
            self.assertNotIn("no_default_features", native["launch"])
            path = Path(directory) / "engine.toml"
            _support.write_config(path, headless)
            self.assertEqual(tomllib.loads(path.read_text()), headless)
        self.assertEqual((_support.ROOT / "tests/app.toml").read_bytes(), original)

    def test_preparation_records_the_cargo_app_variant_not_a_mode_label_alone(self):
        inventory = acceptance.inventory()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "tests").mkdir()
            for name in ("app.toml", "settings.json"):
                (root / "tests" / name).write_bytes((_support.ROOT / "tests" / name).read_bytes())
            cli, app = root / "woodpecker", root / "test_app"
            cli.write_text("fake CPU-only CLI product")
            app.write_text("fake CPU-only app product")
            def compiled(command, path, **kwargs):
                name, executable = ("woodpecker", cli) if "woodpecker_cli" in command else ("test_app", app)
                Path(path).write_text(json.dumps({"reason": "compiler-artifact", "executable": str(executable),
                    "target": {"name": name, "kind": ["bin"]}, "profile": {"test": False},
                    "features": ["automation"] if name == "test_app" else []}) + "\n")
                return 0
            with patch.object(acceptance, "ROOT", root), \
                    patch.object(acceptance, "inventory", return_value=inventory), \
                    patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(acceptance.shutil, "which", side_effect=lambda name: name), \
                    patch.object(acceptance.subprocess, "run", return_value=Mock(stdout="version")), \
                    patch.object(acceptance.subprocess, "Popen") as popen, \
                    patch.object(acceptance, "logged", side_effect=compiled), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(acceptance.main(["--headless", "--build-only"]), 2)
            report = json.loads((root / "summary.json").read_text())
            self.assertEqual(report["errors"], [])
            manifest = json.loads(Path(report["products_manifest"]).read_text())
            self.assertEqual(manifest["app_build"], {"mode": "headless", "features": ["automation"]})
            self.assertEqual(set(manifest["products"]), {"bin:test_app", "bin:woodpecker"})
            self.assertTrue(Path(manifest["products"]["bin:test_app"]["path"]).is_relative_to(root.resolve() / "target/test-products"))
            popen.assert_not_called()

    def test_headless_still_requires_both_graphics_guards_before_preparation(self):
        for rendered, approved in ((False, "0"), (True, "0"), (False, "1")):
            with self.subTest(rendered=rendered, approved=approved), tempfile.TemporaryDirectory() as directory, \
                    patch.dict(acceptance.os.environ, {"WOODPECKER_TEST_RENDERED": approved}), \
                    patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(acceptance, "logged") as logged, \
                    patch.object(acceptance.subprocess, "Popen") as popen, \
                    contextlib.redirect_stdout(io.StringIO()):
                flags = ["--headless", *(["--rendered"] if rendered else [])]
                self.assertEqual(acceptance.main(flags), 2)
                report = json.loads((Path(directory) / "summary.json").read_text())
                self.assertEqual(report["errors"], [])
                self.assertTrue(any("Fresh scoped graphics approval" in reason for reason in report["preflight"]))
                logged.assert_not_called()
                popen.assert_not_called()

    def test_prepared_headless_execution_binds_one_shared_fixture_and_never_certifies_the_suite(self):
        visited, games = [], []
        rows, objects = {}, {}
        class Probe(unittest.TestCase):
            def runTest(case):
                visited.append(case.owner_module)
                games.append(case.shared_game)
                self.assertTrue(case.shared_game.headless)
                self.assertEqual(_support.app_binary(), prepared)
        for module in sorted(_shared.MODULES):
            case = Probe()
            case.owner_module = module
            rows[module] = dict(kind="python", module=module, graphics=True, external=False,
                                status="Nicht ausgeführt", artifacts=[])
            objects[module] = case
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prepared = root / "prepared-headless-app"
            manifest = root / "products.json"
            manifest.write_text(json.dumps({"app_build": {"mode": "headless", "features": ["automation"]}}))
            with patch.dict(acceptance.os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                    patch.object(acceptance, "inventory", return_value=(rows, objects, [], [])), \
                    patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(acceptance.shutil, "which", side_effect=lambda name: name), \
                    patch.object(acceptance.subprocess, "run", return_value=Mock(stdout="version")), \
                    patch.object(acceptance.subprocess, "Popen") as popen, \
                    patch.object(acceptance, "logged") as logged, \
                    patch.object(acceptance._artifacts, "load", return_value={"bin:test_app": prepared, "bin:woodpecker": root / "cli"}), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(acceptance.main(["--headless", "--rendered", "--artifacts", str(manifest)]), 2)
            report = json.loads((root / "summary.json").read_text())
            self.assertEqual(report["selection_status"], "Bestanden")
            self.assertEqual(report["status"], "Unvollständig")
            self.assertEqual(report["errors"], [])
            self.assertEqual(visited, sorted(_shared.MODULES))
            self.assertEqual(len({id(game) for game in games}), 1)
            logged.assert_not_called()
            popen.assert_not_called()

    def test_prepared_variant_mismatches_fail_without_rebuild_or_app_start(self):
        examples = [(True, None), (True, {"mode": "native", "features": ["automation", "default", "native"]}),
                    (True, {"mode": "headless", "features": ["automation", "native"]}),
                    (True, {"mode": "headless", "features": []}),
                    (False, {"mode": "headless", "features": ["automation"]})]
        for headless, proof in examples:
            with self.subTest(headless=headless, proof=proof), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                manifest = root / "products.json"
                manifest.write_text(json.dumps({"app_build": proof}))
                with patch.dict(acceptance.os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                        patch.object(acceptance.tempfile, "mkdtemp", return_value=directory), \
                        patch.object(acceptance.shutil, "which", side_effect=lambda name: name), \
                        patch.object(acceptance.subprocess, "run", return_value=Mock(stdout="version")), \
                        patch.object(acceptance.subprocess, "Popen") as popen, \
                        patch.object(acceptance, "logged") as logged, \
                        patch.object(acceptance._artifacts, "load", return_value={"bin:test_app": root / "app", "bin:woodpecker": root / "cli"}), \
                        contextlib.redirect_stdout(io.StringIO()):
                    flags = ["--headless"] if headless else ["--category", "keyboard"]
                    self.assertEqual(acceptance.main([*flags, "--rendered", "--artifacts", str(manifest)]), 1)
                report = json.loads((root / "summary.json").read_text())
                self.assertIn("does not prove the requested", report["errors"][0])
                self.assertFalse(any(row["status"] == "Bestanden" for row in report["tests"].values()))
                logged.assert_not_called()
                popen.assert_not_called()

    def test_native_default_does_not_silently_select_headless_only_cases(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--list"]), 0)
        plan = json.loads(output.getvalue())
        self.assertFalse(any(row.get("module", "").startswith("tests.headless.")
                             for row in plan["selected"].values()))
        self.assertEqual([row["name"] for row in plan["builds"]][-2:], ["app-abort", "rust-tests"])
