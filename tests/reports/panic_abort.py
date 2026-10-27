"""A separate real panic=abort app must emit its synchronous panic marker before death."""
import json
import os
import signal
import subprocess
import time
import unittest

from tests._builds import ABORT_TARGET
from tests._support import ROOT, load_config, start_server, write_config
from tests.reports import audit_local_files, failures, observe, reports


class PanicAbortTest(unittest.TestCase):
    def _cleanup_build(self, process):
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
            self.fail("Abort build needed forced cleanup")

    def test_abort_build_reports_panic_once_and_not_process_exit(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Build a separate panic=abort automation app, then one manual window, "
                         "F11 and one intentional aborting tick. Server declared exit 1; local reports. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        server = start_server(self, "panic-abort-build")
        target = ABORT_TARGET  # Reuse Cargo artifacts; per-run logs stay under server.artifacts.
        environment = {**os.environ, "CARGO_TARGET_DIR": str(target), "CARGO_PROFILE_DEV_PANIC": "abort"}
        build_log = self.enterContext((server.artifacts / "abort-build.log").open("wb"))
        command = ["cargo", "build", "--locked", "--manifest-path", str(ROOT / "test_app/Cargo.toml"),
                   "--features", "automation", "--bin", "test_app"]
        build = subprocess.Popen(command, cwd=ROOT, stdout=build_log, stderr=subprocess.STDOUT,
                                 env=environment, start_new_session=True)
        self.addCleanup(self._cleanup_build, build)
        self.assertEqual(build.wait(timeout=1800), 0)
        self.assertTrue((target / "debug/test_app").is_file())
        (server.artifacts / "abort-build.json").write_text(json.dumps({
            "command": command, "CARGO_TARGET_DIR": str(target), "CARGO_PROFILE_DEV_PANIC": "abort"}) + "\n")
        # The live server is replaced only before any session exists, with the same build environment.
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 0)
        server = start_server(self, "panic-abort-run", env={
            "CARGO_TARGET_DIR": str(target), "CARGO_PROFILE_DEV_PANIC": "abort"}, expected_exit_code=1)
        configuration = load_config()
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics"]
        path = server.artifacts / "config.toml"
        write_config(path, configuration)
        session_id = server.cli("session", "create", "--config", str(path))["id"]
        deadline = time.monotonic() + 30
        while True:
            detail = server.cli("session", "inspect", session_id)
            self.assertIsNone(detail["error"], detail)
            if detail["state"] == "SessionReady":
                break
            if time.monotonic() >= deadline:
                self.fail("Session readiness deadline")
            time.sleep(0.03)
        self.assertIsNone(server.call(session_id, {"command": "input.keyboard.press", "arguments": {"key": "f11"}}))
        pending = server.cli("session", "submit", session_id, "--command",
                             '{"command":"tick.warp.start","arguments":{"ticks":1}}')
        self.assertEqual(server.result(session_id, pending)["kind"], "failed")
        events = observe(server, session_id, lambda e: len(reports(e)) >= 1 and
                         any(v["kind"] == "lifecycle" and v["state"] == "Failed" for v in e))
        self.assertEqual(len(failures(events)), 1, events)
        self.assertEqual(failures(events)[0]["origin"]["kind"], "panic")
        self.assertEqual(failures(events)[0]["message"], "Signalstation diagnostic panic")
        self.assertEqual(len(reports(events)), 1, events)
        self.assertEqual(reports(events)[0]["result"]["status"], "submitted")
        ended = [event["event"] for event in events
                 if event["kind"] == "event" and event["event"]["kind"] == "ended"]
        self.assertEqual(len(ended), 1, ended)
        reason = ended[0]["reason"]
        self.assertEqual(reason["kind"], "process_exit", reason)
        # Unix ExitStatus identifies the actual abort, not just the requested build profile.
        self.assertRegex(reason["status"], rf"^signal: {int(signal.SIGABRT)}(?:\D|$)")
        server.record(["observation", "panic-abort"], end_reason=reason)
        server.cli("server", "stop")
        self.assertEqual(server.process.wait(timeout=5), 1)
        audit_local_files(server, detail["artifact_dir"], events)
