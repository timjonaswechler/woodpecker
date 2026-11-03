"""Repeatable rendered Signalstation measurements; no builds or implicit retries.

Run from the repository root: python3 examples/performance.py --help.
The direct game protocol removes CLI startup/polling from Warp timing. It uses
real windows, explicit counted ticks, fixed time, and the game's own counters.
"""
import argparse
import json
import os
import hashlib
from pathlib import Path
import queue
import statistics
import subprocess
import threading
import time

ROOT = Path(__file__).resolve().parents[1]
TICKS = "test_app::woodpecker::SimulationTicks"
TIME = "bevy_time::time::Time<()>"
METRICS = "test_app::performance::Metrics"


class Game:
    def __init__(self, binary, directory, extra):
        self.directory = directory.resolve()
        self.directory.mkdir(parents=True, exist_ok=False)
        (self.directory / "config").mkdir()
        self.errors = (directory / "game.stderr.log").open("w")
        env = {**{key: value for key, value in os.environ.items() if not key.startswith("WOODPECKER_")},
               "WOODPECKER_ARTIFACT_DIR": str(self.directory),
               "WOODPECKER_SESSION_DIR": str(self.directory),
               "WOODPECKER_TICK_CLOCK": json.dumps({"kind": "fixed", "seconds_per_tick": 0.016666667}),
               "WOODPECKER_STARTUP": json.dumps({"kind": "manual"}),
               "WOODPECKER_TICK_PACE": json.dumps({"kind": "as_fast_as_possible"})}
        self.process = subprocess.Popen([str(binary.resolve()), "--settings-seed",
            str(ROOT / "tests/settings.json"), "--fixed-window",
            "--scale-factor", "1", "--window-position", "100,100", *extra], cwd=ROOT, env=env, stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=self.errors, text=True, bufsize=1)
        self.messages = queue.Queue()
        self.pending = {}
        self.next_id = 0
        self.journal = (directory / "commands.jsonl").open("w")
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        try:
            for line in self.process.stdout:
                self.messages.put(json.loads(line))
        except BaseException as error:
            self.messages.put(error)
        finally:
            self.messages.put(EOFError("Game stdout closed"))

    def ready(self):
        message = self.messages.get(timeout=60)
        if isinstance(message, BaseException):
            raise message
        assert message["status"] == "ready", message
        self.raise_window()

    def raise_window(self):
        # Raise only our directly owned game, without generating mouse/key input.
        assert self.process.poll() is None, "Game exited before visibility check"
        subprocess.run(["osascript", "-e", 'tell application "System Events" to set frontmost of first process whose unix id is '
                        + str(self.process.pid) + ' to true'], check=True, timeout=10)

    def send(self, command, arguments):
        self.next_id += 1
        message = dict(request_id=self.next_id, command=command, arguments=arguments)
        self.journal.write(json.dumps({"at": time.monotonic(), **message}) + "\n")
        self.journal.flush()
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()
        return self.next_id

    def result(self, request_id, timeout=120, optional=False):
        deadline = time.monotonic() + timeout
        while request_id not in self.pending:
            message = self.messages.get(timeout=max(0.001, deadline - time.monotonic()))
            if isinstance(message, BaseException):
                raise message
            self.journal.write(json.dumps({"at": time.monotonic(), "response": message}) + "\n")
            self.pending[message["request_id"]] = message
        message = self.pending.pop(request_id)
        if optional and message["status"] == "rejected" and message["error"]["code"] == "unknown_type_path":
            return None
        assert message["status"] == "completed", message
        return message["output"]

    def call(self, command, arguments):
        return self.result(self.send(command, arguments))

    def resource(self, name, optional=False):
        request = self.send("inspect.query", {"source": "resources",
            "selector": {"kind": "type", "type_path": name}, "projection": {"kind": "value"}})
        output = self.result(request, optional=optional)
        if output is None:
            return None
        value = output["items"][0]["result"]["value"]
        if optional and value["status"] != "readable":
            return None
        assert value["status"] == "readable", output
        return value["value"]

    def warp(self, ticks, pace=None):
        arguments = {"ticks": ticks}
        if pace is not None:
            arguments["pace"] = pace
        result = self.call("tick.warp.start", arguments)
        assert result == dict(requested_ticks=ticks, executed_ticks=ticks, outcome="completed"), result

    def close(self):
        try:
            if self.process.poll() is None:
                self.call("tick.warp.stop", {})
                self.call("shutdown", {})
                self.process.wait(timeout=10)
            assert self.process.returncode == 0, self.process.returncode
        finally:
            if self.process.poll() is None:
                # Only this directly owned child. A forced exit is not passing evidence.
                self.process.kill()
                self.process.wait()
            self.reader.join(timeout=2)
            self.process.stdin.close()
            self.process.stdout.close()
            self.errors.close()
            self.journal.close()


