"""Isolated harness checks: files, owned process fixtures and mocks; no product starts."""
import contextlib
import io
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest.mock import Mock, patch

from tests import _support
from tests._support import owned_process, wait_process_gone
from tests.faults import gh
from tests.reports import audit_local_files
from tests.shutdown import require_observation_budget
from tests.keyboard import movement
from tests.tick import set_pace, stop_warp


class HardeningHelpersTest(unittest.TestCase):
    def test_forced_server_cleanup_kills_verified_child_groups_before_root_and_still_fails(self):
        snapshot = (f"100 {os.getpid()} 100 Mon Oct 27 10:00:00 2026 owned-server\n"
                    "300 100 300 Mon Oct 27 10:00:01 2026 owned-app\n"
                    "400 300 400 Mon Oct 27 10:00:02 2026 owned-provider\n")
        process = Mock(pid=100, returncode=-int(signal.SIGKILL))
        process.poll.return_value = None
        process.wait.side_effect = [subprocess.TimeoutExpired("server", 5), -int(signal.SIGKILL)]
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(_support.subprocess, "run", return_value=Mock(stdout=snapshot)), \
                patch.object(_support.os, "getpgrp", return_value=9999), \
                patch.object(_support.os, "killpg") as kill, \
                patch.object(_support, "wait_process_gone") as gone:
            with self.assertRaisesRegex(AssertionError, "forced=True"):
                _support.cleanup_server(process, Path(directory))
            self.assertEqual([call.args for call in kill.call_args_list],
                             [(400, signal.SIGKILL), (300, signal.SIGKILL), (100, signal.SIGKILL)])
            self.assertEqual({call.args[0]["pid"] for call in gone.call_args_list}, {100, 300, 400})
            self.assertTrue(json.loads((Path(directory) / "cleanup.json").read_text())["forced"])


    def test_forced_cleanup_refuses_foreign_reused_or_reparented_members(self):
        original = {100: (os.getpid(), 100, "server"), 300: (100, 300, "app"),
                    400: (300, 400, "provider")}
        for mode in ("foreign", "reused", "reparented"):
            current = dict(original)
            if mode == "foreign":
                current[999] = (1, 400, "foreign group member")
            elif mode == "reused":
                current[400] = (300, 400, "new start time and command")
            else:
                current[400] = (1, 400, "provider")
            with self.subTest(mode=mode), \
                    patch.object(_support, "_process_snapshot", side_effect=[original, current]), \
                    patch.object(_support.os, "getpgrp", return_value=9999), \
                    patch.object(_support.os, "killpg") as kill:
                process = Mock(pid=100)
                process.poll.return_value = None
                with self.assertRaisesRegex(RuntimeError, "Refusing signal"):
                    _support.force_owned_tree(process)
                kill.assert_not_called()

    def test_real_owned_tree_is_gone_after_forced_cleanup_without_claiming_success(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            process = subprocess.Popen([sys.executable, str(_support.ROOT / "tests/faults/owned_tree.py"),
                                        str(root), "0"], stdin=subprocess.DEVNULL,
                                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                       start_new_session=True)
            identities = []
            try:
                deadline = time.monotonic() + 5
                while len(list(root.glob("[012].json"))) != 3:
                    self.assertIsNone(process.poll(), "Owned process fixture exited before readiness")
                    if time.monotonic() >= deadline:
                        self.fail("Owned tree fixture readiness deadline")
                    time.sleep(0.01)
                identities = [json.loads((root / f"{level}.json").read_text()) for level in range(3)]
                self.assertEqual(len({item["group"] for item in identities}), 3)
                with self.assertRaisesRegex(AssertionError, "forced=True"):
                    _support.cleanup_server(process, root)
                journal = json.loads((root / "cleanup.json").read_text())
                self.assertTrue(journal["forced"])
                self.assertEqual(journal["returncode"], -int(signal.SIGKILL))
                self.assertCountEqual(journal["forced_owned_processes"], identities)
                for identity in identities:
                    _support.wait_process_gone(identity, timeout=0)
            finally:
                # Each fixture self-expires. On failure, wait for that bound;
                # never signal a now-orphaned or unverified identity as rescue.
                process.wait(timeout=25)
                for identity in identities:
                    _support.wait_process_gone(identity, timeout=25)

    def test_pace_window_uses_observed_capacity_and_preserves_ceiling(self):
        # Captured full-run counter window: 18 ticks after a 7.443/s calibration.
        first = {"ticks": 25, "before": 211122.505802166, "after": 211122.505889791}
        last = {"ticks": 43, "before": 211125.007709083, "after": 211125.007986541}
        minimum, maximum = set_pace.pace_bounds(8.0, first, last, 7.443074281221429)
        self.assertEqual((minimum, maximum), (17, 21))
        self.assertGreaterEqual(last["ticks"] - first["ticks"], minimum)
        self.assertLess(15, minimum, "A 6/s window remains below the calibrated proof threshold")
        self.assertGreater(22, maximum, "Exceeding the requested ceiling remains a failure")
        self.assertEqual(set_pace.pace_bounds(2.0, first, last, 7.443074281221429), (4, 6))
        self.assertEqual(set_pace.pace_bounds(8.0, first, last, 4.0), (9, 21))


    def test_hold_assertions_reject_a_single_lost_middle_tick(self):
        for lost in (False, True):
            with self.subTest(lost=lost), tempfile.TemporaryDirectory() as directory:
                state = {"ticks": 0, "x": 0, "held": False}
                server = SimpleNamespace(artifacts=Path(directory), record=Mock(), process=Mock())
                server.process.wait.return_value = 0
                server.cli = Mock(side_effect=lambda *args, **kwargs:
                                  {"id": "owned"} if args[1] == "create" else
                                  {"state": "EngineReady", "error": None})

                def resource(_, path):
                    if path.endswith("LevelLoad"):
                        return "Ready"
                    if path.endswith("SimulationTicks"):
                        return {"ticks": state["ticks"]}
                    return {"position": [state["x"], 0]}

                def call(_, command):
                    name = command["command"]
                    if name == "input.keyboard.press":
                        state["held"] = True
                    elif name == "input.keyboard.release":
                        state["held"] = False
                    else:
                        count = command["arguments"]["ticks"]
                        for _ in range(count):
                            state["ticks"] += 1
                            if state["held"] and not (lost and state["ticks"] == 3):
                                state["x"] += 1
                        return {"requested_ticks": count, "executed_ticks": count, "outcome": "completed"}

                server.read_resource, server.call = resource, call
                case = movement.KeyboardMovementTest()
                with patch.dict(movement.os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                        patch.object(movement, "start_server", return_value=server), \
                        patch.object(Path, "is_file", return_value=True):
                    if lost:
                        with self.assertRaisesRegex(AssertionError, "Held key lost on action tick 3"):
                            case.test_press_hold_and_release_control_player_movement()
                    else:
                        case.test_press_hold_and_release_control_player_movement()
                        self.assertEqual(state["ticks"], 8)

    def test_stop_classification_never_skips_a_partial_or_ignored_stop(self):
        for mode in ("active", "natural", "lost", "ignored"):
            with self.subTest(mode=mode):
                state = {"ticks": 0, "running": False}
                server = SimpleNamespace(artifacts=Path("unused"), record=Mock(), process=Mock())
                server.process.wait.return_value = 0

                def cli(*args, **kwargs):
                    if args[1] == "create":
                        return {"id": "owned"}
                    if args[1] == "inspect":
                        return {"state": "SessionReady", "error": None}
                    if args[1] == "submit":
                        state.update(requested=json.loads(args[-1])["arguments"]["ticks"],
                                     base=state["ticks"], running=True)
                        return {"kind": "pending"}
                    return {"state": "Stopping"}

                def resource(_, path):
                    if state["running"]:
                        state["ticks"] += 2
                    return {"ticks": state["ticks"]} if path.endswith("SimulationTicks") else {"elapsed": state["ticks"]}

                def call(*args):
                    state["running"] = False
                    if mode == "natural":
                        state["ticks"] = state["base"] + state["requested"]
                    return {"was_running": mode in ("active", "ignored")}

                def result(*args):
                    natural = mode in ("natural", "ignored")
                    return {"kind": "completed", "output": {"requested_ticks": state["requested"],
                        "executed_ticks": state["requested"] if natural else state["ticks"] - state["base"],
                        "outcome": "completed" if natural else "stopped"}}

                server.cli, server.read_resource, server.call, server.result = cli, resource, call, result
                case = stop_warp.StopWarpTest()
                with patch.dict(stop_warp.os.environ, {"WOODPECKER_TEST_RENDERED": "1"}), \
                        patch.object(stop_warp, "start_server", return_value=server), \
                        patch.object(Path, "is_file", return_value=True), patch.object(stop_warp.time, "sleep"):
                    if mode == "active":
                        case.test_stop_finishes_active_warps_and_freezes_simulation()
                    else:
                        expected = unittest.SkipTest if mode == "natural" else AssertionError
                        with self.assertRaises(expected):
                            case.test_stop_finishes_active_warps_and_freezes_simulation()

    def test_gh_gate_is_complete_before_atomic_publication(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "mode.json").write_text('{"mode":"hold"}')
            (root / "release").touch()
            original = Path.replace
            publications = []

            def publish(source, destination):
                self.assertFalse(destination.exists())
                self.assertEqual(json.loads(source.read_text()), {"pid": gh.os.getpid()})
                publications.append(destination)
                return original(source, destination)

            arguments = ["gh", "api", "repos/{owner}/{repo}/issues?state=all&per_page=100",
                         "--method", "GET", "--paginate"]
            with patch.dict(gh.os.environ, {"WOODPECKER_TEST_GH_ROOT": directory}), \
                    patch.object(gh.sys, "argv", arguments), patch.object(Path, "replace", publish), \
                    contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(gh.main(), 0)
            self.assertEqual(publications, [root / "entered.json"])
            self.assertEqual(list(root.glob("*.next")), [])

    def test_live_ownership_registers_pid_and_separate_group(self):
        with tempfile.TemporaryDirectory() as directory:
            (Path(directory) / "process-probe.json").write_text('{"pid":234}')
            server = SimpleNamespace(process=Mock(pid=123), owned_processes=[], record=Mock())
            server.process.poll.return_value = None
            result = SimpleNamespace(returncode=0, stdout="123 test_app --woodpecker-diagnostics")
            with patch("tests._support.subprocess.run", return_value=result), \
                    patch("tests._support.os.getpgid", side_effect=[234, 123]), \
                    patch("tests._support.os.getpgrp", return_value=99):
                self.assertEqual(owned_process(server, directory), {"pid": 234, "group": 234})
            self.assertEqual(server.owned_processes, [{"pid": 234, "group": 234}])

    def test_foreign_pid_is_not_adopted(self):
        with tempfile.TemporaryDirectory() as directory:
            (Path(directory) / "process-probe.json").write_text('{"pid":234}')
            server = SimpleNamespace(process=Mock(pid=123), owned_processes=[], record=Mock())
            server.process.poll.return_value = None
            results = [SimpleNamespace(returncode=0, stdout="1 test_app"),
                       SimpleNamespace(returncode=1, stdout="")]
            with patch("tests._support.subprocess.run", side_effect=results), \
                    patch("tests._support.os.getpgid") as group:
                with self.assertRaisesRegex(AssertionError, "not a descendant"):
                    owned_process(server, directory)
                group.assert_not_called()
            self.assertEqual(server.owned_processes, [])

    def test_group_survivor_is_not_hidden_by_pid_disappearance(self):
        with patch("tests._support.os.kill", side_effect=ProcessLookupError) as pid, \
                patch("tests._support.os.killpg") as group:
            with self.assertRaisesRegex(AssertionError, "still present"):
                wait_process_gone({"pid": 234, "group": 234}, timeout=0)
            pid.assert_called_once_with(234, 0)
            group.assert_called_once_with(234, 0)

    def test_gone_process_and_group_need_no_cleanup_signal(self):
        with patch("tests._support.os.kill", side_effect=ProcessLookupError) as pid, \
                patch("tests._support.os.killpg", side_effect=ProcessLookupError) as group:
            wait_process_gone({"pid": 234, "group": 234}, timeout=0)
            pid.assert_called_once_with(234, 0)
            group.assert_called_once_with(234, 0)

    def test_final_file_audit_rejects_extra_report_and_live_server(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            reports = root / "reports"
            reports.mkdir()
            (reports / "one.md").write_text("# diagnostic\n\n<!-- bug_hunter-signature: owned -->\n")
            event = {"kind": "report", "report": {"signature": {"value": "owned"}},
                     "result": {"status": "submitted", "outcome": {"kind": "created",
                        "reference": {"kind": "file", "reference": {"path": "reports/one.md"}}}}}
            server = SimpleNamespace(process=Mock(), record=Mock())
            server.process.poll.return_value = 0
            audit_local_files(server, root, [event])
            (reports / "late.md").write_text("unexpected late report")
            with self.assertRaisesRegex(AssertionError, "Final report files"):
                audit_local_files(server, root, [event])
            server.process.poll.return_value = None
            with self.assertRaisesRegex(AssertionError, "reaped server"):
                audit_local_files(server, root, [event])

    def test_budget_headroom_changes_neither_status_nor_process(self):
        server = SimpleNamespace(process=Mock(), expected_exit_code=0, record=Mock())
        with patch("tests.shutdown.time.monotonic", return_value=10):
            require_observation_budget(self, server, (1, 2), 20, provider={})
        self.assertEqual(server.expected_exit_code, 0)
        server.process.wait.assert_not_called()

    def test_exhausted_budget_only_skips_after_verified_deadline_cleanup(self):
        for now, code, survivor in ((25, 1, False), (21.5, 1, False), (25, 2, False), (25, 1, True)):
            with self.subTest(now=now, code=code, survivor=survivor), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "server.stderr.log").write_text("shutdown_incomplete")
                process = {"pid": 234, "group": 234}
                provider = {"pid": 345, "group": 345}
                server = SimpleNamespace(process=Mock(), expected_exit_code=0, record=Mock(),
                                         artifacts=root, owned_processes=[process])
                server.process.wait.return_value = code
                with patch("tests.shutdown.time.monotonic", return_value=now), \
                        patch("tests.shutdown.wait_process_gone") as gone:
                    if survivor:
                        gone.side_effect = AssertionError("survivor")
                    expected = unittest.SkipTest if code == 1 and not survivor else AssertionError
                    with self.assertRaises(expected):
                        require_observation_budget(self, server, (1, 2), 20, provider=provider)
                    self.assertEqual(server.expected_exit_code, 1 if expected is unittest.SkipTest else 0)
                    if expected is unittest.SkipTest:
                        self.assertEqual([call.args[0] for call in gone.call_args_list], [process, provider])
