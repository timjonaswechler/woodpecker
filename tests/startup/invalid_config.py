"""Invalid startup/tick values are rejected before a session or cargo launch exists."""
import json
from pathlib import Path
import shlex
import tempfile
import tomllib
import unittest

from tests._support import ROOT, load_config, start_server, write_config


class InvalidStartupTickConfigTest(unittest.TestCase):
    def test_invalid_startup_and_tick_values_are_rejected_before_launch(self):
        # Keep the real launch configuration valid. An external cargo tripwire
        # prevents a game window even if a regression wrongly accepts a variant.
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        guard_dir = Path(tempfile.mkdtemp(prefix="invalid-startup-tick-guard-", dir=runs))
        attempted = guard_dir / "unexpected-cargo-invocation.txt"
        cargo = guard_dir / "cargo"
        cargo.write_text(
            "#!/bin/sh\n"
            f"printf '%s\\n' 'unexpected cargo invocation' >> {shlex.quote(str(attempted))}\n"
            "printf '%s\\n' 'Test launch barrier: cargo must not run' >&2\n"
            "exit 86\n",
            encoding="utf-8",
        )
        cargo.chmod(0o755)
        self.artifact_dirs = [str(guard_dir)]
        server = start_server(self, "invalid-startup-tick-config", env={"CARGO": str(cargo)})
        server.record(["process", "environment"], overrides={"CARGO": str(cargo)},
                      launch_barrier=str(attempted))
        source = load_config()
        variants = []
        for label, timeout, ticks in (
            ("startup-timeout-zero", 0, 2),
            ("startup-ticks-zero", 10, 0),
        ):
            configuration = load_config()
            configuration["startup"] = {"kind": "engine", "timeout_seconds": timeout, "max_ticks": ticks}
            variants.append((label, configuration, True))
        for value in ("0", "-1", "nan", "inf", "-inf"):
            configuration = load_config()
            configuration["tick"]["pace"] = {
                "kind": "ticks_per_second", "target": tomllib.loads(f"value = {value}")["value"],
            }
            variants.append((f"pace-{value}", configuration, value not in ("nan", "inf", "-inf")))
        for value in ("0", "-0.01", "1e-12", "1e19", "1e30", "nan", "inf", "-inf"):
            configuration = load_config()
            configuration["tick"]["clock"]["seconds_per_tick"] = tomllib.loads(f"value = {value}")["value"]
            variants.append((f"clock-{value}", configuration, value not in ("nan", "inf", "-inf")))

        self.assertEqual(server.cli("session", "ls"), [])
        for label, invalid, typed_error in variants:
            # Valid TOML throughout; no repetition of the empty-package case.
            self.assertEqual(
                invalid["launch"], source["launch"], label,
            )
            config = server.artifacts / f"{label}.toml"
            write_config(config, invalid)
            server.record(["variant", label], config=str(config))
            result = server.run_cli("session", "create", "--config", str(config))
            self.assertEqual(result.returncode, 1, (label, result.stdout, result.stderr))
            self.assertEqual(result.stdout, "", label)
            error = json.loads(result.stderr)["error"]
            self.assertIsInstance(error, str, label)
            self.assertTrue(error, label)
            if typed_error:
                self.assertIn("invalid_config", error, label)
            # Non-finite TOML floats may be refused at the JSON transport boundary;
            # they must never become accepted work or reach a child launch.
            self.assertEqual(server.cli("session", "ls"), [], label)
            self.assertFalse(attempted.exists(), f"{label}: cargo was invoked; see {guard_dir}")

        self.assertEqual(server.cli("server", "stop")["state"], "Stopping")
        self.assertEqual(server.process.wait(timeout=5), 0)
        self.assertFalse(attempted.exists(), f"Late cargo invocation; see {guard_dir}")
