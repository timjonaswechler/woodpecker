"""Standard Cargo commands and their artifacts; no test scenarios or application starts."""
import json
from pathlib import Path

from tests._support import ROOT


FEATURES = "cli,ui,screenshot"
ABORT_TARGET = ROOT / "target/panic-abort"


def commands(selected, *, feature_matrix=False):
    """Build selected cases' dependencies; a full feature audit is explicit."""
    selected = list(selected)
    python = [row for row in selected if row["kind"] in ("python", "control")]
    targets = [row["target"] for row in selected if row["kind"] == "rust"]
    internal = any(row["kind"] == "internal" for row in selected)
    rows = []
    if feature_matrix:
        for features in (None, "ui", "screenshot", "ui,screenshot", "server", "client", "cli"):
            command = ["cargo", "check", "--locked", "--no-default-features", "--lib"]
            if features:
                command += ["--features", features]
            rows.append(("library-" + (features or "default"), command))
    # The SDK live driver does not use the CLI. The abort case builds its own app variant.
    if any(row["module"] != "tests.providers.github_live" for row in python):
        rows.append(("cli", ["cargo", "build", "--locked", "--features", "cli", "--bin", "woodpecker"]))
    if feature_matrix:
        rows.append(("app-normal", ["cargo", "build", "--locked", "--manifest-path", "test_app/Cargo.toml",
                                   "--no-default-features", "--bin", "test_app"]))
    # Automation last: normal and automation share the test_app output path.
    if any(row["graphics"] and row.get("module") != "tests.reports.panic_abort" for row in selected):
        rows.append(("app-automation", ["cargo", "build", "--locked", "--manifest-path", "test_app/Cargo.toml",
                                       "--features", "automation", "--bin", "test_app"]))
    if internal or targets:
        rows.append(("rust-tests", ["cargo", "test", "--locked", "--features", FEATURES,
                                    *(["--lib"] if internal else []),
                                    *[arg for target in targets for arg in ("--test", target)],
                                    "--no-run", "--message-format=json"]))
    if any(row["module"] == "tests.providers.github_live" for row in python):
        rows.append(("github-live-driver", ["cargo", "build", "--locked", "--manifest-path",
                                            "tests/providers/live_driver/Cargo.toml", "--bin", "github-live-driver"]))
    return rows


def executables(log):
    found = {}
    for line in Path(log).read_text().splitlines():
        try:
            item = json.loads(line)
        except ValueError:
            continue
        if item.get("reason") == "compiler-artifact" and item.get("executable") and item["profile"]["test"]:
            found[(item["target"]["name"], tuple(item["target"]["kind"]))] = item["executable"]
    return found


def rust_source_tests(path):
    """Names only; no scenario reimplementation or Rust execution during --list."""
    import re
    return re.findall(r"#\[test\]\s*fn\s+(\w+)\s*\(", Path(path).read_text())
