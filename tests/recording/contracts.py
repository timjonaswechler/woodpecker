import json

from tests.catalog import scenario

START, STOP = "recording.start", "recording.stop"


def read_recording(path):
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    assert rows[0] == {"type": "recording_started", "format_version": 1}, rows
    assert rows[-1]["type"] == "recording_ended", rows
    commands = rows[1:-1]
    assert rows[-1]["recorded_commands"] == len(commands), rows
    assert all(row["type"] == "command" for row in commands), rows
    assert not any(row["command"] in {START, STOP, "replay.start", "replay.stop", "shutdown"}
                   for row in commands), rows
    return rows


@scenario("recording", "ordered_commands_and_controls", commands=(START, STOP))
def ordered_commands_and_controls(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        relative = "recordings/ordered.jsonl"
        assert driver.command(START, {"path": relative}) == {"path": relative}
        driver.command("tick.warp.start", {"ticks": 3})
        driver.command("tick.warp.start", {"ticks": 0}, expected_error="invalid_tick_count")
        result = driver.command(STOP, {})
        assert result == {"path": relative, "recorded_commands": 2}, result
        rows = read_recording(driver.session.artifacts / relative)
        assert rows[-1]["outcome"] == "stopped"
        assert [row["arguments"]["ticks"] for row in rows[1:-1]] == [3, 0]
        assert [row["outcome"]["status"] for row in rows[1:-1]] == ["completed", "rejected"]
        assert rows[2]["outcome"]["error"]["code"] == "invalid_tick_count", rows
        assert ctx.probe(driver)["ticks"] == before["ticks"] + 3
        driver.command(STOP, {}, expected_error="recording_not_active")


@scenario("recording", "shutdown_closes_active_recording", commands=(START, STOP))
def shutdown_closes_active_recording(ctx):
    with ctx.session() as driver:
        ctx.probe(driver)
        root = driver.session.artifacts
        relative = "recordings/shutdown.jsonl"
        driver.command(START, {"path": relative})
        driver.command("tick.warp.start", {"ticks": 2})
        driver.close()
        driver.session.stop()
        rows = read_recording(root / relative)
        assert rows[-1]["outcome"] == "stopped", rows
        # Management stop also records the internal Warp stop before closing.
        assert [row["command"] for row in rows[1:-1]] == ["tick.warp.start", "tick.warp.stop"], rows
        assert rows[1]["arguments"]["ticks"] == 2
        assert rows[2]["outcome"] == {"status": "completed", "output": {"was_running": False}}
