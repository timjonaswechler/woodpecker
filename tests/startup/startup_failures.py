"""Engine-startup failures stay bounded and never become EngineReady."""
import json
import os
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config


class StartupFailuresTest(unittest.TestCase):
    def test_startup_prerequisites_and_budgets_fail_with_their_own_diagnostics(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test starts four Signalstation windows serially with opt-in startup "
            "faults. Each permits at most 3600 startup ticks and ten startup seconds; "
            "the pending variant permits only two ticks. No input or external ticks. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        for mode, max_ticks, expected_code, expected_ticks in (
            ("missing-scope", 3600, "startup_asset_scope_missing", 0),
            ("pending", 2, "startup_tick_limit", 2),
            ("missing-asset", 3600, "startup_asset_failed", None),
            ("no-camera", 3600, "startup_timeout", None),
        ):
            # Reap a separate server before the next variant. A Failed lifecycle
            # alone does not promise its worker has finished process cleanup.
            server = start_server(self, f"startup-failure-{mode}", expected_exit_code=1)
            config = server.artifacts / "startup-failure.toml"
            configuration = load_config()
            configuration["startup"] = {"kind": "engine", "timeout_seconds": 10, "max_ticks": max_ticks}
            configuration["launch"]["arguments"][:0] = [
                "--woodpecker-diagnostics", "--woodpecker-startup", mode,
            ]
            write_config(config, configuration)
            server.record(["variant", mode], max_ticks=max_ticks, timeout_seconds=10,
                          expected_code=expected_code)
            created = server.cli("session", "create", "--config", str(config))
            self.assertEqual(created["state"], "Starting", mode)
            session_id = created["id"]
            deadline = time.monotonic() + 50
            while True:
                detail = server.cli("session", "inspect", session_id)
                if detail["state"] == "Failed":
                    break
                self.assertIn(detail["state"], ("Starting", "SessionReady"), (mode, detail))
                self.assertIsNone(detail["error"], (mode, detail))
                if time.monotonic() >= deadline:
                    self.fail(f"{mode} did not fail within its bounded startup; see {server.artifacts}")
                time.sleep(0.05)
            self.assertEqual(detail["error"]["code"], expected_code, (mode, detail))

            events = []
            cursor = None
            deadline = time.monotonic() + 5
            while True:
                arguments = ["session", "poll", session_id, "--wait-ms", "100"]
                if cursor is not None:
                    arguments.extend(["--cursor", json.dumps(cursor)])
                activity = server.cli(*arguments)
                self.assertEqual(activity["kind"], "activity", activity)
                self.assertEqual(activity["cursor"]["session_id"], session_id, activity)
                if cursor is not None:
                    self.assertGreaterEqual(activity["cursor"]["position"], cursor["position"])
                for entry in activity["entries"]:
                    self.assertEqual(entry["cursor"]["session_id"], session_id, entry)
                    events.append(entry["event"])
                states = [event["state"] for event in events if event["kind"] == "lifecycle"]
                if "Failed" in states:
                    break
                if time.monotonic() >= deadline:
                    self.fail(f"{mode}: terminal activity missing; see {server.artifacts}")
                cursor = activity["cursor"]
            self.assertIn("SessionReady", states, mode)
            self.assertNotIn("EngineReady", states, mode)
            self.assertFalse(any(event["kind"] == "event"
                                 and event["event"]["kind"] == "engine_ready" for event in events), mode)
            failures = [event["event"] for event in events
                        if event["kind"] == "event" and event["event"]["kind"] == "startup_failed"]
            self.assertEqual(len(failures), 1, (mode, failures))
            failure = failures[0]
            self.assertEqual(failure["error"], detail["error"], mode)
            self.assertIsInstance(failure["ticks"], int, mode)
            self.assertGreaterEqual(failure["ticks"], 0, mode)
            self.assertLessEqual(failure["ticks"], max_ticks, mode)
            if expected_ticks is not None:
                self.assertEqual(failure["ticks"], expected_ticks, mode)
            if mode == "missing-asset":
                self.assertIn("__woodpecker_missing_startup__.png", failure["error"]["message"])
            if mode == "no-camera":
                self.assertIn("camera=false", failure["error"]["message"])
            server.record(["observation", "startup-failed"], mode=mode, failure=failure)

            self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
            self.assertEqual(server.process.wait(timeout=5), 1, mode)
            self.assertIn(
                "shutdown_incomplete",
                (server.artifacts / "server.stderr.log").read_text(encoding="utf-8"),
            )
