from tests.catalog import scenario

MOVE, BY = "input.pointer.move_to", "input.pointer.move_by"
PRESS, RELEASE, SCROLL = "input.pointer.press", "input.pointer.release", "input.pointer.scroll"


@scenario("pointer", "movement_buttons_and_scroll", commands=(MOVE, BY, PRESS, RELEASE, SCROLL),
          capabilities=("state", "pointer"))
def movement_buttons_and_scroll(ctx):
    with ctx.session() as driver:
        driver.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(driver)
        driver.command(MOVE, {"position": [20, 20]})
        driver.command(BY, {"delta": [5, -3]})
        driver.command(PRESS, {"button": "left"})
        driver.command(SCROLL, {"delta": [-2, 3]})
        assert ctx.probe(driver) == before, "Pointer-Input wurde vor dem Tick verarbeitet"
        driver.command(PRESS, {"button": "left"}, expected_error="pointer_button_already_pressed")
        driver.command(MOVE, {"position": [-1, 0]}, expected_error="pointer_position_out_of_bounds")
        driver.command(PRESS, {"button": "extra"}, expected_error="invalid_pointer_button")
        driver.command("tick.warp.start", {"ticks": 1})
        pressed = ctx.probe(driver)["pointer"]
        assert pressed["position"] == [25, 17], pressed
        assert "left" in pressed["pressed"]
        assert pressed["press_count"] == before["pointer"]["press_count"] + 1
        assert pressed["scroll"] == [before["pointer"]["scroll"][0] - 2,
                                     before["pointer"]["scroll"][1] + 3], pressed
        driver.command(RELEASE, {"button": "left"})
        assert "left" in ctx.probe(driver)["pointer"]["pressed"]
        driver.command("tick.warp.start", {"ticks": 1})
        after = ctx.probe(driver)["pointer"]
        assert "left" not in after["pressed"]
        assert after["release_count"] == before["pointer"]["release_count"] + 1
        driver.command(RELEASE, {"button": "left"}, expected_error="pointer_button_not_pressed")
        driver.command("tick.warp.start", {"ticks": 1})
        assert ctx.probe(driver)["pointer"] == after, "Input wurde unbeabsichtigt wiederholt"
