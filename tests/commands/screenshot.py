from tests.catalog import scenario
from tests.commands._png import rgb_pixels

COMMAND = "screenshot.capture"


@scenario("screenshot", "unavailable_without_renderer", commands=(COMMAND,), modes=("headless",))
def unavailable_without_renderer(ctx):
    with ctx.session() as driver:
        before = ctx.probe(driver)
        relative = "screenshots/unavailable.png"
        driver.command(COMMAND, {"path": relative}, expected_error="screenshot_unavailable")
        assert not (driver.session.artifacts / relative).exists()
        assert ctx.probe(driver) == before


@scenario("screenshot", "known_pixels_without_simulation_progress", commands=(COMMAND,),
          modes=("rendered",), capabilities=("state", "screenshot"))
def known_pixels_without_simulation_progress(ctx):
    with ctx.session() as driver:
        driver.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(driver)
        relative = "screenshots/known.png"
        expected = ctx.app.data["screenshot"]
        assert expected["pixels"], "Screenshot-Vertrag benötigt unabhängig bekannte Sollpixel"
        for overwritten in (False, True):
            result = driver.command(COMMAND, {"path": relative})
            assert result == {"path": relative, "width": expected["width"],
                              "height": expected["height"], "overwritten": overwritten}, result
            width, height, rows = rgb_pixels(driver.session.artifacts / relative)
            assert (width, height) == (expected["width"], expected["height"])
            for pixel in expected["pixels"]:
                x, y = pixel["at"]
                actual = list(rows[y][x * 3:x * 3 + 3])
                assert actual == pixel["rgb"], (pixel, actual)
            assert ctx.probe(driver) == before, "Capture verändert die Simulation"
        driver.command(COMMAND, {"path": "../escape.png"}, expected_error="invalid_screenshot_path")
        assert not (driver.session.artifacts.parent / "escape.png").exists()
        assert ctx.probe(driver) == before
