"""Play the shared game through actual session input and replay its outcome."""
from tests.catalog import scenario


@scenario("workflows", "collect_targets_and_restart", capabilities=("state", "game", "keyboard", "pointer"))
def collect_targets_and_restart(ctx):
    with ctx.session() as driver:
        driver.command("tick.warp.start", {"ticks": 1})
        initial = ctx.probe(driver)["game"]
        assert initial == {"position": [0, 0], "collected": 0, "remaining": 3, "won": False}, initial
        driver.command("input.keyboard.press", {"key": "arrow_right"})
        driver.command("tick.warp.start", {"ticks": 3})
        driver.command("input.keyboard.release", {"key": "arrow_right"})
        assert ctx.probe(driver)["game"]["position"] == [12, 0]
        # A miss must not score; merely moving over a target must not collect it.
        collected = 0
        for position, expected in (([20, 20], 0), ([500, 130], 1), ([160, 240], 2), ([460, 270], 3)):
            driver.command("input.pointer.move_to", {"position": position})
            driver.command("tick.warp.start", {"ticks": 1})
            assert ctx.probe(driver)["game"]["collected"] == collected
            before = ctx.probe(driver)
            driver.command("input.pointer.press", {"button": "left"})
            assert ctx.probe(driver) == before
            driver.command("tick.warp.start", {"ticks": 1})
            game = ctx.probe(driver)["game"]
            assert game["collected"] == expected and game["remaining"] == 3 - expected, game
            assert game["won"] == (expected == 3), game
            driver.command("tick.warp.start", {"ticks": 2})
            assert ctx.probe(driver)["game"] == game, "Holding a click scored again"
            driver.command("input.pointer.release", {"button": "left"})
            driver.command("tick.warp.start", {"ticks": 1})
            collected = expected
        driver.command("input.keyboard.press", {"key": "f5"})
        driver.command("tick.warp.start", {"ticks": 1})
        assert ctx.probe(driver)["game"] == initial
        driver.command("input.keyboard.release", {"key": "f5"})
        driver.command("input.keyboard.press", {"key": "arrow_right"})
        driver.command("tick.warp.start", {"ticks": 45})
        driver.command("input.keyboard.release", {"key": "arrow_right"})
        driver.command("input.keyboard.press", {"key": "arrow_up"})
        driver.command("tick.warp.start", {"ticks": 13})
        touched = ctx.probe(driver)["game"]
        assert touched["position"] == [180, 52] and touched["collected"] == 1, touched


@scenario("workflows", "game_recording_replays_outcome", capabilities=("state", "game", "keyboard", "pointer", "text"))
def game_recording_replays_outcome(ctx):
    with ctx.session() as source:
        source.command("tick.warp.start", {"ticks": 1})
        source.command("recording.start", {"path": "game.jsonl"})
        source.command("input.text.input", {"text": "Mika 🐦"})
        source.command("input.keyboard.press", {"key": "arrow_right"})
        source.command("tick.warp.start", {"ticks": 5})
        source.command("input.keyboard.release", {"key": "arrow_right"})
        source.command("input.pointer.move_to", {"position": [500, 130]})
        source.command("input.pointer.press", {"button": "left"})
        source.command("tick.warp.start", {"ticks": 1})
        source.command("input.pointer.release", {"button": "left"})
        source.command("tick.warp.start", {"ticks": 1})
        source.command("recording.stop", {})
        expected = ctx.probe(source)
        assert expected["game"]["position"] == [20, 0]
        assert expected["game"]["collected"] == 1
        assert expected["text"] == "Mika 🐦"
        recording = (source.session.artifacts / "game.jsonl").read_text()
        with ctx.session() as destination:
            destination.command("tick.warp.start", {"ticks": 1})
            (destination.session.artifacts / "game.jsonl").write_text(recording)
            destination.command("replay.start", {"path": "game.jsonl"})
            actual = ctx.probe(destination)
            assert actual["process_id"] != expected["process_id"]
            for field in ("ticks", "game", "text", "keyboard", "pointer"):
                assert actual[field] == expected[field], (field, expected, actual)
