"""Fresh CLI clients correlate out-of-order outcomes within and across sessions."""
import json
import os
import time
import unittest

from tests._support import ROOT, WINDOW_POSITIONS, start_server, load_config, write_config


class RequestCorrelationTest(unittest.TestCase):
    def test_request_ids_and_outcomes_stay_with_their_client_and_session(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens two staggered Signalstation windows with manual startup. "
            "A requests 60 ticks at 2 ticks/s and is explicitly stopped early; B runs "
            "three maximum-speed ticks. Multiple one-shot CLI clients, no inputs. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "request-correlation")
        sessions = []
        for label, (x, y) in zip(("A", "B"), WINDOW_POSITIONS):
            config = server.artifacts / f"session-{label}.toml"
            configuration = load_config()
            configuration["launch"]["arguments"][:0] = ["--window-position", f"{x},{y}"]
            write_config(config, configuration)
            sessions.append(server.cli("session", "create", "--config", str(config))["id"])
        session_a, session_b = sessions
        self.assertNotEqual(session_a, session_b)
        for session_id in sessions:
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "SessionReady":
                    break
                self.assertEqual(detail["state"], "Starting", detail)
                if time.monotonic() >= deadline:
                    self.fail(f"Session did not become ready; see {server.artifacts}")
                time.sleep(0.05)

        # Management queries above consume no command IDs. Each submit below
        # comes from a fresh CLI process and therefore a different connection.
        warp_a = server.cli("session", "submit", session_a, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 60, "pace": {"kind": "ticks_per_second", "target": 2.0},
            },
        }))
        warp_b = server.cli("session", "submit", session_b, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {"ticks": 3},
        }))
        for pending in (warp_a, warp_b):
            self.assertEqual(pending["kind"], "pending", pending)
            self.assertGreater(pending["request_id"], 0)
        self.assertEqual(warp_a["request_id"], warp_b["request_id"])
        completed_b = server.result(session_b, warp_b)
        self.assertEqual(completed_b["kind"], "completed", completed_b)
        self.assertEqual(completed_b["output"], {
            "requested_ticks": 3, "executed_ticks": 3, "outcome": "completed",
        })

        settings_type = "test_app::app::settings::Settings"
        query_a = server.cli("session", "submit", session_a, "--command", json.dumps({
            "command": "inspect.query", "arguments": {
                "source": "resources", "selector": {"kind": "type", "type_path": settings_type},
                "projection": {"kind": "value"},
            },
        }))
        self.assertGreater(query_a["request_id"], warp_a["request_id"])
        ticks_type = "test_app::woodpecker::SimulationTicks"
        query_b = server.cli("session", "submit", session_b, "--command", json.dumps({
            "command": "inspect.query", "arguments": {
                "source": "resources", "selector": {"kind": "type", "type_path": ticks_type},
                "projection": {"kind": "value"},
            },
        }))
        self.assertGreater(query_b["request_id"], warp_b["request_id"])
        self.assertEqual(query_a["request_id"], query_b["request_id"])
        outcome_b = server.result(session_b, query_b)
        self.assertEqual(outcome_b["kind"], "completed", outcome_b)
        self.assertEqual(len(outcome_b["output"]["items"]), 1)
        item_b = outcome_b["output"]["items"][0]
        self.assertEqual(item_b["kind"], "resource")
        self.assertEqual(item_b["result"]["type_path"], ticks_type)
        self.assertEqual(item_b["result"]["value"]["status"], "readable")
        self.assertEqual(item_b["result"]["value"]["value"], {"ticks": 3})
        # Read A only after B's same-numbered query completed. A global ID-only
        # result slot must not substitute B's counter for A's Settings value.
        outcome_a = server.result(session_a, query_a)
        self.assertEqual(outcome_a["kind"], "completed", outcome_a)
        self.assertEqual(len(outcome_a["output"]["items"]), 1)
        item_a = outcome_a["output"]["items"][0]
        self.assertEqual(item_a["kind"], "resource")
        self.assertEqual(item_a["result"]["type_path"], settings_type)
        self.assertEqual(item_a["result"]["value"]["status"], "readable")
        self.assertEqual(item_a["result"]["value"]["value"]["player_name"], "Woodpecker-Abnahme")

        activity_a = server.cli("session", "poll", session_a)
        self.assertEqual(activity_a["kind"], "activity", activity_a)
        self.assertEqual(activity_a["cursor"]["session_id"], session_a)
        terminals = {entry["event"].get("request_id") for entry in activity_a["entries"]
                     if entry["event"]["kind"] in ("completed", "rejected", "failed")}
        self.assertIn(query_a["request_id"], terminals)
        self.assertNotIn(warp_a["request_id"], terminals, "A must still be open after A's later query and B finish")
        stop_a = server.cli("session", "submit", session_a, "--command", json.dumps({
            "command": "tick.warp.stop", "arguments": {},
        }))
        self.assertGreater(stop_a["request_id"], query_a["request_id"])
        stopped = server.result(session_a, stop_a)
        self.assertEqual(stopped["kind"], "completed", stopped)
        self.assertEqual(stopped["output"], {"was_running": True})
        outcome_a = server.result(session_a, warp_a)
        self.assertEqual(outcome_a["kind"], "completed", outcome_a)
        self.assertEqual(outcome_a["output"]["requested_ticks"], 60)
        self.assertEqual(outcome_a["output"]["outcome"], "stopped")
        self.assertLess(outcome_a["output"]["executed_ticks"], 60)
        # Both first-request outcomes now coexist too: A's stopped warp cannot
        # overwrite the retained completion of B's same-numbered request.
        self.assertEqual(server.result(session_b, warp_b), completed_b)

        for session_id, expected_ids in (
            (session_a, [warp_a["request_id"], query_a["request_id"], stop_a["request_id"]]),
            (session_b, [warp_b["request_id"], query_b["request_id"]]),
        ):
            activity = server.cli("session", "poll", session_id)
            self.assertEqual(activity["kind"], "activity", activity)
            self.assertEqual(activity["cursor"]["session_id"], session_id)
            self.assertTrue(all(entry["cursor"]["session_id"] == session_id
                                for entry in activity["entries"]))
            pending_ids = [entry["event"]["request_id"] for entry in activity["entries"]
                           if entry["event"]["kind"] == "pending"]
            completed_ids = [entry["event"]["request_id"] for entry in activity["entries"]
                             if entry["event"]["kind"] == "completed"]
            self.assertEqual(pending_ids, expected_ids)
            self.assertCountEqual(completed_ids, expected_ids)
            if session_id == session_a:
                self.assertLess(completed_ids.index(query_a["request_id"]),
                                completed_ids.index(warp_a["request_id"]))
            server.record(["observation", "request-correlation"], session_id=session_id,
                          pending_ids=pending_ids, completed_ids=completed_ids)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
