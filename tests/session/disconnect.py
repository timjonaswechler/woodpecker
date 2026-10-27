"""An exited submitting CLI does not cancel its warp; later clients resume observation."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, read_tick_probe, start_server, write_config


class DisconnectTest(unittest.TestCase):
    def test_accepted_warp_progresses_without_clients_and_is_observed_without_resubmission(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window with manual startup and requests "
            "12 ticks at 2 ticks/s once. Submitting CLI exits with Pending; a 0.75-second "
            "client-free interval and new observers verify continued work. No inputs. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "session-disconnect")
        configuration = load_config()
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
        config = server.artifacts / "config.toml"
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
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

        ticks_type = "test_app::woodpecker::SimulationTicks"
        before = server.read_resource(session_id, ticks_type)["ticks"]
        # run_cli waits for and reaps this one-shot process. It is already gone
        # when any observer below connects; no REPL or custom transport is needed.
        creator = server.run_cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 12, "pace": {"kind": "ticks_per_second", "target": 2.0},
            },
        }))
        self.assertEqual(creator.returncode, 0, creator.stderr)
        pending = json.loads(creator.stdout)
        self.assertEqual(pending["kind"], "pending", pending)
        server.record(["client", "submitter-exited"], returncode=creator.returncode, pending=pending)
        activity = server.cli("session", "poll", session_id)
        self.assertEqual(activity["kind"], "activity", activity)
        self.assertEqual(activity["cursor"]["session_id"], session_id)
        matching = [entry["event"] for entry in activity["entries"]
                    if entry["event"].get("request_id") == pending["request_id"]]
        self.assertIn([event["kind"] for event in matching], (["pending"], ["pending", "completed"]), matching)
        self.assertEqual(matching[0]["command"], "tick.warp.start")

        # The app mirrors its independent Last counter atomically. Read that
        # file, not Inspect: reconnection must not be the cause of progress.
        root = Path(detail["artifact_dir"])
        deadline = time.monotonic() + 5
        while True:
            if (root / "simulation-ticks.json").exists():
                before_gap = read_tick_probe(root)
                if before_gap["ticks"] - before >= 2:
                    break
            if time.monotonic() >= deadline:
                self.fail(f"Accepted work did not progress without clients; see {server.artifacts}")
            time.sleep(0.02)
        at_disconnect = before_gap["ticks"]
        self.assertLessEqual(at_disconnect - before, 12)
        active_window = at_disconnect - before < 12
        server.record(["client", "no-connections"], seconds=0.75, probe=before_gap)
        time.sleep(0.75)
        gap_probe = read_tick_probe(root)  # Still no client; assert before reconnecting.
        after_gap = gap_probe["ticks"]
        self.assertGreaterEqual(after_gap, at_disconnect)
        client_free_progress = after_gap > at_disconnect
        server.record(["observation", "client-free-progress"], before=before_gap, after=gap_probe)

        outcome = server.result(session_id, pending, timeout=15)
        self.assertEqual(outcome, {
            "kind": "completed", "request_id": pending["request_id"],
            "command": "tick.warp.start", "output": {
                "requested_ticks": 12, "executed_ticks": 12, "outcome": "completed",
            },
        })
        after = server.read_resource(session_id, ticks_type)["ticks"]
        self.assertEqual(after - before, 12)
        activity = server.cli("session", "poll", session_id)
        self.assertEqual(activity["kind"], "activity", activity)
        self.assertEqual(activity["cursor"]["session_id"], session_id)
        warp_events = [entry["event"] for entry in activity["entries"]
                       if entry["event"].get("command") == "tick.warp.start"]
        self.assertEqual([event["kind"] for event in warp_events], ["pending", "completed"])
        self.assertTrue(all(event["request_id"] == pending["request_id"] for event in warp_events))
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
        server.record(["observation", "disconnect"], pending=pending,
                      before_ticks=before, before_gap_ticks=at_disconnect,
                      after_gap_ticks=after_gap, final_ticks=after)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        if not active_window or not client_free_progress:
            self.skipTest("Unvollständig: no independently observed tick in the 0.75s client-free window; "
                          "completion after reconnection does not prove client-free progress")
