"""Controlled time crosses the real CLI/REPL, launcher, plugin and replay path."""
import json
import time

from tests.catalog import scenario
from tests.connection.activity_retention.contracts import query


def fixed_config(ctx, seconds):
    path = ctx.program("activity")
    with path.open("a") as file:
        file.write(f'\n[tick.clock]\nkind = "fixed"\nseconds_per_tick = {seconds}\n')
    return path


def counter(driver):
    value = driver.command("inspect.query", query("Counter"))["items"][0]["result"]["value"]["value"]
    driver.session.pids.add(value["process_id"])
    return value


@scenario("connection", "fixed_clock_survives_delays_and_replay", capabilities=(),
          programs=("activity",), independent=True,
          commands=("tick.warp.start", "inspect.query", "recording.start", "recording.stop", "replay.start"))
def fixed_clock_survives_delays_and_replay(ctx):
    with ctx.session(fixed_config(ctx, 0.01)) as source:
        assert source.session.detail()["state"] == "Ready"
        if ctx.interface == "repl":
            assert f"connected {source.session.id} Ready" in source.lines
        assert counter(source)["elapsed_nanos"] == 0
        source.command("recording.start", {"path": "clock.jsonl"})
        source.command("tick.warp.start", {"ticks": 1})
        first = counter(source)
        assert first["elapsed_nanos"] == first["delta_nanos"] == 10_000_000, first
        time.sleep(0.05)
        assert counter(source) == first, "Reale Denkpause hat die Spielzeit verändert"
        source.command("tick.warp.start", {"ticks": 2,
            "pace": {"kind": "ticks_per_second", "target": 100}})
        source.command("recording.stop", {})
        expected = counter(source)
        assert expected["ticks"] == 3 and expected["elapsed_nanos"] == 30_000_000, expected
        assert expected["delta_nanos"] == 10_000_000, expected
        recording = (source.session.artifacts / "clock.jsonl").read_text()
        header = json.loads(recording.splitlines()[0])
        assert header == {"type": "recording_started", "format_version": 2,
                          "clock": {"kind": "fixed", "seconds_per_tick": 0.01}}, header

        with ctx.session(fixed_config(ctx, 0.01)) as destination:
            (destination.session.artifacts / "clock.jsonl").write_text(recording)
            assert destination.command("replay.start", {"path": "clock.jsonl"}) == {
                "outcome": {"kind": "completed"}}
            actual = counter(destination)
            for field in ("ticks", "elapsed_nanos", "delta_nanos"):
                assert actual[field] == expected[field], (field, actual, expected)

        with ctx.session(fixed_config(ctx, 0.02)) as different:
            (different.session.artifacts / "clock.jsonl").write_text(recording)
            different.command("replay.start", {"path": "clock.jsonl"},
                              expected_error="replay_clock_mismatch")
            unchanged = counter(different)
            assert unchanged["ticks"] == unchanged["elapsed_nanos"] == 0, unchanged
