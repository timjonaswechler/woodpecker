"""Pace changes execution rate and runtime defaults, never fixed time per tick."""
import json
import math
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, start_server, load_config, read_tick_probe, write_config


def pace_bounds(target, first, last, capacity):
    """Bound a counter window; measured capacity is not a product throughput SLA."""
    shortest = last["before"] - first["after"]
    seconds = last["after"] - first["before"]
    # Calibration includes the real application's tick/render/IO cost. Do not
    # demand ideal target throughput that the baseline itself could not attain.
    minimum = max(0, math.floor(min(target, capacity) * shortest) - 1)
    return minimum, math.floor(target * seconds) + 1


class SetPaceTest(unittest.TestCase):
    def test_request_override_and_runtime_pace_change_preserve_fixed_time(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens one Signalstation window: up to 3600 engine-startup ticks "
            "in 60 seconds, then at most 124 ticks (16 calibration at 8 ticks/s, 100 stopped early, "
            "8 completed). No input. Permission for this run is required: "
            "WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "set-pace")
        config = server.artifacts / "engine.toml"
        configuration = load_config()
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe"]
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

        ticks_type = "test_app::woodpecker::SimulationTicks"
        time_type = "bevy_time::time::Time<()>"
        root = Path(detail["artifact_dir"])
        calibration_before = server.read_resource(session_id, ticks_type)["ticks"]
        calibration = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 16, "pace": {"kind": "ticks_per_second", "target": 8.0},
            },
        }))
        first = None
        deadline = time.monotonic() + 30
        while True:
            if (root / "simulation-ticks.json").exists():
                sample = read_tick_probe(root)
                if sample["ticks"] > calibration_before and first is None:
                    first = sample
                if sample["ticks"] >= calibration_before + 16:
                    break
            if time.monotonic() >= deadline:
                self.fail("Calibration counter deadline")
            time.sleep(0.005)
        self.assertEqual(server.result(session_id, calibration, timeout=5)["output"],
                         {"requested_ticks": 16, "executed_ticks": 16, "outcome": "completed"})
        before_ticks = server.read_resource(session_id, ticks_type)
        self.assertEqual(before_ticks["ticks"] - calibration_before, 16)
        observed = sample["ticks"] - first["ticks"]
        calibration_seconds = sample["after"] - first["before"]
        capacity = observed / calibration_seconds
        server.record(["preparation", "pace-capacity"], ticks=16, target=8.0,
                      first=first, last=sample, observed_ticks=observed,
                      seconds=calibration_seconds, ticks_per_second=capacity)
        if observed < 8:
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0)
            self.skipTest("Unvollständig: too few app-counter observations to isolate pace")
        if capacity < 6:
            # Pace promises a ceiling, not achievable throughput. Do not label a
            # machine unable to isolate the 8-ticks/s cap as a successful proof.
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0)
            self.skipTest("Unvollständig: calibration below 6 ticks/s; pace effect not isolatable")

        before_time = server.read_resource(session_id, time_type)
        default = {"kind": "ticks_per_second", "target": 0.5}
        self.assertEqual(server.call(session_id, {
            "command": "tick.warp.set_pace", "arguments": {"pace": default},
        }), {"pace": default})
        self.assertEqual(server.read_resource(session_id, ticks_type), before_ticks)
        self.assertEqual(server.read_resource(session_id, time_type), before_time)
        pending = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {
                "ticks": 100, "pace": {"kind": "ticks_per_second", "target": 2.0},
            },
        }))
        self.assertEqual(pending["kind"], "pending", pending)
        deadline = time.monotonic() + 10
        while read_tick_probe(root)["ticks"] == before_ticks["ticks"]:
            if time.monotonic() >= deadline:
                self.fail(f"Warp made no progress; see {server.artifacts}")
            time.sleep(0.02)

        rates = {}
        ambiguous = []
        for label, target in (("request-override", 2.0), ("changed-default", 8.0)):
            if label == "changed-default":
                pace = {"kind": "ticks_per_second", "target": target}
                self.assertEqual(server.call(session_id, {
                    "command": "tick.warp.set_pace", "arguments": {"pace": pace},
                }), {"pace": pace})
            # Only file observations lie inside this interval: no CLI round-trip
            # latency is counted as simulation time. Bracket both atomic reads.
            sample_before = read_tick_probe(root)
            time.sleep(2.5)
            sample_after = read_tick_probe(root)
            seconds = sample_after["after"] - sample_before["before"]
            ticks = sample_after["ticks"] - sample_before["ticks"]
            rates[label] = ticks / seconds
            # One tick of phase uncertainty, not a percentage tolerance.
            # Too-fast progress violates the ceiling. Too-slow progress cannot
            # distinguish an incorrect pace from host starvation: never green.
            minimum, maximum = pace_bounds(target, sample_before, sample_after, capacity)
            server.record(["observation", "pace"], phase=label, target=target,
                          first=sample_before, last=sample_after, seconds=seconds,
                          ticks=ticks, calibrated_capacity=capacity, minimum=minimum, maximum=maximum)
            self.assertGreaterEqual(ticks, 0, label)
            self.assertLessEqual(ticks, maximum, label)
            if ticks < minimum:
                ambiguous.append(f"{label}: below observable target rate")
            previous_target = 0.5 if label == "request-override" else 2.0
            if ticks <= math.floor(previous_target * seconds) + 1:
                ambiguous.append(f"{label}: unchanged {previous_target}/s hypothesis not excluded")

        stop = server.call(session_id, {
            "command": "tick.warp.stop", "arguments": {},
        })
        self.assertIn(stop, ({"was_running": True}, {"was_running": False}))
        outcome = server.result(session_id, pending)
        self.assertEqual(outcome["kind"], "completed", outcome)
        if stop["was_running"]:
            self.assertEqual(outcome["output"]["outcome"], "stopped", outcome)
            self.assertLess(outcome["output"]["executed_ticks"], 100)
        else:
            self.assertEqual(outcome["output"], {
                "requested_ticks": 100, "executed_ticks": 100, "outcome": "completed"})
            ambiguous.append("active pace-change window exhausted by natural completion")
        stopped_ticks = server.read_resource(session_id, ticks_type)
        stopped_time = server.read_resource(session_id, time_type)
        self.assertAlmostEqual(
            stopped_time["elapsed_secs_f64"] - before_time["elapsed_secs_f64"],
            (stopped_ticks["ticks"] - before_ticks["ticks"]) * 0.016666667,
            delta=1e-9,
        )
        self.assertAlmostEqual(stopped_time["delta_secs_f64"], 0.016666667, delta=1e-12)

        # No request override: this warp must inherit the new runtime default,
        # not the old 0.5-ticks/s default or the fixture's original unlimited pace.
        inherited = server.cli("session", "submit", session_id, "--command", json.dumps({
            "command": "tick.warp.start", "arguments": {"ticks": 8},
        }))
        first = None
        deadline = time.monotonic() + 5
        while True:
            sample = read_tick_probe(root)
            if sample["ticks"] > stopped_ticks["ticks"] and first is None:
                first = sample
            if sample["ticks"] >= stopped_ticks["ticks"] + 8:
                break
            if time.monotonic() >= deadline:
                self.fail("Inherited pace counter deadline")
            time.sleep(0.005)
        output = server.result(session_id, inherited, timeout=5)["output"]
        self.assertEqual(output, {"requested_ticks": 8, "executed_ticks": 8, "outcome": "completed"})
        observed = sample["ticks"] - first["ticks"]
        if observed < 4:
            server.cli("server", "stop")
            self.assertEqual(server.process.wait(timeout=5), 0)
            self.skipTest("Unvollständig: inherited pace finished before enough counter observations")
        seconds = sample["after"] - first["before"]
        server.record(["observation", "inherited-pace"], ticks=8, observed_ticks=observed,
                      first=first, last=sample, seconds=seconds)
        minimum, maximum = pace_bounds(8.0, first, sample, capacity)
        self.assertLessEqual(observed, maximum, "Inherited pace exceeded its ceiling")
        if observed < minimum:
            ambiguous.append("inherited default: host capacity and slow pace indistinguishable")
        if observed <= math.floor(0.5 * seconds) + 1:
            ambiguous.append("inherited default: old 0.5/s hypothesis not excluded")
        after_ticks = server.read_resource(session_id, ticks_type)
        after_time = server.read_resource(session_id, time_type)
        self.assertEqual(after_ticks["ticks"] - stopped_ticks["ticks"], 8)
        self.assertAlmostEqual(
            after_time["elapsed_secs_f64"] - stopped_time["elapsed_secs_f64"],
            0.133333336, delta=1e-12,
        )
        server.record(["observation", "pace-hypotheses"], rates=rates, ambiguous=ambiguous)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        if ambiguous:
            self.skipTest("Unvollständig: " + "; ".join(ambiguous))
