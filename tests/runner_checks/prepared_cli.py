"""Direct CLI cases resolve prepared programs after discovery, not imported path snapshots."""
import ast
import contextlib
import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

from tests import _support
from tests.cli import command_errors


class PreparedCliTest(unittest.TestCase):
    def test_prepared_direct_case_ignores_missing_or_replaced_disposable_cli(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prepared = root / "prepared-cli"
            prepared.write_text(f"#!{sys.executable}\n" + '''import json, sys
try:
    source = sys.argv[-1]
    source.encode("utf-8")
    value = json.loads(source)
    json.dumps(value, ensure_ascii=False).encode("utf-8")
except (ValueError, UnicodeError):
    kind = "invalid_json"
else:
    kind = "invalid_command"
print(json.dumps({"error": kind + ": fixture rejection"}), file=sys.stderr)
sys.exit(1)
''')
            prepared.chmod(0o700)
            fallback = root / "unchecked-cli"
            sentinel = root / "unchecked-invoked"
            for replaced in (False, True):
                if replaced:
                    fallback.write_text(f"#!{sys.executable}\nfrom pathlib import Path\nPath({str(sentinel)!r}).touch()\nraise SystemExit(42)\n")
                    fallback.chmod(0o700)
                with self.subTest(replaced=replaced), patch.object(_support, "CLI", fallback), \
                        patch.object(command_errors, "ROOT", root), \
                        _support.use_products({"bin:woodpecker": prepared}), \
                        contextlib.redirect_stdout(io.StringIO()):
                    result = unittest.TestResult()
                    case = command_errors.CommandErrorsTest("test_invalid_json_utf8_and_command_shapes_do_not_send_even_a_valid_prefix")
                    case.run(result)
                    self.assertEqual(result.testsRun, 1)
                    self.assertEqual(result.errors + result.failures, [])
                    self.assertFalse(sentinel.exists())

    def test_no_case_imports_a_snapshot_of_the_cli_path(self):
        for path in (_support.ROOT / "tests").rglob("*.py"):
            if "runner_checks" in path.parts:
                continue
            tree = ast.parse(path.read_text())
            for node in ast.walk(tree):
                if isinstance(node, ast.ImportFrom) and node.module == "tests._support":
                    self.assertNotIn("CLI", [alias.name for alias in node.names], str(path))
