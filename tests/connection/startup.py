"""Technical startup uses Signalstation's existing loading systems and real renderer."""
import time

from tests.catalog import scenario
from tests.commands._png import rgb_pixels


def transforms(driver):
    return driver.command("inspect.query", {
        "source": "entities", "entity": None,
        "with": ["bevy_transform::components::transform::Transform"], "without": [],
        "projection": {"kind": "components", "selection": {"kind": "listed",
            "type_paths": ["bevy_transform::components::transform::Transform"]}},
    })


@scenario("connection", "engine_startup_renders_and_freezes", capabilities=("screenshot",),
          commands=("inspect.query", "screenshot.capture", "tick.warp.start"))
def engine_startup_renders_and_freezes(ctx):
    with ctx.session() as driver:
        assert driver.session.detail()["state"] == "EngineReady"
        if ctx.interface == "repl":
            assert f"connected {driver.session.id} EngineReady" in driver.lines
        events = driver.session.poll()
        states = [event["state"] for event in events if event["kind"] == "lifecycle"]
        assert states.index("SessionReady") < states.index("EngineReady"), states
        ready = [event["event"] for event in events
                 if event["kind"] == "event" and event["event"]["kind"] == "engine_ready"]
        assert len(ready) == 1 and ready[0]["ticks"] >= 2, ready
        before = transforms(driver)
        assert before["items"], before
        time.sleep(0.1)
        output = driver.command("screenshot.capture", {"path": "startup.png"})
        width, height, pixels = rgb_pixels(driver.session.artifacts / "startup.png")
        assert (width, height) == (output["width"], output["height"])
        assert width >= 960 and height >= 640
        assert any(any(row) for row in pixels), "EngineReady produced an empty black image"
        assert transforms(driver) == before, "startup did not pause application updates"
        result = driver.command("tick.warp.start", {"ticks": 1})
        assert result["executed_ticks"] == 1, result
