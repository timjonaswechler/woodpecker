"""Inspect returns the configured resource value from a real Signalstation session."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class ExpectationControlFailure(AssertionError):
    """Identifies only the deliberate 3.18 assertion, never a setup or SDK failure."""


class ResourceValueTest(unittest.TestCase):
    # Set only on the single instance selected by acceptance --negative-control.
    wrong_expected_player_name = None

    def test_resource_value_matches_the_settings_seed(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window. Explicit permission for this run "
            "is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        expected_settings = json.loads((ROOT / "tests/settings.json").read_text(encoding="utf-8"))
        server = start_server(self, "resource-value")

        # Settings exist at SessionReady. No game loading, input or tick is needed.
        session_id = server.cli(
            "session", "create", "--config", str(ROOT / "tests/app.toml"),
        )["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            self.assertEqual(detail["state"], "Starting", detail)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not reach SessionReady; see {server.artifacts}")
            time.sleep(0.05)

        type_path = "test_app::app::settings::Settings"
        query = {
            "command": "inspect.query",
            "arguments": {
                "source": "resources",
                "selector": {"kind": "type", "type_path": type_path},
                "projection": {"kind": "value"},
            },
        }
        pending = server.cli("session", "submit", session_id, "--command", json.dumps(query))
        self.assertEqual(pending["kind"], "pending", pending)
        self.assertEqual(pending["command"], "inspect.query", pending)
        self.assertGreater(pending["request_id"], 0, pending)

        # Polling observes this accepted request; it never resubmits the query.
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome, {
            "kind": "completed",
            "request_id": pending["request_id"],
            "command": "inspect.query",
            "output": {
                "items": [{
                    "kind": "resource",
                    "result": {
                        "kind": "value",
                        "type_path": type_path,
                        "value": {"status": "readable", "value": expected_settings},
                    },
                }],
            },
        })

        # The optional control first requires the complete correct response above.
        # It changes only an assertion, never the seed or the observed app value.
        if self.wrong_expected_player_name is not None:
            actual = outcome["output"]["items"][0]["result"]["value"]["value"]["player_name"]
            if actual != self.wrong_expected_player_name:
                raise ExpectationControlFailure(
                    f"3.18: deliberately wrong player name: "
                    f"{actual!r} != {self.wrong_expected_player_name!r}"
                )

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
