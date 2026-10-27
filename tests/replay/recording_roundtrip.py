"""Replay restores checkpoints with equal simulation setup in staggered windows."""
import json
import os
from pathlib import Path
import shutil
import time
import unittest

from tests._support import ROOT, WINDOW_POSITIONS, start_server, load_config, write_config


class RecordingRoundtripTest(unittest.TestCase):
    def test_recording_replays_intermediate_and_final_positions_in_a_fresh_session(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens two Signalstation windows. Each permits up to 3600 startup "
            "and 120 readiness ticks; five ticks are recorded in A and replayed in B. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        server = start_server(self, "recording-roundtrip")
        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        player_type = "test_app::player::Player"
        settings_type = "test_app::app::settings::Settings"
        sessions = []
        for label, (x, y) in zip(("A", "B"), WINDOW_POSITIONS):
            config = server.artifacts / f"engine-{label}.toml"
            configuration = load_config()
            configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
            configuration["launch"]["arguments"][:0] = ["--window-position", f"{x},{y}"]
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
                    self.fail(f"Engine {label} did not become ready; see {server.artifacts}")
                time.sleep(0.05)
            server.record(["preparation", "engine-ready"], session_id=session_id,
                          ticks=server.read_resource(session_id, ticks_type)["ticks"])

            # Neither session creation nor game-readiness ticks are hidden in helpers.
            deadline = time.monotonic() + 10
            for additional_ticks in range(121):
                load = server.read_resource(session_id, "test_app::level::LevelLoad")
                if load == "Ready":
                    break
                self.assertEqual(load, "Loading", load)
                if additional_ticks == 120 or time.monotonic() >= deadline:
                    self.fail(f"Game {label} not ready after {additional_ticks} extra ticks")
                output = server.call(session_id, {
                    "command": "tick.warp.start", "arguments": {"ticks": 1},
                })
                self.assertEqual(output["outcome"], "completed", output)
            server.record(["preparation", "game-ready"], session_id=session_id,
                          additional_ticks=additional_ticks)
            sessions.append({
                "id": session_id,
                "root": Path(detail["artifact_dir"]),
                "ticks": server.read_resource(session_id, ticks_type)["ticks"],
                "time": server.read_resource(session_id, time_type)["elapsed_secs_f64"],
                "position": server.read_resource(session_id, player_type)["position"],
                "settings": server.read_resource(session_id, settings_type),
            })
        source, target = sessions
        self.assertNotEqual(source["id"], target["id"])
        self.assertEqual(target["settings"], source["settings"])
        self.assertEqual(target["position"], source["position"])

        # Resource selectors are session-independent; no entity handle is reused.
        recording_path = "recordings/source.jsonl"
        self.assertEqual(server.call(source["id"], {
            "command": "recording.start", "arguments": {"path": recording_path},
        }), {"path": recording_path})
        server.call(source["id"], {
            "command": "input.keyboard.press", "arguments": {"key": "arrow_right"},
        })
        output = server.call(source["id"], {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        })
        self.assertEqual(output["outcome"], "completed", output)
        checkpoints = [server.read_resource(source["id"], player_type)["position"]]
        output = server.call(source["id"], {
            "command": "tick.warp.start", "arguments": {"ticks": 3},
        })
        self.assertEqual(output["outcome"], "completed", output)
        checkpoints.append(server.read_resource(source["id"], player_type)["position"])
        server.call(source["id"], {
            "command": "input.keyboard.release", "arguments": {"key": "arrow_right"},
        })
        output = server.call(source["id"], {
            "command": "tick.warp.start", "arguments": {"ticks": 1},
        })
        self.assertEqual(output["outcome"], "completed", output)
        checkpoints.append(server.read_resource(source["id"], player_type)["position"])
        stopped = server.call(source["id"], {"command": "recording.stop", "arguments": {}})
        self.assertEqual(stopped, {"path": recording_path, "recorded_commands": 8})
        self.assertNotEqual(checkpoints[-1], source["position"], "The reference run must not be a no-op")
        source_ticks = server.read_resource(source["id"], ticks_type)["ticks"] - source["ticks"]
        source_time = server.read_resource(source["id"], time_type)["elapsed_secs_f64"] - source["time"]
        self.assertEqual(source_ticks, 5)
        self.assertAlmostEqual(source_time, 0.083333335, delta=1e-9)

        source_file = source["root"] / recording_path
        replay_file = target["root"] / recording_path
        replay_file.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_file, replay_file)
        server.record(["artifact", "copy"], source=str(source_file), destination=str(replay_file))

        # A second recording observes the actual internal replay query results.
        # Recorded outcomes are not treated as replay assertions by Woodpecker.
        verification_path = "recordings/replayed.jsonl"
        server.call(target["id"], {
            "command": "recording.start", "arguments": {"path": verification_path},
        })
        self.assertEqual(server.call(target["id"], {
            "command": "replay.start", "arguments": {"path": recording_path},
        }), {"outcome": {"kind": "completed"}})
        self.assertEqual(server.call(target["id"], {
            "command": "recording.stop", "arguments": {},
        }), {"path": verification_path, "recorded_commands": 8})

        # Compare intermediate observations from both public recording artefacts.
        # Never issue a prohibited external Inspect while Replay is active.
        for path in (source_file, target["root"] / verification_path):
            observed = []
            for line in path.read_text(encoding="utf-8").splitlines():
                entry = json.loads(line)
                if entry["type"] == "command" and entry["command"] == "inspect.query":
                    self.assertEqual(entry["outcome"]["status"], "completed", entry)
                    value = entry["outcome"]["output"]["items"][0]["result"]["value"]
                    self.assertEqual(value["status"], "readable", entry)
                    observed.append(value["value"]["position"])
            self.assertEqual(observed, checkpoints, str(path))
        self.assertEqual(server.read_resource(target["id"], player_type)["position"], checkpoints[-1])
        self.assertEqual(server.read_resource(target["id"], ticks_type)["ticks"] - target["ticks"], 5)
        target_time = server.read_resource(target["id"], time_type)["elapsed_secs_f64"] - target["time"]
        self.assertAlmostEqual(target_time, source_time, delta=1e-9)

        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
