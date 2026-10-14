"""Real OS signals and shared deadlines; independent of the future Bevy app."""
import json
from pathlib import Path
import signal
import time

from tests.session.application import ROOT
from tests.session.runtime import Server, process_gone, until


def run(binary, directory):
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=False)
    for mode, interruption, expected, timeout in (
        ("normal", "http", 0, 10), ("delayed", "http", 0, 10),
        ("hang_shutdown", "term", 1, 0.3), ("hang_shutdown", "interrupt", 1, 30),
    ):
        config = directory / f"{mode}-{interruption}.toml"
        config.write_text(f'''version = 1
[launch]
manifest_path = {json.dumps(str(ROOT / 'tests/connection/protocol_faults/peer/Cargo.toml'))}
package = "process_fixture"
features = []
arguments = [{json.dumps(mode)}]
[launch.target]
kind = "binary"
name = "process_fixture"
[tick.pace]
kind = "as_fast_as_possible"
[report]
tracing_errors = false
output = "reports"
[report.provider]
kind = "local"
''')
        with Server(binary, directory / f"{mode}-{interruption}", timeout) as server:
            sessions = [server.cli("session", "create", "--config", str(config)) for _ in range(3)]

            def ready():
                details = [server.cli("session", "inspect", s["id"]) for s in sessions]
                assert all(d["state"] != "Failed" for d in details), details
                if mode == "delayed":
                    return all((Path(s["artifact_dir"]) / "pid").is_file() for s in sessions)
                return all(d["state"] == "SessionReady" for d in details)

            until(ready, "Shutdown-Prozesse wurden nicht gestartet", timeout=120)
            pids = [int((Path(s["artifact_dir"]) / "pid").read_text()) for s in sessions]
            started = time.monotonic()
            if interruption == "http":
                server.cli("server", "stop", allow_empty=True)
            elif interruption == "term":
                server.process.send_signal(signal.SIGTERM)
            else:
                server.process.send_signal(signal.SIGINT)
                until(lambda: all(server.cli("session", "inspect", s["id"])["state"] == "Stopping"
                                  for s in sessions), "Sessions gehen nicht in Stopping", timeout=5)
                server.process.send_signal(signal.SIGINT)
            assert server.process.wait(timeout=10) == expected
            assert time.monotonic() - started < 3, "Shutdown-Frist wurde pro Session statt gemeinsam angewendet"
            for pid in pids:
                until(lambda: process_gone(pid), f"Prozess {pid} überlebte Server-Shutdown", timeout=3)
