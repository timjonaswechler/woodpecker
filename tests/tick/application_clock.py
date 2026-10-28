"""Application time policy, virtual speed and pause survive explicit SDK ticks."""
import os
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config


class ApplicationClockTest(unittest.TestCase):
    def test_application_clock_keeps_its_step_speed_and_pause(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test starts three Signalstation windows serially with the opt-in "
            "application clock: steady, double and paused. Manual startup; exactly "
            "three ticks per session, no inputs. Permission for this run is required: "
            "WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue(
            (ROOT / "target/debug/test_app").is_file(),
            "Build first: cargo build --locked --manifest-path test_app/Cargo.toml "
            "--features automation --bin test_app",
        )
        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        virtual_type = "bevy_time::time::Time<bevy_time::virt::Virtual>"
        # The app's declared 50-ms policy supplies these worked examples,
        # independently of the observed delta or reported executed_ticks.
        for mode, expected_elapsed, expected_delta, expected_speed in (
            ("steady", 0.15, 0.05, 1.0),
            ("double", 0.30, 0.10, 2.0),
            ("paused", 0.0, 0.0, 1.0),
        ):
            server = start_server(self, f"application-clock-{mode}")
            config = server.artifacts / "application-clock.toml"
            configuration = load_config()
            configuration["tick"]["clock"] = {"kind": "application"}
            configuration["launch"]["arguments"][:0] = [
                "--woodpecker-diagnostics", "--woodpecker-clock", mode,
            ]
            write_config(config, configuration)
            server.record(["variant", mode], ticks=3, expected_elapsed=expected_elapsed,
                          expected_delta=expected_delta, expected_speed=expected_speed)
            session_id = server.cli("session", "create", "--config", str(config))["id"]
            deadline = time.monotonic() + 30
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertIsNone(detail["error"], (mode, detail))
                if detail["state"] == "SessionReady":
                    break
                self.assertEqual(detail["state"], "Starting", (mode, detail))
                if time.monotonic() >= deadline:
                    self.fail(f"{mode} did not become ready; see {server.artifacts}")
                time.sleep(0.05)

            before_ticks = server.read_resource(session_id, ticks_type)
            before_time = server.read_resource(session_id, time_type)
            output = server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 3},
            })
            self.assertEqual(output, {
                "requested_ticks": 3, "executed_ticks": 3, "outcome": "completed",
            }, mode)
            after_ticks = server.read_resource(session_id, ticks_type)
            after_time = server.read_resource(session_id, time_type)
            virtual = server.read_resource(session_id, virtual_type)
            self.assertEqual(after_ticks["ticks"] - before_ticks["ticks"], 3, mode)
            self.assertAlmostEqual(
                after_time["elapsed_secs_f64"] - before_time["elapsed_secs_f64"],
                expected_elapsed, delta=1e-12, msg=mode,
            )
            self.assertAlmostEqual(after_time["delta_secs_f64"], expected_delta, delta=1e-12, msg=mode)
            self.assertEqual(virtual["context"]["paused"], mode == "paused", mode)
            self.assertEqual(virtual["context"]["relative_speed"], expected_speed, mode)

            # App-owned time is also frozen outside a warp, including while paused.
            server.record(["client", "pause"], seconds=0.25, mode=mode)
            time.sleep(0.25)
            self.assertEqual(server.read_resource(session_id, ticks_type), after_ticks, mode)
            self.assertEqual(server.read_resource(session_id, time_type), after_time, mode)
            self.assertEqual(server.read_resource(session_id, virtual_type), virtual, mode)
            self.assertEqual(server.cli("session", "inspect", session_id)["state"], "SessionReady")
            self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
            self.assertEqual(server.process.wait(timeout=5), 0, mode)
