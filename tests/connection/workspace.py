"""Two real Signalstation sessions persist settings through their normal UI."""
import json
import sys

from tests.catalog import scenario
from tests.session.application import ROOT


def settings_path(driver):
    workspace = driver.session.artifacts / "workspace"
    assert workspace.parent.name == driver.session.id
    assert all((workspace / name).is_dir() for name in ("config", "data", "tmp"))
    return workspace / "config/settings.json"


def tick(driver):
    driver.command("tick.warp.start", {"ticks": 1})


def rename_player(driver, name):
    # The player-name field is the first tab stop in the game's existing HUD.
    driver.command("input.keyboard.press", {"key": "tab"})
    tick(driver)
    driver.command("input.keyboard.release", {"key": "tab"})
    tick(driver)
    modifier = "super_left" if sys.platform == "darwin" else "control_left"
    for key in (modifier, "a"):
        driver.command("input.keyboard.press", {"key": key})
    tick(driver)
    for key in ("a", modifier):
        driver.command("input.keyboard.release", {"key": key})
    tick(driver)
    driver.command("input.text.input", {"text": name})
    tick(driver)
    # Bevy applies queued edits after Update; the game's autosave sees them next tick.
    tick(driver)


@scenario("connection", "session_settings_are_isolated", capabilities=("keyboard", "text"),
          commands=("input.keyboard.press", "input.keyboard.release", "input.text.input", "tick.warp.start"))
def session_settings_are_isolated(ctx):
    source = ((ctx.app.path.parent / ctx.app.data["settings_seed"]).resolve()
              if "settings_seed" in ctx.app.data else ROOT / "test_app/settings.json")
    baseline = source.read_bytes() if source.exists() else None
    with ctx.session() as first, ctx.session() as second:
        a, b = settings_path(first), settings_path(second)
        assert a != b
        initial = a.read_bytes()
        assert initial == b.read_bytes(), "sessions did not receive the same starting settings"
        if baseline is not None:
            assert initial == baseline, "the startup copy changed the source settings"
        rename_player(first, "Session A")
        assert json.loads(a.read_text())["player_name"] == "Session A"
        assert b.read_bytes() == initial, "saving A modified B"
        saved_a = a.read_bytes()
        rename_player(second, "Session B")
        assert json.loads(b.read_text())["player_name"] == "Session B"
        assert a.read_bytes() == saved_a, "saving B modified A"
        assert (source.read_bytes() if source.exists() else None) == baseline
        assert not a.with_suffix(".json.tmp").exists()
        assert not b.with_suffix(".json.tmp").exists()
    assert a.read_bytes() == saved_a, "session evidence disappeared after stop"
    with ctx.session() as fresh:
        assert settings_path(fresh).read_bytes() == initial, "a later session inherited edited settings"
