"""Full IDs and unique prefixes keep selecting the same live or ended session."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server


class SessionBindingTest(unittest.TestCase):
    def test_full_id_and_prefix_bind_to_the_same_retained_session(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window with manual startup. Full-ID "
            "and prefix queries, explicit session stop; no ticks or inputs. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "session-binding")
        session_id = server.cli("session", "create", "--config", str(ROOT / "tests/app.toml"))["id"]
        deadline = time.monotonic() + 30
        while True:
            ready = server.cli("session", "inspect", session_id)
            self.assertIsNone(ready["error"], ready)
            if ready["state"] == "SessionReady":
                break
            self.assertEqual(ready["state"], "Starting", ready)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not become ready; see {server.artifacts}")
            time.sleep(0.05)

        prefix = session_id[:8]
        listed = server.cli("session", "ls")
        self.assertEqual([item["id"] for item in listed if item["id"].startswith(prefix)], [session_id])
        self.assertEqual(server.cli("session", "inspect", prefix), ready)
        self.assertEqual(server.cli("session", "inspect", session_id), ready)
        ticks_type = "test_app::woodpecker::SimulationTicks"
        for selector in (session_id, prefix):
            # Poll and submit use new WebSocket bindings, not management GETs.
            activity = server.cli("session", "poll", selector)
            self.assertEqual(activity["kind"], "activity", activity)
            self.assertEqual(activity["cursor"]["session_id"], session_id)
            self.assertTrue(all(entry["cursor"]["session_id"] == session_id
                                for entry in activity["entries"]))
            pending = server.cli("session", "submit", selector, "--command", json.dumps({
                "command": "inspect.query", "arguments": {
                    "source": "resources", "selector": {"kind": "type", "type_path": ticks_type},
                    "projection": {"kind": "value"},
                },
            }))
            # Activity always contains the full ID, even when submitted by prefix.
            outcome = server.result(session_id, pending)
            self.assertEqual(outcome["kind"], "completed", outcome)
            self.assertEqual(outcome["output"], {"items": [{"kind": "resource", "result": {
                "kind": "value", "type_path": ticks_type,
                "value": {"status": "readable", "value": {"ticks": 0}},
            }}]})

        server.cli("session", "stop", prefix)
        deadline = time.monotonic() + 10
        while True:
            ended = server.cli("session", "inspect", session_id)
            self.assertIsNone(ended["error"], ended)
            if ended["state"] == "Ended":
                break
            self.assertEqual(ended["state"], "Stopping", ended)
            if time.monotonic() >= deadline:
                self.fail(f"Session did not end; see {server.artifacts}")
            time.sleep(0.05)
        self.assertEqual(ended["id"], ready["id"])
        self.assertEqual(ended["created_at"], ready["created_at"])
        self.assertEqual(ended["artifact_dir"], ready["artifact_dir"])
        for selector in (session_id, prefix):
            self.assertEqual(server.cli("session", "inspect", selector), ended)
            activity = server.cli("session", "poll", selector)
            self.assertEqual(activity["kind"], "activity", activity)
            self.assertEqual(activity["cursor"]["session_id"], session_id)
            self.assertTrue(any(entry["event"].get("kind") == "lifecycle"
                                and entry["event"]["state"] == "Ended" for entry in activity["entries"]))
        listed = server.cli("session", "ls")
        self.assertEqual(len(listed), 1, listed)
        self.assertEqual((listed[0]["id"], listed[0]["state"]), (session_id, "Ended"))
        server.record(["observation", "binding"], full_id=session_id, prefix=prefix,
                      ready=ready, ended=ended)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
