"""Interactive mode is planning-only until explicit, compatible, prebuilt graphics use."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace
from unittest.mock import MagicMock, patch

from tests import interactive


class InteractiveTest(unittest.TestCase):
    def test_default_group_planning_contains_only_the_ten_compatible_cases(self):
        data = interactive.plan([arg for category in ("keyboard", "pointer", "text")
                                 for arg in ("--category", category)])
        self.assertEqual(len(data["selected"]), 10)
        self.assertTrue(all(row["module"] in interactive._shared.MODULES for row in data["selected"].values()))

    def test_isolated_cases_and_build_flags_are_rejected_without_starting_a_process(self):
        with patch.object(interactive.subprocess, "Popen") as process:
            for arguments in (["--category", "shutdown"], ["--category", "keyboard", "--build-only"],
                              ["--test", "--list"], ["--category", "keyboard", "--test", "3.12"]):
                with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                    interactive.plan(arguments)
            process.assert_not_called()

    def test_failed_cleanup_cannot_become_successful_by_repeating_close(self):
        game = interactive._shared.SharedGame()
        def failure():
            raise AssertionError("owned server exited 1")
        game.owner.stack.callback(failure)
        with self.assertRaisesRegex(AssertionError, "owned server"):
            game.close()
        self.assertTrue(game.failed)
        with self.assertRaisesRegex(RuntimeError, "Previous shared app cleanup failed"):
            game.close()

    def test_explicit_stop_ends_host_even_when_cleanup_fails(self):
        import fcntl
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "target/test-runs").mkdir(parents=True)
            state = root / "target/shared-app.json"
            game = interactive._shared.SharedGame()
            game.identity = {"pid": 123}
            game.session_id = "session"
            game.server = SimpleNamespace(address="127.0.0.1:1234")
            def failure():
                raise AssertionError("owned server exited 1")
            game.owner.stack.callback(failure)
            listener, connection, lease = MagicMock(), MagicMock(), MagicMock()
            listener.__enter__.return_value = listener
            connection.__enter__.return_value = connection
            listener.getsockname.return_value = ("127.0.0.1", 1234)
            listener.accept.side_effect = [(connection, None), AssertionError("host accepted another request")]
            connection.makefile.return_value.readline.return_value = b'{"instance":"test","operation":"stop"}\n'
            with patch.object(interactive, "ROOT", root), patch.object(interactive, "STATE", state), \
                    patch.object(interactive._shared, "SharedGame", return_value=game), \
                    patch.object(game, "acquire"), patch.object(interactive, "implementation", return_value="test"), \
                    patch.object(interactive.uuid, "uuid4", return_value=SimpleNamespace(hex="test")), \
                    patch.object(interactive._artifacts, "load", return_value={"bin:woodpecker": root / "cli", "bin:test_app": root / "app"}), \
                    patch.object(interactive, "use_products", return_value=lease), \
                    patch.object(interactive.socket, "socket", return_value=listener), \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(interactive.main(["--host", "--artifacts", str(root / "products.json")]), 1)
            self.assertEqual(listener.accept.call_count, 1)
            response = json.loads(connection.sendall.call_args.args[0])
            self.assertFalse(response["ok"])
            self.assertIn("owned server exited 1", json.loads(state.read_text())["cleanup_error"])
            lease.__exit__.assert_called_once()
            with (root / "target/shared-app.lock").open("a") as lock:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)

    def test_existing_state_is_not_automatically_replaced_after_connection_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory) / "state.json"
            state.write_text('{"instance":"old", "port":1234, "manifest":"/old/products.json"}')
            with patch.object(interactive, "STATE", state), \
                    patch.dict(interactive.os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                    patch.object(interactive._artifacts, "load", return_value={}), \
                    patch.object(interactive, "request", side_effect=ConnectionRefusedError("old host died")), \
                    patch.object(interactive.subprocess, "Popen") as process, \
                    contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(interactive.main(["--category", "keyboard"]), 1)
                process.assert_not_called()
                self.assertTrue(state.exists())
