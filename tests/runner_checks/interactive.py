"""Interactive mode is planning-only until explicit, compatible, prebuilt graphics use."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from tests import interactive


class InteractiveTest(unittest.TestCase):
    def test_default_group_planning_contains_only_the_ten_compatible_cases(self):
        data = interactive.plan([arg for category in ("keyboard", "pointer", "text")
                                 for arg in ("--category", category)])
        self.assertEqual(len(data["selected"]), 10)
        self.assertTrue(all(row["module"] in interactive._shared.MODULES for row in data["selected"].values()))

    def test_isolated_cases_and_build_flags_are_rejected_without_starting_a_process(self):
        with patch.object(interactive.subprocess, "Popen") as process:
            for arguments in (["--category", "shutdown"], ["--category", "keyboard", "--build-only"],
                              ["--test", "--list"], ["--category", "keyboard", "--test", "3.12"]):
                with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                    interactive.plan(arguments)
            process.assert_not_called()

    def test_existing_state_is_not_automatically_replaced_after_connection_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory) / "state.json"
            state.write_text('{"instance":"old", "port":1234, "manifest":"/old/products.json"}')
            with patch.object(interactive, "STATE", state), \
                    patch.dict(interactive.os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                    patch.object(interactive._artifacts, "load", return_value={}), \
                    patch.object(interactive, "request", side_effect=ConnectionRefusedError("old host died")), \
                    patch.object(interactive.subprocess, "Popen") as process, \
                    contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(interactive.main(["--category", "keyboard"]), 1)
                process.assert_not_called()
                self.assertTrue(state.exists())
