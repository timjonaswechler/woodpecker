"""Three separately grouped owned processes; ignore SIGINT, self-expire after 20s."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def main():
    root, level = Path(sys.argv[1]), int(sys.argv[2])
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    child = None
    if level < 2:
        child = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), str(root), str(level + 1)],
                                 stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                 stderr=subprocess.DEVNULL, start_new_session=True)
    pending = root / f"{level}.next"
    pending.write_text(json.dumps({"pid": os.getpid(), "group": os.getpgrp()}) + "\n")
    pending.replace(root / f"{level}.json")
    # A broken cleanup test cannot leave an unbounded process behind. Expiry is
    # not a successful cleanup proof: the test requires all groups gone earlier.
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        if child is not None:
            child.poll()
        time.sleep(0.01)


if __name__ == "__main__":
    main()
