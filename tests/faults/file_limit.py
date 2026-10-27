"""External RLIMIT_FSIZE launcher; only the owned server keeps the reduced soft limit."""
import json
import os
from pathlib import Path
import resource
import shlex
import shutil
import signal
import sys


def main():
    if sys.argv[1] == "restore-cargo":
        saved = json.loads(os.environ["WOODPECKER_TEST_SAVED_FILE_LIMIT"])
        resource.setrlimit(resource.RLIMIT_FSIZE, tuple(saved["limit"]))
        signal.signal(signal.SIGXFSZ, saved["signal"])
        cargo = saved["cargo"]
        os.environ["CARGO"] = cargo
        with (Path(saved["root"]) / "cargo-limit-restored.jsonl").open("a") as output:
            output.write(json.dumps({"pid": os.getpid(), "limit": resource.getrlimit(resource.RLIMIT_FSIZE),
                                     "arguments": sys.argv[2:]}) + "\n")
        os.execv(cargo, [cargo, *sys.argv[2:]])
    assert sys.argv[1] == "limit"
    limit = int(sys.argv[2])
    root = Path(sys.argv[3])
    command = sys.argv[4:]
    assert limit > 0 and root.is_dir() and command
    cargo = shutil.which(os.environ.get("CARGO", "cargo"))
    assert cargo, "Cargo executable must exist before imposing the server limit"
    saved = {"limit": resource.getrlimit(resource.RLIMIT_FSIZE),
             "signal": int(signal.getsignal(signal.SIGXFSZ)), "cargo": cargo, "root": str(root)}
    assert saved["limit"][0] == resource.RLIM_INFINITY or saved["limit"][0] >= limit
    os.environ["WOODPECKER_TEST_SAVED_FILE_LIMIT"] = json.dumps(saved)
    adapter = root / "restore-cargo"
    adapter.write_text("#!/bin/sh\nexec " + shlex.quote(sys.executable) + " " +
                       shlex.quote(str(Path(__file__).resolve())) + ' restore-cargo "$@"\n')
    adapter.chmod(0o700)
    os.environ["CARGO"] = str(adapter)
    signal.signal(signal.SIGXFSZ, signal.SIG_IGN)
    resource.setrlimit(resource.RLIMIT_FSIZE, (limit, saved["limit"][1]))
    (root / "server-file-limit.json").write_text(json.dumps({
        "server_pid": os.getpid(), "soft_limit": limit, "saved": saved,
    }, indent=2) + "\n")
    os.execv(command[0], command)


if __name__ == "__main__":
    main()
