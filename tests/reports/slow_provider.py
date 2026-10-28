"""An explicitly stalled external gh does not block accepted game work or other clients."""
import json
import os
from pathlib import Path
import tempfile
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.providers import gh_fixture
from tests.reports import observe, reports


class SlowProviderTest(unittest.TestCase):
    def test_owned_gh_can_wait_while_game_activity_and_other_clients_progress(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual game window, F12 tracing error and one trigger tick plus "
                         "two progress ticks (three total). Only an external fake gh is held/released; "
                         "no network or real issues. Permission required: WOODPECKER_TEST_RENDERED=1")
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        fixture = Path(tempfile.mkdtemp(prefix="slow-gh-fixture-", dir=runs))
        self.artifact_dirs = [str(fixture)]
        gh = fixture / "bin"
        environment = gh_fixture(gh)
        environment["RUST_LOG"] = "error"
        (gh / "mode.json").write_text('{"mode":"hold"}\n')
        server = start_server(self, "slow-provider", env=environment)
        self.addCleanup((gh / "release").write_text, "cleanup release\n")
        configuration = load_config()
        configuration["report"].update(tracing_errors=True, provider={"kind": "github"})
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics"]
        path = server.artifacts / "config.toml"
        write_config(path, configuration)
        session_id = server.cli("session", "create", "--config", str(path))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            if time.monotonic() >= deadline:
                self.fail("Readiness deadline exceeded")
            time.sleep(0.03)
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f12"}}))
        server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.release", "arguments": {"key": "f12"}}))
        deadline = time.monotonic() + 15
        while not (gh / "entered.json").exists():
            if time.monotonic() >= deadline:
                self.fail("Owned gh never reached its hold point")
            time.sleep(0.02)
        gh_pid = json.loads((gh / "entered.json").read_text())["pid"]
        self.assertFalse((gh / "release").exists())
        self.assertEqual(server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 2}})["executed_ticks"], 2)
        self.assertEqual(server.read_resource(session_id, "test_app::woodpecker::SimulationTicks"), {"ticks": 3})
        self.assertEqual(server.read_resource(session_id, "test_app::app::settings::Settings"),
                         json.loads((ROOT / "tests/settings.json").read_text()))
        activity = server.cli("session", "poll", session_id)
        self.assertFalse(reports([row["event"] for row in activity["entries"]]))
        self.assertFalse((gh / "release").exists(), "Progress must precede provider release")
        os.kill(gh_pid, 0)
        (gh / "release").write_text("explicit release\n")
        events = observe(server, session_id, lambda e: len(reports(e)) >= 1)
        self.assertEqual(len(reports(events)), 1)
        self.assertEqual(reports(events)[0]["result"], {"status": "submitted", "outcome": {
            "kind": "created", "reference": {"kind": "issue", "reference": {
                "identifier": "12", "url": "https://example.invalid/issues/12"}}}})
        # Final external effects are immutable only after the owning server joined.
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        calls = [json.loads(line) for line in (gh / "calls.jsonl").read_text().splitlines()]
        self.assertEqual(len(calls), 2)
        self.assertTrue(all(call["prompt"] == "1" for call in calls))
        self.assertTrue(all(Path(call["cwd"]) == (ROOT / "test_app").resolve() for call in calls))
        self.assertEqual(calls[1]["body"]["title"], "Signalstation diagnostic tracing error")
        self.assertIn(reports(events)[0]["report"]["signature"]["value"], calls[1]["body"]["body"])
        with self.assertRaises(ProcessLookupError):
            os.kill(gh_pid, 0)
