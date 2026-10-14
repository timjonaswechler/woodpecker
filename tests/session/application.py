"""Configuration of the single application used by every behavioral contract."""
import json
from pathlib import Path
import tomllib

from tests.catalog import MODES

ROOT = Path(__file__).resolve().parents[2]


class Unavailable(RuntimeError):
    """A required application or capability has not been supplied."""


class Application:
    def __init__(self, path):
        self.path = Path(path).resolve()
        if not self.path.is_file():
            raise Unavailable(f"Test-App-Konfiguration fehlt: {self.path}")
        self.data = json.loads(self.path.read_text())
        if self.data.get("version") != 1:
            raise ValueError("Test-App-Konfiguration benötigt version=1")
        self.type_path = self.data["state_type"]
        if not isinstance(self.type_path, str) or not self.type_path:
            raise ValueError("state_type muss ein vollständiger Reflection-Type-Path sein")
        self.modes = self.data["modes"]
        if not isinstance(self.modes, dict) or set(self.modes) - set(MODES):
            raise ValueError("modes unterstützt nur rendered")

    def mode(self, name):
        if name not in self.modes:
            raise Unavailable(f"Session-Modus {name} ist noch nicht angebunden")
        mode = self.modes[name]
        if not isinstance(mode.get("capabilities"), list):
            raise ValueError(f"{name}: capabilities muss eine Liste sein")
        config = (self.path.parent / mode["config"]).resolve()
        if not config.is_file():
            raise Unavailable(f"Launch-Konfiguration fehlt: {config}")
        return config, set(mode["capabilities"])

    def require(self, mode, capabilities):
        config, available = self.mode(mode)
        missing = set(capabilities) - available
        if missing:
            raise Unavailable(f"{mode}: App-Fähigkeiten fehlen: {', '.join(sorted(missing))}")
        return config

    def build_command(self, mode):
        config, _ = self.mode(mode)
        launch = tomllib.loads(config.read_text())["launch"]
        manifest = (ROOT / launch["manifest_path"]).resolve()
        if not manifest.is_file():
            raise Unavailable(f"Test-App-Manifest fehlt: {manifest}")
        target = launch["target"]
        if target["kind"] not in ("binary", "example"):
            raise ValueError(f"Unbekanntes Launch-Target: {target}")
        command = ["cargo", "build", "--locked", "--manifest-path", str(manifest),
                   "--package", launch["package"],
                   "--bin" if target["kind"] == "binary" else "--example", target["name"]]
        if launch.get("features"):
            command += ["--features", ",".join(launch["features"])]
        return command
