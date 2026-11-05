"""Standard Cargo commands and their artifacts; no test scenarios or application starts."""
import json
from pathlib import Path

from tests._support import ROOT


FEATURES = "cli,ui,screenshot,offscreen"
ABORT_TARGET = ROOT / "target/panic-abort"
CANONICAL = ["-p", "woodpecker", "-p", "test_app", "-p", "woodpecker_cli",
             "--features", "woodpecker/cli,woodpecker/ui,woodpecker/screenshot,woodpecker/offscreen,test_app/automation"]


def commands(selected, *, feature_matrix=False, headless=False):
    """Build selected cases' dependencies; a full feature audit is explicit."""
    selected = list(selected)
    if headless:
        if feature_matrix:
            raise ValueError("Headless L1 preparation is not a native feature-matrix audit")
        # Separate Cargo invocations: CANONICAL would enable test_app/native and
        # unify Winit/clipboard/X11 into the headless app dependency graph.
        return [("cli", ["cargo", "build", "--locked", "-p", "woodpecker_cli", "--bin", "woodpecker"]),
                ("app-headless", ["cargo", "build", "--locked", "-p", "test_app",
                                  "--no-default-features", "--features", "automation", "--bin", "test_app"])]
    python = [row for row in selected if row["kind"] in ("python", "control")]
    targets = list(dict.fromkeys(row["target"] for row in selected if row["kind"] == "rust"))
    internal = any(row["kind"] == "internal" for row in selected)
    engine_graph = internal or bool(targets) or any(row["graphics"] for row in selected)
    rows = []
    if feature_matrix:
        for features in (None, "ui", "screenshot", "ui,screenshot", "offscreen", "server", "client", "cli"):
            command = ["cargo", "check", "--locked", "--no-default-features", "--lib"]
            if features:
                command += ["--features", features]
            rows.append(("library-" + (features or "default"), command))
    # The SDK live driver does not use the CLI. The abort case builds its own app variant.
    if any(row["module"] != "tests.providers.github_live" for row in python):
        rows.append(("cli", ["cargo", "build", "--locked", *(CANONICAL if engine_graph else
            ["-p", "woodpecker_cli"]), "--bin", "woodpecker", *(["--bin", "test_app"] if engine_graph else [])]))
    if feature_matrix:
        rows.append(("app-normal", ["cargo", "build", "--locked", "--manifest-path", "test_app/Cargo.toml",
                                   "--bin", "test_app"]))
    # Automation last: normal and automation share the test_app output path.
    if any(row["graphics"] and row.get("module") != "tests.reports.panic_abort" for row in selected):
        rows.append(("app-automation", ["cargo", "build", "--locked", *CANONICAL, "--bin", "test_app"]))
    if any(row.get("module") == "tests.reports.panic_abort" for row in selected):
        rows.append(("app-abort", ["cargo", "build", "--locked", "-p", "woodpecker_abort_fixture", "--bin", "woodpecker-abort-fixture"]))
    if internal or targets:
        rows.append(("rust-tests", ["cargo", "test", "--locked", *CANONICAL,
                                    *(["--lib"] if internal else []),
                                    *[arg for target in targets for arg in ("--test", target)],
                                    "--no-run", "--message-format=json"]))
    if any(row["module"] == "tests.providers.github_live" for row in python):
        rows.append(("github-live-driver", ["cargo", "build", "--locked", "--manifest-path",
                                            "tests/providers/live_driver/Cargo.toml", "--bin", "github-live-driver"]))
    return rows


def validate_app_build(proof, *, headless=False):
    """A prepared app must prove automation with the requested native feature policy."""
    mode = "headless" if headless else "native"
    features = set(proof.get("features", [])) if isinstance(proof, dict) else set()
    valid_features = features == {"automation"} if headless else {"automation", "native"}.issubset(features)
    if not isinstance(proof, dict) or proof.get("mode") != mode or not valid_features:
        raise ValueError(f"Prepared app does not prove the requested {mode} variant; prepare that mode again")


def app_build(logs, *, headless=False):
    """Read actual Cargo target features, not a label inferred from requested flags."""
    name = "app-headless" if headless else "app-automation"
    for build, path in logs:
        if build != name:
            continue
        for line in Path(path).read_text().splitlines():
            try:
                item = json.loads(line)
            except ValueError:
                continue
            if (item.get("reason") == "compiler-artifact" and item.get("executable")
                    and item["target"]["name"] == "test_app" and not item["profile"]["test"]):
                proof = {"mode": "headless" if headless else "native", "features": sorted(item.get("features", []))}
                validate_app_build(proof, headless=headless)
                return proof
    raise ValueError(f"Missing Cargo feature proof for {name}")


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
