"""A real settings edit persists only in its session workspace, never shared files."""
import hashlib
import json
import os
from pathlib import Path
import time
import unittest

from tests._support import ROOT, WINDOW_POSITIONS, start_server, load_config, write_config


class FileIsolationTest(unittest.TestCase):
    def _fingerprint(self, path):
        """Read-only privacy check; never log or back up personal settings contents."""
        return hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else None

    def _assert_unchanged(self, path, expected):
        self.assertEqual(self._fingerprint(path), expected, f"Unexpected change to {path}")

    def test_editing_settings_writes_only_the_addressed_session_file(self):
        self.assertEqual(
            os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
            "This test opens two staggered Signalstation windows. Each permits 3600 "
            "startup ticks/60 seconds and 120 readiness ticks/ten seconds. A uses Tab "
            "press/release, text '!' and four action ticks; B gets one tick, no inputs. "
            "Personal settings are read only for a hash comparison, never changed. "
            "Permission for this run is required: WOODPECKER_TEST_RENDERED=1",
        )
        self.assertTrue((ROOT / "target/debug/test_app").is_file(), "Build the automation app first")
        personal = ROOT / "test_app/settings.json"
        seed_path = ROOT / "tests/settings.json"
        personal_before = self._fingerprint(personal)
        seed_before = self._fingerprint(seed_path)
        self.addCleanup(self._assert_unchanged, personal, personal_before)
        self.addCleanup(self._assert_unchanged, seed_path, seed_before)
        seed = json.loads(seed_path.read_text())
        self.assertNotIn("!", seed["player_name"])
        server = start_server(self, "file-isolation")
        sessions = []
        files = []
        roots = []
        settings_type = "test_app::app::settings::Settings"
        for label, (x, y) in zip(("A", "B"), WINDOW_POSITIONS):
            config = server.artifacts / f"session-{label}.toml"
            configuration = load_config()
            configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
            configuration["launch"]["arguments"][:0] = ["--window-position", f"{x},{y}"]
            write_config(config, configuration)
            session_id = server.cli("session", "create", "--config", str(config))["id"]
            sessions.append(session_id)
            deadline = time.monotonic() + 90
            while True:
                detail = server.cli("session", "inspect", session_id)
                self.assertIsNone(detail["error"], detail)
                if detail["state"] == "EngineReady":
                    break
                self.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
                if time.monotonic() >= deadline:
                    self.fail(f"{label}: EngineReady missing; see {server.artifacts}")
                time.sleep(0.05)
            deadline = time.monotonic() + 10
            for additional in range(121):
                load = server.read_resource(session_id, "test_app::level::LevelLoad")
                if load == "Ready":
                    break
                self.assertEqual(load, "Loading", load)
                if additional == 120 or time.monotonic() >= deadline:
                    self.fail(f"{label}: game readiness budget exceeded")
                server.call(session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
            server.record(["preparation", "game-ready"], label=label, additional_ticks=additional)
            root = Path(detail["artifact_dir"])
            self.assertTrue(root.is_absolute())
            self.assertTrue(root.resolve().is_relative_to((server.artifacts / "sessions").resolve()))
            path = root / "workspace/config/settings.json"
            self.assertTrue(path.is_file(), f"Do not edit until isolated storage exists: {path}")
            self.assertFalse(path.is_symlink())
            self.assertTrue(path.resolve().is_relative_to(root.resolve()), "Workspace must not escape its own root")
            self.assertEqual(path.read_bytes(), seed_path.read_bytes())
            self.assertEqual(server.read_resource(session_id, settings_type), seed)
            roots.append(root)
            files.append(path)
        session_a, session_b = sessions
        self.assertNotEqual(roots[0].resolve(), roots[1].resolve())
        self.assertFalse(os.path.samefile(files[0], files[1]))
        bytes_b = files[1].read_bytes()
        ticks_type = "test_app::woodpecker::SimulationTicks"
        before = {session_id: server.read_resource(session_id, ticks_type)["ticks"]
                  for session_id in sessions}

        # The first HUD Tab target is its real player-name input. Insertion
        # position is irrelevant here: text editing rules belong to 4.27/4.28.
        server.call(session_a, {"command": "input.keyboard.press", "arguments": {"key": "tab"}})
        server.call(session_a, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        server.call(session_a, {"command": "input.keyboard.release", "arguments": {"key": "tab"}})
        server.call(session_a, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        server.call(session_a, {"command": "input.text.input", "arguments": {"text": "!"}})
        # Two explicit ticks cover UI editing, app synchronization and persistence;
        # there is no retry loop that keeps ticking until the assertion passes.
        server.call(session_a, {"command": "tick.warp.start", "arguments": {"ticks": 2}})
        server.call(session_b, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        changed = server.read_resource(session_a, settings_type)
        self.assertEqual(changed["player_name"].count("!"), 1)
        self.assertEqual(changed["player_name"].replace("!", ""), seed["player_name"])
        self.assertEqual({key: value for key, value in changed.items() if key != "player_name"},
                         {key: value for key, value in seed.items() if key != "player_name"})
        self.assertEqual(json.loads(files[0].read_text()), changed)
        self.assertEqual(files[1].read_bytes(), bytes_b)
        self.assertEqual(server.read_resource(session_b, settings_type), seed)
        self.assertEqual(server.read_resource(session_a, ticks_type)["ticks"] - before[session_a], 4)
        self.assertEqual(server.read_resource(session_b, ticks_type)["ticks"] - before[session_b], 1)
        server.record(["observation", "file-isolation"], roots=list(map(str, roots)),
                      session_a_settings=changed, session_b_unchanged=True,
                      personal_settings_unchanged=self._fingerprint(personal) == personal_before)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertEqual(json.loads(files[0].read_text()), changed)
        self.assertEqual(files[1].read_bytes(), bytes_b)
