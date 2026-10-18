"""Keep recording evidence strict when comparing independent rendered sessions."""
import json
from pathlib import Path
import tempfile
import unittest

from tests.recording.contracts import read_recording
from tests.session.signalstation import NAME, RESOURCES, TRANSFORM
from tests.workflows.replay import STATE, checkpoint_values, compare


CLOCK = {"kind": "fixed", "seconds_per_tick": 0.016666667}


class RecordingEvidenceTests(unittest.TestCase):
    def test_fixed_clock_recording_requires_its_clock_and_complete_footer(self):
        rows = [{"type": "recording_started", "format_version": 2, "clock": CLOCK},
                {"type": "recording_ended", "outcome": "stopped", "recorded_commands": 0}]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "clock.jsonl"
            path.write_text("\n".join(json.dumps(row) for row in rows) + "\n")
            self.assertEqual(read_recording(path, clock=CLOCK), rows)
            with self.assertRaises(AssertionError):
                read_recording(path, clock={"kind": "fixed", "seconds_per_tick": 0.02})
            path.write_text(json.dumps(rows[0]) + "\n")
            with self.assertRaises(AssertionError):
                read_recording(path, clock=CLOCK)

    def test_checkpoints_compare_relative_time_and_named_camera_without_entity_identity(self):
        def checkpoint(origin, entity):
            rows, indices = [], {}
            for name in (*STATE, "time"):
                value = ({"elapsed": {"secs": 0, "nanos": origin + 16666667},
                          "delta": {"secs": 0, "nanos": 16666667}}
                         if name == "time" else {"observed": name})
                indices[name] = len(rows)
                rows.append({"command": "inspect.query", "arguments": {
                    "selector": {"kind": "type", "type_path": RESOURCES[name]}},
                    "outcome": {"status": "completed", "output": {"items": [{"result": {
                        "type_path": RESOURCES[name], "value": {"status": "readable", "value": value}}}]}}})
            indices["camera"] = len(rows)
            rows.append({"outcome": {"status": "completed", "output": {"items": [{"entity": entity,
                "result": {"components": [
                    {"component": {"type_path": NAME}, "value": {"status": "readable", "value": "Spielkamera"}},
                    {"component": {"type_path": TRANSFORM}, "value": {"status": "readable", "value": {
                        "translation": [-32.0, 1.5, 22.0], "rotation": [0.0, 0.0, 0.0, 1.0]}}}]}}]}}})
            return rows, indices

        source, indices = checkpoint(100000000, {"index": 2, "generation": 1})
        replay, _ = checkpoint(200000000, {"index": 99, "generation": 7})
        expected = checkpoint_values(source, indices, 100000000)
        actual = checkpoint_values(replay, indices, 200000000)
        compare(expected, actual)
        self.assertEqual(actual["time"], {"elapsed_nanos": 16666667, "delta_nanos": 16666667})
        actual["time"]["elapsed_nanos"] += 16666667
        with self.assertRaises(AssertionError):
            compare(expected, actual)
        with self.assertRaises(AssertionError):
            checkpoint_values(source, {key: value for key, value in indices.items() if key != "settings"}, 100000000)

    def test_changed_game_state_missing_values_and_nan_cannot_pass(self):
        expected = {"player": {"position": [-32.0, 22.0]}, "notes": "Grüße 世界🙂"}
        for actual in ({"player": {"position": [-32.0, 21.0]}, "notes": "Grüße 世界🙂"},
                       {"player": {"position": [-32.0, 22.0]}},
                       {"player": {"position": [-32.0, float("nan")]}, "notes": "Grüße 世界🙂"}):
            with self.assertRaises(AssertionError):
                compare(expected, actual)


if __name__ == "__main__":
    unittest.main()
