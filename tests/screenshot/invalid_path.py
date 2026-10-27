"""Lexical and escaping-symlink paths cannot overwrite test-owned targets outside the root."""
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, load_config, start_server, write_config
from tests.screenshot import snapshot_tree

TICKS = "test_app::woodpecker::SimulationTicks"


class InvalidScreenshotPathTest(unittest.TestCase):
    def test_bad_paths_and_escaping_symlinks_leave_all_owned_targets_unchanged(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "One window, up to 3600 engine ticks/60 s and 120 readiness ticks/10 s, "
                         "then zero ticks/inputs and 12 expected invalid_screenshot_path rejections. "
                         "Only test-owned files and two escaping symlinks are prepared. "
                         "No foreign target, capture retry or OS input. Permission required: WOODPECKER_TEST_RENDERED=1")
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        server = start_server(self, "screenshot-invalid-path")
        config = server.artifacts / "engine.toml"
        configuration = load_config()
        configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
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
                self.fail("EngineReady deadline exceeded")
            time.sleep(0.05)
        startup_ticks = server.read_resource(session_id, TICKS)["ticks"]
        deadline = time.monotonic() + 10
        for additional in range(121):
            load = server.read_resource(session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            self.assertEqual(load, "Loading")
            if additional == 120 or time.monotonic() >= deadline:
                self.fail("Game readiness budget exceeded")
            self.assertEqual(server.call(session_id, {
                "command": "tick.warp.start", "arguments": {"ticks": 1},
            }), {"requested_ticks": 1, "executed_ticks": 1, "outcome": "completed"})
        server.record(["preparation", "game-ready"], startup_ticks=startup_ticks, additional_ticks=additional)
        root = Path(detail["artifact_dir"])
        self.assertTrue(root.resolve().is_relative_to((server.artifacts / "sessions").resolve()))
        self.assertFalse(root.is_symlink())
        shots = root / "screenshots"
        shots.mkdir()
        outside = server.artifacts / "outside-session-root"
        outside.mkdir()
        outside_file = outside / "protected.png"
        outside_file.write_bytes(b"Test-owned outside-session file: must remain untouched.\n")
        (outside / "absolute.png").write_bytes(b"Owned absolute target.\n")
        (outside / "traversal.png").write_bytes(b"Owned traversal target.\n")
        (shots / "keep.png").write_bytes(b"Owned in-root sentinel.\n")
        root_keep = root / "keep-root.png"
        root_keep.write_bytes(b"Owned embedded-parent sentinel.\n")
        (shots / "escape-dir").symlink_to(os.path.relpath(outside, shots), target_is_directory=True)
        (shots / "escape-file.png").symlink_to(os.path.relpath(outside_file, shots))
        before_shots = snapshot_tree(shots)
        before_outside = snapshot_tree(outside)
        before_root_keep = root_keep.read_bytes()
        before_pngs = sorted(str(path.relative_to(root)) for path in root.rglob("*.png"))
        time_type = "bevy_time::time::Time<()>"
        before_ticks = server.read_resource(session_id, TICKS)
        before_time = server.read_resource(session_id, time_type)
        rejected = []
        for label, path in (
            ("empty", ""),
            ("absolute", str(outside / "absolute.png")),
            ("traversal", os.path.relpath(outside / "traversal.png", root)),
            ("embedded-parent", "screenshots/../keep-root.png"),
            ("dot", "screenshots/./dot.png"),
            ("empty-component", "screenshots//double.png"),
            ("backslash", "screenshots/back\\slash.png"),
            ("extension", "screenshots/wrong.jpg"),
            ("uppercase-extension", "screenshots/upper.PNG"),
            ("nul", "screenshots/invalid\0.png"),
            ("symlink-parent", "screenshots/escape-dir/protected.png"),
            ("symlink-file", "screenshots/escape-file.png"),
        ):
            pending = server.cli("session", "submit", session_id, "--command", json.dumps({
                "command": "screenshot.capture", "arguments": {"path": path},
            }))
            outcome = server.result(session_id, pending, timeout=45)
            self.assertEqual(outcome["kind"], "rejected", (label, outcome))
            self.assertEqual(outcome["error"]["code"], "invalid_screenshot_path", (label, outcome))
            self.assertEqual(snapshot_tree(shots), before_shots, label)
            self.assertEqual(snapshot_tree(outside), before_outside, label)
            self.assertEqual(root_keep.read_bytes(), before_root_keep, label)
            self.assertEqual(sorted(str(item.relative_to(root)) for item in root.rglob("*.png")),
                             before_pngs, label)
            self.assertEqual(server.read_resource(session_id, TICKS), before_ticks, label)
            self.assertEqual(server.read_resource(session_id, time_type), before_time, label)
            rejected.append({"variant": label, "path": path, "outcome": outcome})
        self.assertEqual(len(rejected), 12)
        self.assertEqual(server.cli("session", "inspect", session_id)["state"], "EngineReady")
        server.record(["observation", "screenshot-invalid-path"], rejections=rejected,
                      unchanged_in_root=before_shots, unchanged_outside=before_outside, action_ticks=0)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertEqual(snapshot_tree(shots), before_shots)
        self.assertEqual(snapshot_tree(outside), before_outside)
        self.assertEqual(root_keep.read_bytes(), before_root_keep)
