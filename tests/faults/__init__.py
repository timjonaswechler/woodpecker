"""Only artifacts for external protocol peers and owned-process cleanup; no session starts."""
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import sys

from tests._support import ROOT


def protocol_fixture(directory):
    """Write a windowless sentinel package and its explicit external Cargo adapter."""
    directory.mkdir()
    manifest = directory / "Cargo.toml"
    manifest.write_text('[package]\nname = "woodpecker-protocol-fixture"\nversion = "0.0.0"\n'
                        'edition = "2024"\n[workspace]\n[[bin]]\nname = "protocol-peer"\npath = "main.rs"\n')
    (directory / "main.rs").write_text('fn main() { std::process::exit(86); }\n')
    real_cargo = shutil.which("cargo")
    if real_cargo is None:
        raise AssertionError("Real Cargo is required for sentinel metadata")
    adapter = directory / "cargo-peer"
    adapter.write_text("#!/bin/sh\nexec " + shlex.quote(sys.executable) + " " +
                       shlex.quote(str(ROOT / "tests/faults/protocol_peer.py")) + ' cargo "$@"\n')
    adapter.chmod(0o700)
    return manifest, {"CARGO": str(adapter), "WOODPECKER_TEST_REAL_CARGO": real_cargo,
                      "WOODPECKER_TEST_PEER_MANIFEST": str(manifest)}


def cleanup_peer(holder):
    """Fail on forced cleanup; signal a surviving PID only after verifying our peer."""
    path = holder.get("path")
    if path is None or not path.exists():
        return
    info = json.loads(path.read_text())
    pid = info["pid"]
    forced = False
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        pass
    else:
        command = subprocess.run(["ps", "-p", str(pid), "-o", "command="],
                                 capture_output=True, text=True, timeout=5).stdout
        protocol_peer = "tests/faults/protocol_peer.py peer" in command
        report_peer = (str(ROOT / "tests/faults/report_peer.py") in command
                       and info.get("mode", "").startswith("report-"))
        if not (protocol_peer or report_peer):
            raise AssertionError(f"PID reused; refusing to signal {pid}")
        os.killpg(pid, signal.SIGKILL)
        forced = True
    (Path(path).parent / "peer-cleanup.json").write_text(json.dumps({**info, "forced": forced}) + "\n")
    holder.clear()
    if forced:
        raise AssertionError("Server failed to reap its external fault peer")
