import json

START, STOP = "recording.start", "recording.stop"


def read_recording(path, clock=None):
    rows = [json.loads(line) for line in path.read_text().splitlines()]
    assert len(rows) >= 2, rows
    header = ({"type": "recording_started", "format_version": 1} if clock is None else
              {"type": "recording_started", "format_version": 2, "clock": clock})
    assert rows[0] == header, rows[0]
    assert rows[-1]["type"] == "recording_ended", rows
    assert rows[-1]["outcome"] in ("stopped", "session_ended"), rows[-1]
    commands = rows[1:-1]
    assert rows[-1]["recorded_commands"] == len(commands), rows
    assert all(row["type"] == "command" for row in commands), rows
    assert not any(row["command"] in {START, STOP, "replay.start", "replay.stop", "shutdown"}
                   for row in commands), rows
    return rows
