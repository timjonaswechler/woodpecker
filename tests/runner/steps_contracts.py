"""The runner must preserve failures without inventing dependent failures."""
import json
from pathlib import Path
import tempfile
import unittest

from tests.session.steps import Steps


class StepTests(unittest.TestCase):
    def test_failed_step_blocks_dependents_but_independent_checks_continue(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "steps.json"
            steps = Steps(path)

            def fail():
                raise AssertionError("Spielzustand stimmt nicht")

            def must_not_run():
                self.fail("Abhängiger Schritt wurde trotz Fehler ausgeführt")

            steps.run("start", lambda: {"session_id": "a", "start_ticks": 17})
            steps.run("state", fail, requires=("start",))
            steps.run("dependent", must_not_run, requires=("state",))
            steps.run("image", lambda: {"path": "frame.png"}, requires=("start",))
            with self.assertRaisesRegex(AssertionError, "state"):
                steps.finish()
            results = json.loads(path.read_text())
            self.assertEqual([r["status"] for r in results],
                             ["passed", "failed", "blocked", "passed"])
            self.assertEqual(results[0]["evidence"]["start_ticks"], 17)
            self.assertEqual(results[2]["blocked_by"], ["state"])
            self.assertEqual(results[3]["evidence"]["path"], "frame.png")

    def test_unknown_dependencies_and_duplicate_names_are_errors(self):
        with tempfile.TemporaryDirectory() as directory:
            steps = Steps(Path(directory) / "steps.json")
            with self.assertRaises(ValueError):
                steps.run("later", lambda: None, requires=("typo",))
            steps.run("ready", lambda: None)
            with self.assertRaises(ValueError):
                steps.run("ready", lambda: None)
            steps.finish()

    def test_invalid_checkpoint_blocks_action_and_a_restored_checkpoint_can_continue(self):
        with tempfile.TemporaryDirectory() as directory:
            steps = Steps(Path(directory) / "steps.json")

            def wrong_state():
                raise AssertionError("Ausgangszustand verändert")

            steps.run("unsafe", lambda: self.fail("Ungültiger Ausgangszustand wurde verwendet"),
                      precondition=wrong_state)
            steps.run("restored", lambda: {"result": "observed"}, precondition=lambda: None)
            self.assertEqual([r["status"] for r in steps.results], ["blocked", "passed"])
            self.assertIn("Ausgangszustand", steps.results[0]["error"])
            with self.assertRaises(AssertionError):
                steps.finish()


if __name__ == "__main__":
    unittest.main()
