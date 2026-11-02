"""Offline mutation-tool guards; never product inventory or Cargo evidence."""
import contextlib
import io
import json
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

from tests import acceptance, mutations as m


def process(code=0, **extra):
    return dict(returncode=code, forced=False, timed_out=False, cleanup_errors=[], **extra)


def passing(name):
    return f"running 1 test\ntest {name} ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s\n"


def failure(candidate, name, line, signature=None):
    return (f"running 1 test\ntest {name} ... \nthread '{name}' (12345) panicked at {candidate.test_source}:{line}:5:\n"
            + "\n".join(signature or candidate.signature)
            + f"\nFAILED\nfailures:\n    {name}\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s\n")


class Classification(unittest.TestCase):
    def setUp(self):
        self.candidate = m.CANDIDATES[0]
        self.name = self.candidate.function
        self.line = m.assertion_line(m.ROOT, self.candidate)
        self.output = failure(self.candidate, self.name, self.line)

    def classify(self, *, build=None, baseline=True, run=None, output=None, candidate=None):
        return m.classify(build or process(), baseline, run or process(101),
                          self.output if output is None else output, candidate or self.candidate, self.name, self.line)

    def test_only_exact_semantic_assertion_is_killed(self):
        self.assertEqual(self.classify(), "killed")
        for candidate in m.CANDIDATES[:-1]:
            line = m.assertion_line(m.ROOT, candidate)
            self.assertEqual(m.classify(process(), True, process(101),
                failure(candidate, candidate.function, line), candidate, candidate.function, line), "killed")

    def test_legacy_panic_header_without_numeric_thread_id_is_also_supported(self):
        self.assertEqual(self.classify(output=self.output.replace(" (12345)", "")), "killed")

    def test_compile_error_is_not_detection_even_with_plausible_test_output(self):
        self.assertEqual(self.classify(build=process(101)), "invalid")
        self.assertEqual(self.classify(output="error[E0308]: mismatched types\n"), "invalid")

    def test_failed_or_missing_baseline_invalidates_kill(self):
        self.assertEqual(self.classify(baseline=False), "invalid")

    def test_wrong_assertion_function_or_location_is_not_detection(self):
        variants = [failure(self.candidate, self.name, self.line, ["assertion failed: unrelated"]),
                    self.output.replace(f":{self.line}:5:", f":{self.line + 1}:5:"),
                    self.output.replace(self.candidate.test_source, "src/unrelated.rs"),
                    self.output.replace(f"thread '{self.name}'", "thread 'other'"),
                    self.output.replace(f"test {self.name} ...", "test wrong ...")]
        for output in variants:
            with self.subTest(output=output):
                self.assertEqual(self.classify(output=output), "invalid")

    def test_zero_ignored_multiple_tests_and_duplicate_summary_are_not_detection(self):
        for output in ["running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;\n",
                       self.output.replace("0 ignored", "1 ignored"),
                       self.output.replace("running 1 test", "running 2 tests"),
                       self.output + self.output,
                       self.output.replace("0 passed; 1 failed", "1 passed; 1 failed")]:
            self.assertEqual(self.classify(output=output), "invalid")

    def test_crash_timeout_forced_cleanup_and_cleanup_error_are_not_kills(self):
        for run in [process(-9), {**process(101), "forced": True},
                    {**process(101), "cleanup_errors": ["peer still alive"]}]:
            self.assertEqual(self.classify(run=run), "invalid")
        self.assertEqual(self.classify(run={**process(101), "timed_out": True}), "incomplete")
        self.assertEqual(self.classify(build={**process(101), "timed_out": True}), "incomplete")

    def test_second_panic_or_unrelated_failure_is_not_detection(self):
        self.assertEqual(self.classify(output=self.output + "thread 'cleanup' panicked at src/x.rs:2:3:\nfault\n"), "invalid")
        self.assertEqual(self.classify(output=self.output.replace("1 failed", "2 failed")), "invalid")

    def test_passing_fault_is_survivor_not_automatically_equivalent(self):
        self.assertEqual(self.classify(run=process(), output=passing(self.name)), "survived")
        self.assertEqual(self.classify(run=process(), output=passing(self.name), candidate=m.CANDIDATES[-1]), "known-equivalent")
        self.assertEqual(self.classify(candidate=m.CANDIDATES[-1]), "invalid")


