"""Offline live-provider harness regressions; no app, gh or network calls."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch

from tests.providers import github_live


class GithubLiveHarnessTest(unittest.TestCase):
    def _case(self, root):
        case = github_live.GithubLiveTest("test_real_provider_creates_one_issue_and_finds_it_on_repeat")
        case.repo = "owner/fixture"
        case.token = "owned-run"
        case.plan = {"title": "owned-run", "signature": "v1:sha256:" + "a" * 64,
                     "body": "owned-run\n<!-- bug_hunter-signature: v1:sha256:" + "a" * 64 + " -->\n"}
        case.artifacts = root
        case.references = {"1": "https://github.com/owner/fixture/issues/1",
                           "2": "https://github.com/owner/fixture/issues/2"}
        case.submit_attempted = True
        case.remote_pending = False
        return case

    def test_cleanup_uses_confirmed_references_when_issue_list_is_stale(self):
        with tempfile.TemporaryDirectory() as directory:
            case = self._case(Path(directory))
            closed = []

            def api(endpoint, *arguments, **kwargs):
                if "?" in endpoint:
                    return "[]"
                number = int(endpoint.rsplit("/", 1)[1])
                if "PATCH" in arguments:
                    closed.append(number)
                return json.dumps({"number": number, "title": case.plan["title"],
                    "body": case.plan["body"], "html_url": f"https://github.com/{case.repo}/issues/{number}",
                    "state": "closed" if number in closed else "open"})

            case._gh = Mock(side_effect=api)
            case._close_owned()
            self.assertEqual(closed, [1, 2])
            result = json.loads((case.artifacts / "remote-cleanup.json").read_text())
            self.assertEqual(result["closed_owned_issues"], [1, 2])
            self.assertEqual(result["deleted_issues"], 0)

    def test_cleanup_refuses_nonmatching_body_before_any_write(self):
        with tempfile.TemporaryDirectory() as directory:
            case = self._case(Path(directory))
            case._gh = Mock(side_effect=lambda endpoint, *args, **kwargs: "[]" if "?" in endpoint else
                            json.dumps({"title": case.plan["title"], "body": "foreign body",
                                        "html_url": f"https://github.com/{case.repo}/issues/1", "state": "open"}))
            with self.assertRaises(AssertionError):
                case._close_owned()
            self.assertFalse(any("PATCH" in call.args for call in case._gh.call_args_list))

    def test_unknown_submit_cannot_be_recorded_as_successful_empty_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            case = self._case(Path(directory))
            case.references = {}
            case.remote_pending = True
            case._gh = Mock(return_value="[]")
            with self.assertRaisesRegex(AssertionError, "unknown|unresolved"):
                case._close_owned()

    def test_second_submit_waits_for_independent_list_visibility(self):
        with tempfile.TemporaryDirectory() as directory:
            case = github_live.GithubLiveTest("test_real_provider_creates_one_issue_and_finds_it_on_repeat")
            state = {"submits": 0, "reads_after_create": 0, "visible": False, "closed": False}
            process = Mock()
            process.wait.return_value = 0

            def issue(number=1):
                return {"number": number, "title": case.token, "body": case.plan["body"],
                        "html_url": f"https://github.com/owner/fixture/issues/{number}",
                        "state": "closed" if state["closed"] else "open"}

            def api(endpoint, *arguments, **kwargs):
                if endpoint == "repos/owner/fixture":
                    return json.dumps({"full_name": "owner/fixture", "has_issues": True, "archived": False,
                                       "description": "woodpecker-acceptance-test-repository",
                                       "permissions": {"push": True}})
                if "?" in endpoint:
                    if not state["submits"]:
                        return "[]"
                    state["reads_after_create"] += 1
                    if state["reads_after_create"] == 1:
                        return "[]"  # Real observed lag: successful list read lacks the new issue.
                    state["visible"] = True
                    return json.dumps([issue()])
                if "PATCH" in arguments:
                    state["closed"] = True
                return json.dumps(issue())

            def request(_, message, **kwargs):
                action = message["action"]
                if action == "report":
                    return {"title": case.token, "signature": "v1:sha256:" + "a" * 64,
                            "body": case.token + "\n<!-- bug_hunter-signature: v1:sha256:" + "a" * 64 + " -->\n"}
                if action == "submit":
                    state["submits"] += 1
                    existing = state["submits"] == 2 and state["visible"]
                    number = 1 if state["submits"] == 1 or existing else 2
                    return {"kind": "existing" if existing else "created", "reference": {"kind": "issue",
                            "reference": {"identifier": str(number),
                                          "url": f"https://github.com/owner/fixture/issues/{number}"}}}
                return None

            case._gh = Mock(side_effect=api)
            with patch.dict(os.environ, {"WOODPECKER_TEST_RENDERED": "1",
                    "WOODPECKER_TEST_GITHUB_REPO": "owner/fixture", "WOODPECKER_TEST_GITHUB_CLEANUP": "close"}), \
                    patch.object(github_live.shutil, "which", return_value="unused-gh"), \
                    patch.object(github_live.tempfile, "mkdtemp", return_value=directory), \
                    patch.object(Path, "is_file", return_value=True), \
                    patch.object(github_live.subprocess, "Popen", return_value=process), \
                    patch.object(github_live, "request", side_effect=request), \
                    patch.object(github_live, "cleanup"):
                result = unittest.TestResult()
                case.run(result)
            self.assertEqual(result.errors, [])
            self.assertEqual(result.failures, [])
            self.assertEqual(state["submits"], 2, "Never retry a publication")
            self.assertTrue(state["visible"])
            self.assertTrue(state["closed"])
