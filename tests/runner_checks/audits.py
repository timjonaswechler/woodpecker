"""Feature proof can be reused only for identical inputs and unchanged audit evidence."""
from pathlib import Path
import tempfile
import unittest

from tests import _audits


class AuditsTest(unittest.TestCase):
    def test_changed_inputs_or_log_never_reuse_audit_proof(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = root / "audit.log"
            log.write_text('audit passed\n')
            inputs = {"source": "first"}
            command = ["cargo", "check", "--locked"]
            _audits.publish(root, inputs, {"library-default": dict(command=command, log=str(log))})
            self.assertEqual(_audits.load(root, inputs)["library-default"]["command"], command)
            self.assertEqual(_audits.load(root, {"source": "second"}), {})
            log.write_text('changed evidence\n')
            self.assertEqual(_audits.load(root, inputs), {})

    def test_temporary_compilation_tree_is_removed_but_evidence_is_not(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            log = root / "evidence.log"
            log.write_text("retained")
            with _audits.workspace(root) as path:
                cache = Path(path)
                (cache / "output.rlib").write_text("disposable")
            self.assertFalse(cache.exists())
            self.assertEqual(log.read_text(), "retained")