class CampaignLifecycle(unittest.TestCase):
    def run_campaign(self, *, build_fails=False, copy_fails=False, cleanup_fails=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            author, parent = root / "author", root / "parent"
            for name in ("tests/mutations.py", "tests/runner_checks/mutations.py", "tests/mutations.md"):
                path = author / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("tooling fixture")
            products = {}
            for name in ("target/debug/woodpecker", "target/debug/test_app"):
                path = parent / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(b"protected parent product")
                products[name] = m.file_state(path)
            products_path = root / "products.json"
            products_path.write_text(json.dumps(products))
            previous = author / "target/test-runs/previous/evidence.log"
            previous.parent.mkdir(parents=True)
            previous.write_text("keep previous evidence")
            args = type("Args", (), dict(parent_root=str(parent), parent_products=str(products_path),
                evidence_dir=str(parent / "target/test-runs/campaign/mutation-evidence"),
                source_ref="HEAD", clone_parent_cache=False))()
            workspaces = []

            def git(unused, *arguments):
                if arguments[0] == "archive":
                    Path(arguments[arguments.index("-o") + 1]).write_bytes(b"archive fixture")
                return b"fixture-commit" if arguments[0] == "rev-parse" else b""

            def extract(unused, source):
                source.mkdir()
                (source / "lib.rs").write_text("source fixture")

            def build(source, cache, evidence, *unused):
                workspaces.append(cache.parent)
                (cache / "artifact").write_bytes(b"private build artifact")
                (evidence / "baseline-build.log").write_text("build evidence")
                if build_fails:
                    raise RuntimeError("baseline build failed")
                return process(), {}

            copytree = m.shutil.copytree
            def copy(source, destination, *arguments, **keywords):
                if copy_fails:
                    raise OSError("evidence copy failed")
                return copytree(source, destination, *arguments, **keywords)

            with patch.object(m, "ROOT", author), patch.object(m, "CANDIDATES", ()), \
                    patch.object(m, "git", side_effect=git), patch.object(m, "extract", side_effect=extract), \
                    patch.object(m, "tracked_state", return_value={}), patch.object(m, "build", side_effect=build), \
                    patch.object(m.shutil, "copytree", side_effect=copy), \
                    contextlib.ExitStack() as stack, contextlib.redirect_stdout(io.StringIO()) as output:
                if cleanup_fails:
                    stack.enter_context(patch.object(m.shutil, "rmtree", side_effect=OSError("cache removal failed")))
                if copy_fails:
                    with self.assertRaisesRegex(OSError, "evidence copy failed"):
                        m.campaign(args)
                    owned = workspaces[0]
                    self.assertTrue((owned / "cargo-target/artifact").is_file())
                    self.assertTrue((owned / "evidence/baseline-build.log").is_file())
                    self.assertFalse(json.loads((owned / "evidence/summary.json").read_text())["complete"])
                    return
                code = m.campaign(args)
            published = json.loads(output.getvalue())
            report = json.loads(Path(published["report"]).read_text())
            self.assertTrue((Path(published["report"]).parent / "baseline-build.log").is_file())
            self.assertEqual(previous.read_text(), "keep previous evidence")
            self.assertEqual({name: m.file_state(parent / name) for name in products}, products)
            self.assertEqual(workspaces[0].exists(), cleanup_fails)
            self.assertEqual(report["workspace_removed"], not cleanup_fails)
            self.assertEqual(code, 2 if build_fails or cleanup_fails else 0)
            self.assertEqual(report["complete"], not (build_fails or cleanup_fails))
            if cleanup_fails:
                self.assertTrue(any("cache removal failed" in error for error in report["errors"]))

    def test_success_removes_owned_cache_after_publishing_evidence(self):
        self.run_campaign()

    def test_failed_build_still_publishes_evidence_and_removes_cache(self):
        self.run_campaign(build_fails=True)

    def test_failed_evidence_copy_preserves_workspace_and_incomplete_report(self):
        self.run_campaign(copy_fails=True)

    def test_cache_removal_failure_is_reported_as_incomplete(self):
        self.run_campaign(cleanup_fails=True)


class Safety(unittest.TestCase):
    def test_list_and_help_do_not_spawn_or_write(self):
        with patch.object(m.subprocess, "run") as run, patch.object(m.subprocess, "Popen") as popen, \
                patch.object(Path, "mkdir") as mkdir, patch.object(Path, "write_text") as write, \
                contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(m.main(["--list"]), 0)
            self.assertEqual(len(json.loads(output.getvalue())), 5)
            with self.assertRaises(SystemExit) as raised:
                m.main(["--help"])
            self.assertEqual(raised.exception.code, 0)
        for mocked in (run, popen, mkdir, write):
            mocked.assert_not_called()

    def test_standard_build_commands_stay_windowless_locked_and_compile_only(self):
        for rows, expected in [([dict(kind="internal")], ["--lib"]),
                               ([dict(kind="rust", target="handle_encoding")], ["--test", "handle_encoding"])]:
            with patch.object(m, "logged", return_value=process(log="offline.log")) as logged, \
                    patch.object(m, "executables", return_value={}):
                m.build(Path("source"), Path("cache"), Path("evidence"), "offline", rows, {})
            command = logged.call_args.args[0]
            for flag in ["--locked", "--no-run", "--message-format=json", *expected]:
                self.assertIn(flag, command)
            self.assertEqual(command[:2], ["cargo", "test"])
            self.assertNotIn("--bin", command)

    def test_missing_run_arguments_stop_before_setup(self):
        with patch.object(m, "campaign") as campaign, contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit):
                m.main([])
        campaign.assert_not_called()

    def test_declared_product_sites_and_oracles_are_unique(self):
        for candidate in m.CANDIDATES:
            self.assertTrue(candidate.source.startswith("src/"))
            self.assertNotIn("/tests/", candidate.source)
            source = (m.ROOT / candidate.source).read_text()
            mutated = m.replace_once(source, candidate.old, candidate.new)
            self.assertNotEqual(mutated, source)
            self.assertEqual(mutated.replace(candidate.new, candidate.old, 1), source)
            self.assertGreater(m.assertion_line(m.ROOT, candidate), 0)
        for text in ("", "same same"):
            with self.assertRaises(ValueError):
                m.replace_once(text, "same", "changed")

    def test_archive_rejects_symlinks_hardlinks_and_traversal_before_extraction(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, kind in [("../escape", tarfile.REGTYPE), ("/absolute", tarfile.REGTYPE),
                               ("link", tarfile.SYMTYPE), ("hard", tarfile.LNKTYPE)]:
                archive = root / "source.tar"
                with tarfile.open(archive, "w") as bundle:
                    member = tarfile.TarInfo(name)
                    member.type = kind
                    member.linkname = "outside"
                    bundle.addfile(member)
                with self.assertRaises(ValueError):
                    m.extract(archive, root / "copy")
                self.assertFalse((root / "copy").exists())

    def test_private_cache_rejects_parent_hardlinks_and_symlinks(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            parent, private = root / "parent", root / "private"
            parent.mkdir()
            private.mkdir()
            (parent / "binary").write_bytes(b"not executed")
            os.link(parent / "binary", private / "hardlink")
            with self.assertRaisesRegex(ValueError, "Hardlink"):
                m.verify_private_cache(parent, private)
            (private / "hardlink").unlink()
            (private / "link").symlink_to(parent / "binary")
            with self.assertRaisesRegex(ValueError, "Symlink"):
                m.verify_private_cache(parent, private)
            (private / "link").unlink()
            (private / "copy").write_bytes((parent / "binary").read_bytes())
            self.assertEqual(m.verify_private_cache(parent, private)["files_checked"], 1)
            with self.assertRaises(ValueError):
                m.verify_private_cache(parent, parent)

    def test_evidence_parent_writes_are_scoped_new_and_not_symlinked(self):
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory).resolve()
            valid = parent / "target/test-runs/campaign/mutation-evidence/run"
            self.assertEqual(m.evidence_destination(parent, valid), valid)
            for path in (parent / "src/run", parent / "target/test-runs/../mutation-evidence/run",
                         parent / "target/test-runs/campaign/other/run"):
                with self.assertRaises(ValueError):
                    m.evidence_destination(parent, path)
            valid.mkdir(parents=True)
            with self.assertRaises(ValueError):
                m.evidence_destination(parent, valid)
            link = parent / "target/test-runs/linked"
            link.symlink_to(valid.parent, target_is_directory=True)
            with self.assertRaises(ValueError):
                m.evidence_destination(parent, link / "mutation-evidence/other")

    def test_inventory_stays_75_python_10_public_9_internal_plus_control(self):
        rows, _, targets, errors = acceptance.inventory()
        self.assertEqual(errors, [])
        self.assertEqual(len(rows), 95)
        self.assertEqual(len(targets), 10)
        self.assertEqual({kind: sum(row["kind"] == kind for row in rows.values())
                          for kind in ("python", "rust", "internal", "control")},
                         dict(python=75, rust=10, internal=9, control=1))
        self.assertFalse(any(row.get("module", "").startswith("tests.runner_checks.") for row in rows.values()))
