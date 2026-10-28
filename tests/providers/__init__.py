"""Artifact-only external gh replacement; never invokes the real gh or a network."""
from pathlib import Path
import os
import shlex
import sys

from tests._support import ROOT


def gh_fixture(directory):
    directory = Path(directory)
    directory.mkdir(parents=True)
    executable = directory / "gh"
    executable.write_text("#!/bin/sh\nexec " + shlex.quote(sys.executable) + " " +
                          shlex.quote(str(ROOT / "tests/faults/gh.py")) + ' "$@"\n')
    executable.chmod(0o700)
    return {"PATH": str(directory) + os.pathsep + os.environ.get("PATH", ""),
            "WOODPECKER_TEST_GH_ROOT": str(directory)}
