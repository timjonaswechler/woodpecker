from tests.catalog import scenario

PRESS, RELEASE = "input.keyboard.press", "input.keyboard.release"


@scenario("keyboard", "queued_press_hold_release", commands=(PRESS, RELEASE),
          capabilities=("state", "keyboard"))
def queued_press_hold_release(ctx):
    with ctx.session() as driver:
        # A first explicit tick initializes the app's ordinary input observation.
        driver.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(driver)
        driver.command(PRESS, {"key": "a"})
        assert ctx.probe(driver) == before, "Keyboard-Input wurde vor dem Tick verarbeitet"
        driver.command(PRESS, {"key": "a"}, expected_error="key_already_pressed")
        driver.command("tick.warp.start", {"ticks": 1})
        pressed = ctx.probe(driver)["keyboard"]
        assert "a" in pressed["pressed"]
        assert pressed["press_count"] == before["keyboard"]["press_count"] + 1
        driver.command("tick.warp.start", {"ticks": 2})
        assert ctx.probe(driver)["keyboard"]["press_count"] == pressed["press_count"]
        driver.command(RELEASE, {"key": "a"})
        assert "a" in ctx.probe(driver)["keyboard"]["pressed"]
        driver.command("tick.warp.start", {"ticks": 1})
        released = ctx.probe(driver)["keyboard"]
        assert "a" not in released["pressed"]
        assert released["release_count"] == before["keyboard"]["release_count"] + 1
        driver.command(RELEASE, {"key": "a"}, expected_error="key_not_pressed")
        driver.command(PRESS, {"key": "not-a-key"}, expected_error="invalid_key")
        assert ctx.probe(driver)["keyboard"] == released
