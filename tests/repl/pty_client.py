"""Supervise one CLI under an owned controlling PTY; observe, never restore, termios."""
import fcntl
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import termios


def main():
    directory = Path(sys.argv[1])
    if not os.isatty(0) or not os.isatty(1) or os.ttyname(0) != os.ttyname(1):
        raise RuntimeError("The launcher requires one test-owned PTY for stdin/stdout")
    # The supervisor remains session leader while the CLI runs in its foreground group.
    # macOS revokes slave descriptors when that leader exits. Observe restoration
    # BEFORE then, otherwise a kernel reset could mask missing CLI restoration.
    fcntl.ioctl(0, termios.TIOCSCTTY, 0)
    (directory / "tty-initial.txt").write_text(repr(termios.tcgetattr(0)) + "\n")
    child = subprocess.Popen(sys.argv[2:], stdin=0, stdout=1, stderr=2)
    (directory / "pty-process.json").write_text(json.dumps({"supervisor_pid": os.getpid(), "cli_pid": child.pid}) + "\n")
    signal.signal(signal.SIGINT, lambda sig, frame: child.send_signal(sig))
    code = child.wait()
    (directory / "tty-after.txt").write_text(repr(termios.tcgetattr(0)) + "\n")
    raise SystemExit(code if code >= 0 else 128 - code)


if __name__ == "__main__":
    main()
