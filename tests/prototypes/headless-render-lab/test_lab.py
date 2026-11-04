"""CPU-only packaging/guard tests; never build or execute a GPU application."""
import contextlib
import io
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

SOURCE = Path(__file__).resolve().parent
sys.path.insert(0, str(SOURCE))
import audit as lab_audit
import run as lab_run
from png_oracle import decode


class PackagingTests(unittest.TestCase):
    def test_recorded_evidence_and_execution_sources(self):
        result = lab_audit.verify_recorded()
        self.assertEqual(result["recorded_cases"], 15)
        self.assertFalse(result["new_gpu_execution"])

    def test_partial_png_is_not_a_valid_image(self):
        with self.assertRaises(AssertionError):
            decode(SOURCE / "evidence/partial-staging-prefix.bin")

    def test_gpu_command_requires_explicit_flag_before_writes(self):
        base = lab_run.ROOT / "target/headless-render-lab/test-tmp"
        base.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=base) as temp:
            destination = Path(temp) / "must-not-exist"
            with patch.object(lab_run, "ARTIFACTS", destination), contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit) as error:
                    lab_run.main(["run", "--name", "not-authorized"])
            self.assertEqual(error.exception.code, 2)
            self.assertFalse(destination.exists())

    def test_plan_has_no_build_or_gpu_side_effects(self):
        output = io.StringIO()
        with patch.object(lab_run, "bounded", side_effect=AssertionError("must not spawn")):
            with contextlib.redirect_stdout(output):
                self.assertEqual(lab_run.main(["plan"]), 0)
        self.assertEqual(len(output.getvalue().splitlines()), 15)

    def test_run_name_cannot_escape_artifact_root(self):
        with contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                lab_run.parse_args(["run", "--allow-gpu", "--name", "../escape"])

    def test_changed_source_hash_fails_evidence_verification(self):
        with patch.object(lab_audit, "sha", return_value="not-the-recorded-hash"):
            with self.assertRaises(AssertionError):
                lab_audit.verify_recorded()


if __name__ == "__main__":
    unittest.main()
