"""Engine startup reports counted ticks, then leaves further progress to the caller."""
import json
import os
import time
import unittest

from tests._support import ROOT, start_server, load_config, write_config


class EngineStartupTest(unittest.TestCase):
    def test_engine_startup_reports_ticks_and_then_stays_idle(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window, allows up to 3600 engine-startup "
            "ticks and up to 120 explicit game-readiness ticks. Permission for this run "
            "is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "engine-startup")

        # This is an explicit per-test variant, not a hidden warm-up in a helper.
        config = server.artifacts / "engine.toml"
        configuration = load_config()
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
        write_config(config, configuration)
        session_id = server.cli("session", "create", "--config", str(config))["id"]
        deadline = time.monotonic() + 90
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "EngineReady":
                break
            self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
            if time.monotonic() >= deadline:
                self.fail(f"Engine did not become ready; see {server.artifacts}")
            time.sleep(0.05)

        # A lifecycle transition can be observed just before its event is published.
        # Follow the public activity cursor until the EngineReady payload arrives.
        cursor = None
        events = []
        deadline = time.monotonic() + 5
        while True:
            arguments = ["session", "poll", session_id, "--wait-ms", "100"]
            if cursor is not None:
                arguments.extend(["--cursor", json.dumps(cursor)])
            activity = server.cli(*arguments)
            self.assertEqual(activity["kind"], "activity", activity)
            events.extend(entry["event"] for entry in activity["entries"])
            ready = [event["event"] for event in events
                     if event["kind"] == "event" and event["event"]["kind"] == "engine_ready"]
            if ready:
                break
            if time.monotonic() >= deadline:
                self.fail(f"EngineReady event missing; see {server.artifacts}")
            cursor = activity["cursor"]
        self.assertEqual(len(ready), 1, ready)
        states = [event["state"] for event in events if event["kind"] == "lifecycle"]
        self.assertEqual(states.count("SessionReady"), 1, states)
        self.assertEqual(states.count("EngineReady"), 1, states)
        self.assertLess(states.index("SessionReady"), states.index("EngineReady"))
        startup_ticks = ready[0]["ticks"]
        self.assertGreater(startup_ticks, 0)
        self.assertLessEqual(startup_ticks, 3600)

        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        before_ticks = server.read_resource(session_id, ticks_type)
        before_time = server.read_resource(session_id, time_type)
        self.assertEqual(before_ticks, {"ticks": startup_ticks})
        server.record(["client", "pause"], seconds=0.25, engine_startup_ticks=startup_ticks)
        time.sleep(0.25)
        self.assertEqual(server.read_resource(session_id, ticks_type), before_ticks)
        self.assertEqual(server.read_resource(session_id, time_type), before_time)

        # Technical EngineReady is not a promise about the game's loading state.
        # Any additional game preparation is visible, bounded and counted here.
        deadline = time.monotonic() + 10
        for additional_ticks in range(121):
            load = server.read_resource(session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            self.assertEqual(load, "Loading", load)
            if additional_ticks == 120 or time.monotonic() >= deadline:
                self.fail(f"Game not ready after {additional_ticks} extra ticks; see {server.artifacts}")
            output = server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            })
            self.assertEqual(output["outcome"], "completed", output)
        self.assertEqual(
            server.read_resource(session_id, ticks_type),
            {"ticks": startup_ticks + additional_ticks},
        )
        server.record(["preparation", "game-ready"], additional_ticks=additional_ticks)

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
