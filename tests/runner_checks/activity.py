"""Activity observation advances its cursor without losing out-of-order outcomes."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tests._support import Server


def activity(position, *events):
    return dict(kind="activity", cursor=dict(session_id="s", position=position), entries=[
        dict(cursor=dict(session_id="s", position=index), event=event)
        for index, event in enumerate(events)])


def completed(request_id, output=None):
    return dict(kind="completed", request_id=request_id, command="inspect.query", output=output)


class ActivityTest(unittest.TestCase):
    def test_cursor_prevents_duplicate_logs_and_preserves_out_of_order_outcomes(self):
        replies = [activity(2, completed(1, "large-sentinel"), completed(2)), activity(3, completed(3))]
        with tempfile.TemporaryDirectory() as directory, patch("tests._support.subprocess.run") as run:
            run.side_effect = [subprocess.CompletedProcess([], 0, json.dumps(reply), "") for reply in replies]
            server = Server(None, Path(directory))
            pending = lambda request: dict(kind="pending", request_id=request, command="inspect.query")
            self.assertEqual(server.result("s", pending(1))["output"], "large-sentinel")
            self.assertEqual(server.result("s", pending(2))["request_id"], 2)
            self.assertEqual(server.result("s", pending(1))["output"], "large-sentinel")
            self.assertEqual(server.result("s", pending(3))["request_id"], 3)
            self.assertEqual(run.call_count, 2)
            command = run.call_args.args[0]
            self.assertEqual(json.loads(command[command.index("--cursor") + 1]), replies[0]["cursor"])
            self.assertEqual((Path(directory) / "commands.jsonl").read_text().count("large-sentinel"), 1)

    def test_explicit_cursor_is_honored_even_for_a_cached_request(self):
        reply = activity(1, completed(1))
        with tempfile.TemporaryDirectory() as directory, patch("tests._support.subprocess.run") as run:
            run.return_value = subprocess.CompletedProcess([], 0, json.dumps(reply), "")
            server = Server(None, Path(directory))
            pending = dict(kind="pending", request_id=1, command="inspect.query")
            server.result("s", pending)
            cursor = dict(session_id="s", position=0)
            server.result("s", pending, cursor=cursor)
            self.assertEqual(run.call_count, 2)
            command = run.call_args.args[0]
            self.assertEqual(json.loads(command[command.index("--cursor") + 1]), cursor)
            with self.assertRaisesRegex(AssertionError, "Uncorrelated"):
                server.result("s", dict(pending, command="input.text"))
