from tests.catalog import scenario

COMMAND = "input.text.input"


@scenario("text", "unicode_and_utf8_limit", commands=(COMMAND,), capabilities=("state", "text"))
def unicode_and_utf8_limit(ctx):
    with ctx.session() as driver:
        driver.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(driver)
        message = "Grüße 日本語 🐦"
        driver.command(COMMAND, {"text": message})
        assert ctx.probe(driver) == before, "Text wurde vor dem Tick verarbeitet"
        driver.command("tick.warp.start", {"ticks": 1})
        after = ctx.probe(driver)
        assert after["text"] == before["text"] + message, after
        driver.command(COMMAND, {"text": ""})
        driver.command(COMMAND, {"text": "ä" * 8193}, expected_error="text_too_large")
        driver.command("tick.warp.start", {"ticks": 1})
        assert ctx.probe(driver)["text"] == after["text"]
        boundary = "ä" * 8192  # Exactly 16,384 UTF-8 bytes.
        driver.command(COMMAND, {"text": boundary})
        driver.command("tick.warp.start", {"ticks": 1})
        assert ctx.probe(driver)["text"] == after["text"] + boundary
