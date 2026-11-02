"""A separate real panic=abort app must emit its synchronous panic marker before death."""
import json
import os
import signal
from pathlib import Path
import time
import unittest

from tests._builds import ABORT_TARGET
from tests._support import load_config, start_server, write_app_config
from tests.reports import audit_local_files, failures, observe, reports


class PanicAbortTest(unittest.TestCase):
    def test_abort_build_reports_panic_once_and_not_process_exit(self):
        self.assertEqual(os.environ.get("WOODPECKER_TEST_RENDERED"), "1",
                         "Use the prepared panic=abort automation app, then one manual window, "
                         "F11 and one intentional aborting tick. Server declared exit 1; local reports. "
                         "Permission required: WOODPECKER_TEST_RENDERED=1")
        executable = Path(os.environ.get("WOODPECKER_TEST_ABORT_APP", ABORT_TARGET / "debug/test_app"))
        self.assertTrue(executable.is_file(), "Prepare panic-abort with acceptance --test tests.reports.panic_abort --build-only")
        server = start_server(self, "panic-abort-run", expected_exit_code=1)
        (server.artifacts / "abort-build.json").write_text(json.dumps({"prepared_executable": str(executable)}) + "\n")
        configuration = load_config()
        configuration["launch"]["executable"] = str(executable.resolve())
        configuration["launch"]["arguments"] += ["--woodpecker-diagnostics"]
        path = server.artifacts / "config.toml"
        write_app_config(path, configuration)
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
