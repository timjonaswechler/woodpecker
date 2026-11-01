"""Separately authorized real GitHub provider acceptance; never part of the local default."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
import unittest
import uuid

from tests._support import ROOT, load_config
from tests.providers.live_transport import cleanup, pages, request


class GithubLiveTest(unittest.TestCase):
    def _gh(self, *arguments, timeout=60):
        # Called only after all repository/graphics/cleanup guards in the test body.
        result = subprocess.run([self.gh, "api", *arguments], cwd=ROOT, env=self.environment,
                                capture_output=True, text=True, timeout=timeout)
        # No auth/token commands and no environment dump in artifacts.
        with (self.artifacts / "github-api.jsonl").open("a") as stream:
            stream.write(json.dumps({"arguments": arguments, "returncode": result.returncode,
                                     "stdout": result.stdout, "stderr": result.stderr}) + "\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout

    def _owned_issues(self, *, timeout=60):
        issues = pages(self._gh(f"repos/{self.repo}/issues?state=all&per_page=100", "--paginate", timeout=timeout))
        return [v for v in issues if "pull_request" not in v and
                v.get("title") == self.plan["title"] and self.token in (v.get("body") or "") and
                f"<!-- bug_hunter-signature: {self.plan['signature']} -->" in (v.get("body") or "")]

    def _remember_reference(self, outcome):
        reference = outcome.get("reference", {})
        if reference.get("kind") != "issue":
            return  # A fallback is not confirmed remote publication or cleanup evidence.
        value = reference["reference"]
        number = value["identifier"]
        self.assertRegex(number, r"^[1-9][0-9]*$")
        self.assertEqual(value["url"], f"https://github.com/{self.repo}/issues/{number}")
        self.references[number] = value["url"]
        self.remote_pending = False

    def _wait_until_listed(self, number):
        # GitHub's list can lag a successful POST/direct GET. Establish the
        # search precondition with successful read-only observations, not sleeps
        # alone, retries of failed requests, or another Submit.
        started = time.monotonic()
        deadline = started + 90
        observations = 0
        while True:
            remaining = deadline - time.monotonic()
            self.assertGreater(remaining, 0, "GitHub list visibility unknown after 90s; no Submit retry")
            issues = self._owned_issues(timeout=min(60, remaining))
            observations += 1
            matching = [issue for issue in issues if str(issue["number"]) == number]
            if matching:
                self.assertEqual(len(issues), 1, "Unexpected duplicate before the second Submit")
                self.assertEqual(matching[0]["body"], self.plan["body"])
                self.assertEqual(matching[0]["html_url"], self.references[number])
                (self.artifacts / "publication-visibility.json").write_text(json.dumps({
                    "identifier": number, "observations": observations,
                    "seconds": time.monotonic() - started, "budget_seconds": 90,
                    "publication_retries": 0}) + "\n")
                return
            time.sleep(min(3, max(0, deadline - time.monotonic())))

    def _close_owned(self):
        if self.plan is None or not self.submit_attempted:
            return
        # Confirmed references are authoritative even while the list is stale.
        # Only uncertain outcomes need discovery, never another publication.
        references = dict(self.references)
        if self.remote_pending or not references:
            for issue in self._owned_issues():
                references.setdefault(str(issue["number"]), issue["html_url"])
        self.assertLessEqual(len(references), 2, "Unexpected ownership set; refuse broad cleanup")
        closed = []
        for number, url in references.items():
            self.assertRegex(number, r"^[1-9][0-9]*$")
            self.assertEqual(url, f"https://github.com/{self.repo}/issues/{number}")
            current = json.loads(self._gh(f"repos/{self.repo}/issues/{number}"))
            self.assertEqual(current["title"], self.plan["title"])
            self.assertEqual(current["body"], self.plan["body"])
            self.assertIn(self.token, current["body"])
            self.assertIn(f"<!-- bug_hunter-signature: {self.plan['signature']} -->", current["body"])
            self.assertEqual(current["html_url"], url)
            if current["state"] != "closed":
                self._gh(f"repos/{self.repo}/issues/{number}", "--method", "PATCH", "-f", "state=closed")
            verify = json.loads(self._gh(f"repos/{self.repo}/issues/{number}"))
            self.assertEqual(verify["state"], "closed")
            self.assertEqual(verify["body"], self.plan["body"])
            closed.append(int(number))
        unresolved = self.remote_pending or not references
        (self.artifacts / "remote-cleanup.json").write_text(json.dumps({
            "repository": self.repo, "run_token": self.token, "closed_owned_issues": closed,
            "deleted_issues": 0, "unresolved": unresolved}) + "\n")
        self.assertFalse(unresolved, "Remote publication outcome unknown; cleanup unresolved")

    def test_real_provider_creates_one_issue_and_finds_it_on_repeat(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Fresh approval: one real Manual window, zero ticks/inputs; GitHub test repository separately approved")
        repo = os.environ.get("WOODPECKER_TEST_GITHUB_REPO", "")
        self.assertRegex(repo, r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$", "Explicit approved owner/repo required")
        self.assertEqual(os.environ.get("WOODPECKER_TEST_GITHUB_CLEANUP"), "close",
                         "Approve two Submit attempts (expected one issue, at most two on dedup defect) and closing all uniquely owned run issues")
        self.assertNotIn("WOODPECKER_TEST_GH_ROOT", os.environ, "Fake gh is not live evidence")
        self.gh = shutil.which("gh")
        self.assertIsNotNone(self.gh, "Real authenticated gh required; never enter a token in chat")
        driver = ROOT / "target/debug/github-live-driver"
        self.assertTrue(driver.is_file(), "Use acceptance.py build phase first")
        self.artifacts = Path(tempfile.mkdtemp(prefix="github-live-", dir=ROOT / "target/test-runs"))
        self.artifact_dirs = [str(self.artifacts)]
        self.environment = {**os.environ, "GH_REPO": repo, "GH_HOST": "github.com", "GH_PROMPT_DISABLED": "1",
                            "CARGO_TARGET_DIR": str(ROOT / "target")}
        self.repo = repo
        self.token = "woodpecker-acceptance-" + str(uuid.uuid4())
        self.plan = None
        self.references = {}
        self.submit_attempted = False
        self.remote_pending = False
        # Read-only repository verification precedes every publication.
        metadata = json.loads(self._gh(f"repos/{repo}"))
        self.assertEqual(metadata["full_name"].lower(), repo.lower())
        self.repo = metadata["full_name"]
        self.environment["GH_REPO"] = self.repo
        self.environment["WOODPECKER_TEST_GITHUB_REPO"] = self.repo
        self.assertTrue(metadata["has_issues"])
        self.assertFalse(metadata["archived"])
        self.assertIn("woodpecker-acceptance-test-repository", metadata.get("description") or "",
                      "Dedicated test repository must explicitly carry this marker in its description")
        self.assertTrue(metadata.get("permissions", {}).get("push"), "Issue cleanup permission required")
        self.addCleanup(self._close_owned)
        stderr = self.enterContext((self.artifacts / "driver.stderr.log").open("w"))
        process = subprocess.Popen([str(driver)], cwd=ROOT, env=self.environment, stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=stderr, text=True, start_new_session=True)
        self.addCleanup(cleanup, process, self.artifacts)
        configuration = load_config()
        configuration["launch"]["manifest_path"] = str(ROOT / configuration["launch"]["manifest_path"])
        configuration["report"]["provider"] = {"kind": "github"}
        # Every SDK operation is explicit here; the helper is only transport.
        request(process, {"action": "start", "config": configuration,
                          "artifact_dir": str(self.artifacts / "session")}, timeout=120)
        request(process, {"action": "shutdown"})
        self.plan = request(process, {"action": "report", "message": self.token})
        (self.artifacts / "publication-plan.json").write_text(json.dumps(self.plan, indent=2) + "\n")
        self.assertEqual(self._owned_issues(), [], "Unique run token must not preexist")
        self.submit_attempted = True
        self.remote_pending = True
        first = request(process, {"action": "submit"}, timeout=180)
        self._remember_reference(first)
        (self.artifacts / "first-submit.json").write_text(json.dumps(first, indent=2) + "\n")
        self.assertEqual(first["kind"], "created", first)  # No fallback accepted.
        self._wait_until_listed(first["reference"]["reference"]["identifier"])
        self.remote_pending = True
        second = request(process, {"action": "submit"}, timeout=180)
        self._remember_reference(second)
        (self.artifacts / "second-submit.json").write_text(json.dumps(second, indent=2) + "\n")
        self.assertEqual(second, {"kind": "existing", "reference": first["reference"]})
        issues = self._owned_issues()
        self.assertEqual(len(issues), 1)
        issue = json.loads(self._gh(f"repos/{self.repo}/issues/{issues[0]['number']}"))
        self.assertEqual(issue["body"], self.plan["body"])
        self.assertEqual(issue["title"], self.plan["title"])
        self.assertEqual(first["reference"], {"kind": "issue", "reference":{"identifier": str(issue["number"]),
                                            "url": issue["html_url"]}})
        self.assertFalse((self.artifacts / "session/reports").exists(), "Remote success must not fall back locally")
        request(process, {"action": "quit"})
        self.assertEqual(process.wait(timeout=10), 0)
        # Registered cleanup verifies the exact run body/reference before closing, never deleting.
