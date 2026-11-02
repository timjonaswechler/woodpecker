"""One explicitly prepared application for compatible sequential cases; never lifecycle cases."""
import contextlib
import os
import time
import unittest

from tests._support import Server, app_binary, load_config, owned_process, start_server, write_config
from tests.pointer import MOUSE, TICKS, pointer_states

# Membership is explicit: only cases that accept the same engine/configuration and reset contract.
MODULES = frozenset({"tests.keyboard.movement", "tests.keyboard.single_action",
    "tests.keyboard.invalid_transitions", "tests.pointer.buttons", "tests.pointer.invalid_input",
    "tests.pointer.relative_motion", "tests.pointer.scroll", "tests.pointer.ui_click",
    "tests.text.unicode_input", "tests.text.input_limits"})
FIXTURE = "test_app::diagnostics::fixture::TestFixture"
KEYS = "bevy_input::button_input::ButtonInput<bevy_input::keyboard::KeyCode>"
BASELINE = ("test_app::player::Player", "test_app::player::camera::ViewState",
    "test_app::app::settings::Settings", "test_app::app::flow::GameFlow", "test_app::input::PointerCapture")


class _Owner(unittest.TestCase):
    def __init__(self):
        super().__init__()
        self.stack = contextlib.ExitStack()

    def addCleanup(self, function, *args, **kwargs):
        self.stack.callback(function, *args, **kwargs)


class SharedGame:
    def __init__(self):
        self.owner = _Owner()
        self.server = None
        self.session_id = None
        self.identity = None
        self.baseline = None
        self.failed = False
        self.cases = []

    def _start(self):
        if os.environ.get("WOODPECKER_TEST_RENDERED") != "1":
            raise RuntimeError("Shared game needs scoped graphics approval: WOODPECKER_TEST_RENDERED=1")
        self.owner.assertTrue(app_binary().is_file(), "Prepare the automation app first")
        self.server = start_server(self.owner, "shared-game", activity_bytes=16 * 1024 * 1024)
        config = load_config()
        config["launch"]["executable"] = str(app_binary().resolve())
        config["launch"]["arguments"] += ["--woodpecker-diagnostics", "--woodpecker-process-probe", "--woodpecker-test-fixture"]
        # These cases test inputs/gameplay, not cold GPU startup. The dedicated
        # engine-startup case remains isolated and owns the EngineReady contract.
        config["startup"] = {"kind": "manual"}
        path = self.server.artifacts / "engine.toml"
        write_config(path, config)
        self.session_id = self.server.cli("session", "create", "--config", str(path))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = self.server.cli("session", "inspect", self.session_id)
            if detail["state"] == "SessionReady":
                break
            self.owner.assertIn(detail["state"], ("Starting", "SessionReady"), detail)
            self.owner.assertIsNone(detail["error"], detail)
            if time.monotonic() >= deadline:
                raise TimeoutError("Shared game SessionReady deadline")
            time.sleep(0.05)
        self.identity = owned_process(self.server, detail["artifact_dir"])
        deadline = time.monotonic() + 10
        for additional in range(121):
            load = self.server.read_resource(self.session_id, "test_app::level::LevelLoad")
            if load == "Ready":
                break
            if load != "Loading" or additional == 120 or time.monotonic() >= deadline:
                raise RuntimeError(f"Shared game not ready: {load}")
            self.server.call(self.session_id, {"command": "tick.warp.start", "arguments": {"ticks": 1}})
        self.server.record(["preparation", "shared-game-ready"], app_pid=self.identity["pid"],
                           additional_ticks=additional, session_id=self.session_id)

    def acquire(self, test, label):
        if self.failed:
            raise RuntimeError("Shared game is contaminated; refusing another case")
        try:
            if self.server is None:
                self._start()
            directory = self.server.artifacts / f"case-{len(self.cases):02}-{label}"
            directory.mkdir()
            server = Server(self.server.process, directory)
            server.address = self.server.address
            server.activity = self.server.activity
            server.ready = self.server.ready
            before = server.read_resource(self.session_id, FIXTURE)
            server.record(["preparation", "reset-start"], generation=before["generation"], app_pid=self.identity["pid"])
            server.call(self.session_id, {"command": "input.keyboard.press", "arguments": {"key": "f9"}})
            # Reset executes at the end of its tick; two ordinary ticks settle UI/physics.
            result = server.call(self.session_id, {"command": "tick.warp.start", "arguments": {"ticks": 3}})
            test.assertEqual(result, {"requested_ticks": 3, "executed_ticks": 3, "outcome": "completed"})
            after = server.read_resource(self.session_id, FIXTURE)
            test.assertEqual(after["generation"], before["generation"] + 1)
            test.assertEqual(after["pid"], self.identity["pid"])
            os.kill(self.identity["pid"], 0)
            empty = {"pressed": [], "just_pressed": [], "just_released": []}
            test.assertEqual(server.read_resource(self.session_id, KEYS), empty)
            test.assertEqual(server.read_resource(self.session_id, MOUSE), empty)
            test.assertEqual(server.read_resource(self.session_id, "bevy_input::mouse::AccumulatedMouseMotion"), {"delta": [0.0, 0.0]})
            test.assertEqual(server.read_resource(self.session_id, "bevy_input::mouse::AccumulatedMouseScroll"), {"unit": "Line", "delta": [0.0, 0.0]})
            pointers = pointer_states(server, self.session_id)
            test.assertTrue(pointers)
            test.assertTrue(all(pointer["location"] is None and not any(pointer["buttons"].values()) for pointer in pointers))
            baseline = {name: server.read_resource(self.session_id, name) for name in BASELINE}
            baseline["cameras"] = cameras(server, self.session_id)
            if self.baseline is None:
                self.baseline = baseline
            else:
                test.assertEqual(baseline, self.baseline, "Shared app reset did not restore its original baseline")
            row = dict(test=test.id(), app_pid=self.identity["pid"], server_pid=self.server.process.pid,
                       session_id=self.session_id, generation=after["generation"], artifacts=str(directory))
            self.cases.append(row)
            test.shared_app_observation = row
            test.artifact_dirs = [str(directory), str(self.server.artifacts)]
            server.record(["preparation", "reset-complete"], **row)
            return server, self.session_id
        except BaseException:
            self.failed = True
            raise

    def close(self):
        self.owner.stack.close()


def cameras(server, session_id):
    """Observe actual camera poses/projections, not only the logical view mode."""
    return server.call(session_id, {"command": "inspect.query", "arguments": {
        "source": "entities", "entity": None, "with": ["bevy_camera::camera::Camera"], "without": [],
        "projection": {"kind": "components", "selection": {"kind": "listed", "type_paths": [
            "bevy_transform::components::transform::Transform", "bevy_camera::projection::Projection"]}}}})


def game(test, label):
    """Same case interface for a single standalone test and a runner-owned group."""
    shared = getattr(test, "shared_game", None)
    if shared is None:
        shared = SharedGame()
        test.addCleanup(shared.close)
    return shared.acquire(test, label)
