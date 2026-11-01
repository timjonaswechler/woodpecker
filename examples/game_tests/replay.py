"""Record five movement ticks in A, stop A, replay in fresh B, compare the result."""
import json
from pathlib import Path
import shutil
import time

from _cli import CONFIG, LEVEL, PLAYER, SEED, SETTINGS, TICKS, Run, arguments


def main():
    args = arguments(__doc__)
    with Run(args, "replay") as game:
        source = game.own(game.cli("session", "create", "--config", str(CONFIG))["id"])
        source_detail = game.wait_engine(source)
        startup_ticks = game.resource(source, TICKS)["ticks"]

        deadline = time.monotonic() + 10
        for extra_ticks in range(121):
            load = game.resource(source, LEVEL)
            if load == "Ready":
                break
            assert load == "Loading", load
            if extra_ticks == 120 or time.monotonic() >= deadline:
                raise TimeoutError("Level A not ready within 120 explicit ticks / 10 s")
            assert game.call(source, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
        game.note(preparation={"session": source, "engine_ticks": startup_ticks, "level_ticks": extra_ticks})

        seed = json.loads(SEED.read_text(encoding="utf-8"))
        assert game.resource(source, SETTINGS) == seed
        initial = game.resource(source, PLAYER)["position"]
        before = game.resource(source, TICKS)["ticks"]
        recording = "recordings/walk.jsonl"
        assert game.call(source, "recording.start", {"path": recording}) == {"path": recording}
        # Exactly four recorded commands; no session-local entity handles.
        assert game.call(source, "input.keyboard.press", {"key": "arrow_right"}) is None
        assert game.call(source, "tick.warp.start", {"ticks": 4}) == {
            "requested_ticks": 4, "executed_ticks": 4, "outcome": "completed",
        }
        assert game.call(source, "input.keyboard.release", {"key": "arrow_right"}) is None
        assert game.call(source, "tick.warp.start", {"ticks": 1}) == {
            "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
        }
        assert game.call(source, "recording.stop", {}) == {
            "path": recording, "recorded_commands": 4,
        }
        expected_position = game.resource(source, PLAYER)["position"]
        expected_delta = game.resource(source, TICKS)["ticks"] - before
        assert expected_delta == 5
        assert expected_position[0] > initial[0], "The recording must not be a no-op"
        assert abs(expected_position[1] - initial[1]) <= 1e-6, expected_position
        source_file = Path(source_detail["artifact_dir"]) / recording
        game.stop(source)  # Confirm Ended BEFORE starting B: never two live windows.

        target = game.own(game.cli("session", "create", "--config", str(CONFIG))["id"])
        assert target != source
        target_detail = game.wait_engine(target)
        startup_ticks = game.resource(target, TICKS)["ticks"]

        deadline = time.monotonic() + 10
        for extra_ticks in range(121):
            load = game.resource(target, LEVEL)
            if load == "Ready":
                break
            assert load == "Loading", load
            if extra_ticks == 120 or time.monotonic() >= deadline:
                raise TimeoutError("Level B not ready within 120 explicit ticks / 10 s")
            assert game.call(target, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
        game.note(preparation={"session": target, "engine_ticks": startup_ticks, "level_ticks": extra_ticks})

        assert game.resource(target, SETTINGS) == seed
        assert game.resource(target, PLAYER)["position"] == initial, "Replay needs the same starting state"
        target_before = game.resource(target, TICKS)["ticks"]
        # Copy only our closed recording, between roots returned by public session inspect.
        # Exclusive creation prevents accidental overwrite; leave both copies as artifacts.
        target_file = Path(target_detail["artifact_dir"]) / recording
        target_file.parent.mkdir(parents=True, exist_ok=True)
        with source_file.open("rb") as recorded, target_file.open("xb") as copied:
            shutil.copyfileobj(recorded, copied)
        game.note(copy={"source": str(source_file), "target": str(target_file)})

        # This waits for the correlated terminal result using ONLY Activity polls.
        # Do not issue Inspect, ticks or other external game commands during Replay.
        assert game.call(target, "replay.start", {"path": recording}) == {
            "outcome": {"kind": "completed"},
        }
        assert game.resource(target, PLAYER)["position"] == expected_position
        assert game.resource(target, TICKS)["ticks"] - target_before == expected_delta
        game.note(observation={"final_position": expected_position, "tick_delta": expected_delta})
        game.stop(target)
    print("PASS: fresh B matched A's final position and five-tick delta; both owned sessions ended.")


if __name__ == "__main__":
    main()