def run(args):
    args.output.mkdir(parents=True, exist_ok=False)
    manifest = dict(binary=str(args.binary.resolve()), arguments=args.app_arg,
        ticks=args.ticks, repeats=args.repeats, warmup_ticks=120,
        fixed_seconds_per_tick=0.016666667, window_pixels=[1440, 900],
        scale_factor=1, window_position=[100, 100], setup_tick_limit=120, checkout_head=subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        hardware=subprocess.check_output(["system_profiler", "SPDisplaysDataType"], text=True),
        build_profile="dev (repository default)")
    manifest["mode"] = "native" if args.native else "controlled"
    manifest["checkout_dirty"] = bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True))
    if args.native:
        for field in ("ticks", "repeats", "warmup_ticks", "fixed_seconds_per_tick", "setup_tick_limit"):
            manifest[field] = None
        manifest["observation_seconds"] = 8
        manifest["keep_owned_window_exposed"] = True
    manifest["binary_sha256"] = hashlib.sha256(args.binary.read_bytes()).hexdigest()
    manifest["raise_owned_window"] = True
    (args.output / "manifest.json").write_text(json.dumps(manifest, indent=2))
    if args.native:
        return native(args, manifest)
    game = Game(args.binary, args.output / "session", args.app_arg)
    rows = []
    try:
        game.ready()
        for setup_ticks in range(121):
            if game.resource("test_app::level::LevelLoad") == "Ready":
                break
            assert setup_ticks < 120, "Level did not become ready within the setup tick budget"
            game.warp(1)
        game.warp(120)
        game.call("screenshot.capture", {"path": "scene.png"})
        scene(game.directory / "scene.png")
        for repeat in range(args.repeats):
            game.raise_window()
            before = game.resource(TICKS)["ticks"]
            before_time = game.resource(TIME)["elapsed_secs_f64"]
            started = time.monotonic()
            game.warp(args.ticks)
            elapsed = time.monotonic() - started
            after = game.resource(TICKS)["ticks"]
            assert after - before == args.ticks, (before, after)
            simulated = game.resource(TIME)["elapsed_secs_f64"] - before_time
            assert abs(simulated - args.ticks * 0.016666667) < 1e-9, simulated
            row = dict(repeat=repeat, ticks=args.ticks, seconds=elapsed, tps=args.ticks / elapsed,
                       metrics=game.resource(METRICS, optional=True))
            rows.append(row)
            print(json.dumps(row), flush=True)
        game.raise_window()
        frozen_ticks = game.resource(TICKS)
        frozen_time = game.resource(TIME)
        time.sleep(1)
        assert game.resource(TICKS) == frozen_ticks
        assert game.resource(TIME) == frozen_time
        idle = game.resource(METRICS, optional=True)
        if args.require_metrics:
            assert idle is not None, "Performance metrics missing"
        if idle is not None:
            assert idle["ticks"] == frozen_ticks["ticks"], idle
            assert idle["tps"] == 0, idle
            assert idle["fps"] > 0, idle
        latencies = []
        for _ in range(5):
            warp = game.send("tick.warp.start", {"ticks": 100000})
            time.sleep(0.1)  # Explicit observation delay, never a hidden simulation action.
            started = time.monotonic()
            game.resource(TICKS)
            inspected = time.monotonic() - started
            started = time.monotonic()
            assert game.call("tick.warp.stop", {}) == {"was_running": True}
            stopped = time.monotonic() - started
            result = game.result(warp)
            assert result["outcome"] == "stopped" and result["executed_ticks"] < 100000, result
            latencies.append(dict(inspect_seconds=inspected, stop_seconds=stopped))
        summary = dict(validated=False, runs=rows, median_tps=statistics.median(row["tps"] for row in rows),
            min_tps=min(row["tps"] for row in rows), max_tps=max(row["tps"] for row in rows),
            paused_metrics=idle, latencies=latencies)
        (args.output / "result.json").write_text(json.dumps(summary, indent=2))
        if args.verify:
            verify(game, args.capture)
        if args.capture:
            time.sleep(1)
            game.raise_window()
            game.call("screenshot.capture", {"path": "performance.png"})
            scene(game.directory / "performance.png")
    finally:
        game.close()
    log = (game.directory / "game.stderr.log").read_text()
    summary["time_channel_warnings"] = log.count("time_system did not receive")
    (args.output / "result.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(dict(time_channel_warnings=summary["time_channel_warnings"])))
    assert subprocess.check_output(["system_profiler", "SPDisplaysDataType"], text=True) == manifest["hardware"], "Display configuration changed during measurement; result is invalid"
    if args.compare:
        reference = json.loads(args.compare.read_text())
        assert reference.get("validated") is True, "Reference did not complete its validation"
        reference_manifest = json.loads(args.compare.with_name("manifest.json").read_text())
        for field in ("hardware", "ticks", "repeats", "warmup_ticks", "fixed_seconds_per_tick", "window_pixels", "scale_factor", "window_position", "build_profile"):
            assert manifest[field] == reference_manifest[field], f"Measurement conditions differ: {field}"
        assert summary["min_tps"] > reference["max_tps"], "No gain outside reference spread"
        for metric in ("inspect_seconds", "stop_seconds"):
            assert max(row[metric] for row in summary["latencies"]) <= max(
                row[metric] for row in reference["latencies"]), f"Latency regression: {metric}"
        print("Comparison passed: every Warp faster than reference, Inspect/Stop maxima no worse.")
    summary["validated"] = True
    (args.output / "result.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary), flush=True)


def scene(path):
    """Fixed start-scene landmarks, not just a valid/non-black PNG.

    These HUD/floor bands are also the existing screenshot fixture's landmarks.
    They reject the observed failure: a correctly counted Warp with no scene.
    """
    from PIL import Image
    with Image.open(path) as image:
        assert image.size == (1440, 900), image.size
        for name, rect, minimum, maximum in (
            ("HUD", (40, 30, 180, 80), (10, 15, 15), (65, 75, 75)),
            ("Entrance floor", (740, 330, 790, 370), (210, 205, 190), (255, 255, 245)),
        ):
            pixels = list(image.convert("RGB").crop(rect).getdata())
            fraction = sum(all(lo <= value <= hi for value, lo, hi in zip(pixel, minimum, maximum))
                           for pixel in pixels) / len(pixels)
            assert fraction >= 0.9, f"Missing {name}: {fraction:.3f} in {path}"


def verify(game, capture):
    """Behavior checks run after measurement, never inside its timed workload."""
    before = game.resource(METRICS)
    for expected in (not before["overlay"], before["overlay"]):
        count = game.resource(TICKS)["ticks"]
        game.call("input.keyboard.press", {"key": "f8"})
        assert game.resource(TICKS)["ticks"] == count, "Toggle input executed a hidden tick"
        game.warp(1)
        assert game.resource(METRICS)["overlay"] == expected
        game.call("input.keyboard.release", {"key": "f8"})
        game.warp(1)
    paced = game.send("tick.warp.start", {"ticks": 30, "pace": {"kind": "ticks_per_second", "target": 20.0}})
    time.sleep(0.6)
    metrics = game.resource(METRICS)
    assert metrics["warp_running"] and metrics["target_tps"] == 20, metrics
    assert 0 < metrics["tps"] <= 21, metrics
    assert metrics["requested_ticks"] == 30 and 0 < metrics["executed_ticks"] < 30, metrics
    assert game.call("tick.warp.stop", {}) == {"was_running": True}
    assert game.result(paced)["outcome"] == "stopped"
    count = game.resource(TICKS)["ticks"]
    active = game.send("tick.warp.start", {"ticks": 100000})
    time.sleep(0.4)
    metrics = game.resource(METRICS)
    assert metrics["warp_running"] and metrics["target_tps"] is None, metrics
    assert metrics["tps"] > 0 and metrics["simulation_speed"] > 0, metrics
    if capture:
        game.raise_window()
        game.call("screenshot.capture", {"path": "warp.png"})
        scene(game.directory / "warp.png")
    assert game.call("tick.warp.stop", {}) == {"was_running": True}
    outcome = game.result(active)
    assert outcome["outcome"] == "stopped"
    assert game.resource(TICKS)["ticks"] - count == outcome["executed_ticks"]
    frozen = game.resource(TIME)
    time.sleep(1)
    metrics = game.resource(METRICS)
    assert metrics["tps"] == 0 and metrics["simulation_speed"] == 0 and metrics["fps"] > 0, metrics
    assert game.resource(TIME) == frozen
    print("Behavior checks passed: F8 toggle, no hidden ticks, paced/max status, progress, Stop, paused clocks.")


def native(args, manifest):
    """Native game: real application time, same scene, bounded wall-clock observation."""
    workspace = (args.output / "native").resolve()
    (workspace / "config").mkdir(parents=True)
    environment = {key: value for key, value in os.environ.items() if not key.startswith("WOODPECKER_")}
    environment["WOODPECKER_SESSION_DIR"] = str(workspace)
    log_path = workspace / "game.stderr.log"
    with log_path.open("w") as log:
        process = subprocess.Popen([str(args.binary.resolve()), "--settings-seed",
            str(ROOT / "tests/settings.json"), "--fixed-window", "--scale-factor", "1",
            "--window-position", "100,100", "--performance-log", *args.app_arg], cwd=ROOT, env=environment,
            stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=log)
        try:
            # Native game initialization, not an implicit controlled-session tick.
            deadline = time.monotonic() + 8
            time.sleep(2)
            while time.monotonic() < deadline:
                assert process.poll() is None, "Native application exited"
                # Keep the declared visible-window precondition throughout the
                # observation. Covered macOS windows legitimately stop rendering.
                subprocess.run(["osascript", "-e", 'tell application "System Events" to set frontmost of first process whose unix id is '
                                + str(process.pid) + ' to true'], check=True, timeout=10)
                time.sleep(min(0.5, max(0, deadline - time.monotonic())))
            assert process.poll() is None, "Native application exited"
        finally:
            if process.poll() is None:
                process.terminate()  # Explicit end of this directly owned native test process.
                process.wait(timeout=10)
    samples = [json.loads(line.removeprefix("SIGNALSTATION_PERFORMANCE "))
               for line in log_path.read_text().splitlines() if line.startswith("SIGNALSTATION_PERFORMANCE ")]
    assert len(samples) >= 8, "Insufficient native samples"
    steady = samples[-8:]
    assert all(not row["controlled"] and not row["warp_running"] for row in steady)
    result = {"validated": False, "samples": steady, "median_fps": statistics.median(row["fps"] for row in steady)}
    (args.output / "result.json").write_text(json.dumps(result, indent=2))
    assert subprocess.check_output(["system_profiler", "SPDisplaysDataType"], text=True) == manifest["hardware"], "Display configuration changed during native measurement"
    if "--uncapped" in args.app_arg:
        assert result["median_fps"] > 60, "Over-60-FPS target not demonstrated on this hardware"
    result["validated"] = True
    (args.output / "result.json").write_text(json.dumps(result, indent=2))
    print(json.dumps(result))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/test_app")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--ticks", type=int, default=600)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--app-arg", action="append", default=[])
    parser.add_argument("--capture", action="store_true")
    parser.add_argument("--verify", action="store_true", help="Exercise overlay toggle, paced/max progress, capture and paused clocks after timing")
    parser.add_argument("--native", action="store_true", help="Observe the native game for a bounded 8 seconds instead of issuing Warps")
    parser.add_argument("--compare", type=Path, help="Reference result.json; require gain beyond spread and no Inspect/Stop latency regression")
    parser.add_argument("--require-metrics", action="store_true", help="Require the new counters; omit only for the old reference binary")
    options = parser.parse_args()
    if options.ticks <= 0 or options.repeats <= 0:
        parser.error("ticks and repeats must be positive")
    run(options)
