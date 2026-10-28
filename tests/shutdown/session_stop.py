"""Management explicitly stops Replay/Warp and recording before shutting down one session."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import (load_config, owned_process, start_server,
                            wait_process_gone, write_config)
from tests.recording import read_jsonl, write_jsonl
from tests.shutdown import no_failures, progress


class SessionStopTest(unittest.TestCase):
    def test_management_stop_drains_replay_warp_and_recording_without_running_tail(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One manual game window with opt-in process/counter files. Replay's "
                         "120-tick Warp at 2/s is stopped through session stop while recording. "
                         "The planned g/one-tick tail must not run; no actual input. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        server = start_server(self, "management-session-stop")
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
            if time.monotonic() >= deadline:
                self.fail("SessionReady deadline")
            time.sleep(0.03)
        root = Path(detail["artifact_dir"])
        process = owned_process(server, root)
        plan = [
            {"type": "recording_started", "format_version": 2, "clock": {"kind": "fixed", "seconds_per_tick": 0.016666667}},
            {"type": "command", "command": "tick.warp.start", "arguments": {
                "ticks": 120, "pace": {"kind": "ticks_per_second", "target": 2.0}}, "outcome": {
                "status": "completed", "output": {"requested_ticks": 120, "executed_ticks": 120, "outcome": "completed"}}},
            {"type": "command", "command": "input.keyboard.press", "arguments": {"key": "g"},
             "outcome": {"status": "completed", "output": None}},
            {"type": "command", "command": "tick.warp.start", "arguments": {"ticks": 1},
             "outcome": {"status": "completed", "output": {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"}}},
            {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 3},
        ]
        write_jsonl(root / "recordings/plan.jsonl", plan)
        server.call(session_id, {"command": "recording.start", "arguments": {"path": "recordings/observed.jsonl"}})
        cursor = server.cli("session", "poll", session_id)["cursor"]
        replay = server.cli("session", "submit", session_id, "--command",
                            '{"command":"replay.start","arguments":{"path":"recordings/plan.jsonl"}}')
        # The real application's existing Last counter is mirrored atomically.
        # Internal replay Pendings are not assumed to be public server Activity.
        reached = progress(root)
        self.assertLess(reached, 120)
        server.cli("session", "stop", session_id)
        outcome = server.result(session_id, replay, timeout=15)
        self.assertEqual(outcome["kind"], "completed", outcome)
        self.assertEqual(outcome["output"], {"outcome": {"kind": "stopped"}})
        deadline = time.monotonic() + 15
        while server.cli("session", "inspect", session_id)["state"] != "Ended":
            if time.monotonic() >= deadline:
                self.fail("Managed stop did not finish")
            time.sleep(0.02)
        self.assertIsNone(server.process.poll())
        events = [row["event"] for row in server.cli("session", "poll", session_id, "--cursor", json.dumps(cursor))["entries"]]
        self.assertTrue(no_failures(events), events)
        completed = [e["command"] for e in events if e["kind"] == "completed"]
        self.assertIn("replay.stop", completed)
        self.assertLess(completed.index("replay.start"), completed.index("recording.stop"))
        self.assertLess(completed.index("recording.stop"), completed.index("shutdown"))
        rows = read_jsonl(root / "recordings/observed.jsonl")
        names = [row["command"] for row in rows[1:-1]]
        self.assertIn(names, [["tick.warp.start", "tick.warp.stop"],
                              ["tick.warp.start", "tick.warp.stop", "tick.warp.stop"]])
        warp = rows[1]["outcome"]["output"]
        self.assertEqual(warp["outcome"], "stopped")
        self.assertGreaterEqual(warp["executed_ticks"], reached)
        self.assertLess(warp["executed_ticks"], 120)
        self.assertEqual(rows[2]["outcome"]["output"], {"was_running": True})
        if len(names) == 3:
            self.assertEqual(rows[3]["outcome"]["output"], {"was_running": False})
        self.assertEqual(rows[-1], {"type": "recording_ended", "outcome": "stopped", "recorded_commands": len(names)})
        self.assertEqual(json.loads((root / "simulation-ticks.json").read_text()), {"ticks": warp["executed_ticks"]})
        self.assertEqual(read_jsonl(root / "recordings/plan.jsonl"), plan)
        self.assertEqual(server.cli("session", "stop", session_id)["state"], "Ended")
        wait_process_gone(process)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
