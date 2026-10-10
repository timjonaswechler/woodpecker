"""Checks of selection and result interpretation that prevent falsely green suites."""
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

from tests.catalog import scenarios, select
from tests.session.application import Application, ROOT, Unavailable
from tests.interfaces.json import check_result
from tests.interfaces.repl import line_for


class SuiteTests(unittest.TestCase):
    def test_every_public_command_has_a_behavioral_contract(self):
        source = (ROOT / "src/command/mod.rs").read_text().split("pub enum Command {", 1)[1].split("\n}", 1)[0]
        commands = set(re.findall(r'#\[serde\(rename = "([^"]+)"\)\]', source))
        covered = {command for item in scenarios() for command in item.commands}
        self.assertTrue(commands)
        self.assertEqual(commands, covered, "Ein neues Command benötigt einen dauerhaften Verhaltensvertrag")

    def test_warp_uses_same_contract_in_four_combinations(self):
        selected = select(["warp"])
        for name in {s.name for s, _, _ in selected}:
            self.assertEqual({(mode, interface) for s, mode, interface in selected if s.name == name},
                             {("rendered", "json"), ("rendered", "repl"),
                              ("headless", "json"), ("headless", "repl")})
        targeted = select(["warp"], ["headless"], ["repl"])
        self.assertEqual(len(targeted), 3)
        self.assertTrue(all(mode == "headless" and interface == "repl" for _, mode, interface in targeted))

    def test_unknown_feature_or_scenario_is_rejected(self):
        with self.assertRaises(ValueError):
            select(["typo"])
        with self.assertRaises(ValueError):
            select(names=["warp.typo"])

    def test_fault_contracts_are_listed_once_per_interface_without_an_app(self):
        name = "reports.panic_abort_reports_once_before_end"
        selected = select(names=[name])
        self.assertEqual({(mode, interface) for _, mode, interface in selected},
                         {(None, "json"), (None, "repl")})
        result = subprocess.run([sys.executable, "-m", "tests.run", "--layer", "system",
                                 "--faults-only", "--scenario", name, "--list",
                                 "--app", "/nonexistent/application.json"], cwd=ROOT,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(len(result.stdout.splitlines()), 2)
        self.assertIn("mode-independent/json", result.stdout)

    def test_incompatible_interface_cannot_pass_using_only_rust_tests(self):
        result = subprocess.run([sys.executable, "-m", "tests.run", "--feature", "repl",
                                 "--interface", "json", "--list"], cwd=ROOT,
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertIn("keine System-Szenarien", result.stderr)

    def test_failed_or_wrong_rejection_is_not_success(self):
        with self.assertRaises(AssertionError):
            check_result({"kind": "failed", "error": {"code": "io"}})
        with self.assertRaises(AssertionError):
            check_result({"kind": "rejected", "error": {"code": "other"}}, "invalid_key")

    def test_unavailable_mode_or_capability_is_not_supported(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "app.json"
            (path.parent / "app.toml").write_text("version = 1\n")
            path.write_text(json.dumps({"version": 1, "state_type": "app::TestState", "modes": {
                "headless": {"config": "app.toml", "capabilities": ["state"]}}}))
            app = Application(path)
            with self.assertRaises(Unavailable):
                app.require("rendered", ["state"])
            with self.assertRaises(Unavailable):
                app.require("headless", ["state", "keyboard"])

    def test_missing_app_produces_incomplete_report_and_nonzero_exit(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run = subprocess.run([sys.executable, "-m", "tests.run", "--layer", "system",
                                  "--feature", "warp", "--mode", "headless", "--interface", "json",
                                  "--no-build", "--app", str(root / "missing.json"),
                                  "--output", str(root / "run")], cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(run.returncode, 2, run.stdout + run.stderr)
            report = json.loads((root / "run/results.json").read_text())
            self.assertEqual(report["status"], "incomplete")
            self.assertEqual(len(report["results"]), 3)
            self.assertTrue(all(r["status"] == "unavailable" for r in report["results"]))

    def test_repl_preserves_unicode_and_paths(self):
        self.assertEqual(line_for("recording.start", {"path": "recordings/my run.jsonl"}),
                         "recording start recordings/my run.jsonl")
        line = line_for("input.text.input", {"text": "Grüße 日本語"})
        self.assertEqual(json.loads(line.removeprefix("command "))["arguments"]["text"], "Grüße 日本語")


if __name__ == "__main__":
    unittest.main()
