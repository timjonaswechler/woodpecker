"""Runner selection through the public command line; no Cargo or application starts."""
import contextlib
import io
import json
import unittest

from tests import acceptance


class SelectionTest(unittest.TestCase):
    def test_category_selects_only_its_existing_cases(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--list", "--category", "keyboard"]), 0)
        plan = json.loads(output.getvalue())
        self.assertEqual(len(plan["selected"]), 3)
        self.assertEqual({row["module"] for row in plan["selected"].values()}, {
            "tests.keyboard.movement", "tests.keyboard.single_action", "tests.keyboard.invalid_transitions"})
        self.assertFalse(plan["feature_matrix"])
        self.assertEqual([row["name"] for row in plan["builds"]], ["cli", "app-automation"])

    def test_single_module_does_not_also_select_its_negative_control(self):
        with contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(acceptance.main(["--list", "--test", "tests.inspect.resource_value"]), 0)
        plan = json.loads(output.getvalue())
        self.assertEqual(len(plan["selected"]), 1)
        self.assertEqual(next(iter(plan["selected"].values()))["kind"], "python")
