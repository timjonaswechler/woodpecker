"""TOML, version and file errors are rejected locally before creating a session."""
import json
from pathlib import Path
import shlex
import tempfile
import unittest

from tests._support import ROOT, load_config, start_server, write_config


class ConfigErrorsTest(unittest.TestCase):
    def test_file_and_toml_errors_never_create_a_session_or_invoke_cargo(self):
        runs = ROOT / "target/test-runs"
        runs.mkdir(parents=True, exist_ok=True)
        artifacts = Path(tempfile.mkdtemp(prefix="cli-config-errors-", dir=runs))
        self.artifact_dirs = [str(artifacts)]
        print(f"\nArtefakte: {artifacts}", flush=True)
        marker = artifacts / "unexpected-cargo.txt"
        tripwire = artifacts / "cargo-tripwire"
        tripwire.write_text("#!/bin/sh\nprintf 'unexpected Cargo invocation\\n' >> " +
                            shlex.quote(str(marker)) + "\nexit 86\n")
        tripwire.chmod(0o700)
        # Child-only guard: a parser regression can never start the real game.
        server = start_server(self, "cli-config-server", env={"CARGO": str(tripwire)})
        self.assertEqual(server.cli("session", "ls"), [])

        directory = artifacts / "directory.toml"
        directory.mkdir()
        malformed = artifacts / "malformed.toml"
        malformed.write_text("version = 1\n[launch\n")
        invalid_utf8 = artifacts / "invalid-utf8.toml"
        invalid_utf8.write_bytes(b"version = 1\n# \xff\n")
        cases = [
            ("missing", artifacts / "missing.toml", None),
            ("directory", directory, None),
            ("malformed", malformed, None),
            ("invalid-utf8", invalid_utf8, "UTF-8"),
        ]
        for label, key, value, diagnostic in (
            ("unknown-top", "unexpected", True, "unknown field"),
            ("unknown-launch", None, True, "unknown field"),
            ("future-version", "version", 2, "unsupported configuration version"),
            ("wrong-version-type", "version", "1", "invalid type"),
        ):
            configuration = load_config()
            if key is None:
                configuration["launch"]["unexpected"] = value
            else:
                configuration[key] = value
            path = artifacts / f"{label}.toml"
            write_config(path, configuration)
            cases.append((label, path, diagnostic))

        observations = []
        for label, path, diagnostic in cases:
            result = server.run_cli("session", "create", "--config", str(path))
            self.assertEqual(result.returncode, 1, (label, result.stdout, result.stderr))
            self.assertEqual(result.stdout, "", label)
            error = json.loads(result.stderr)
            self.assertEqual(set(error), {"error"}, label)
            self.assertIsInstance(error["error"], str, label)
            self.assertTrue(error["error"].strip(), label)
            if diagnostic:
                self.assertIn(diagnostic.lower(), error["error"].lower(), label)
            self.assertEqual(server.cli("session", "ls"), [], label)
            self.assertFalse(marker.exists(), "Config rejection must precede Cargo/launch")
            sessions = server.artifacts / "sessions"
            self.assertEqual(list(sessions.iterdir()) if sessions.exists() else [], [], label)
            observations.append({"case": label, "diagnostic": error["error"]})
        (artifacts / "observations.json").write_text(json.dumps(observations, indent=2) + "\n")
