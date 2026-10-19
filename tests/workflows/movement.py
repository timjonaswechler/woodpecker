"""Movement and tick contracts in the existing Signalstation session."""
import math
import time
from tests.interfaces.json import check_result
from tests.interfaces.repl import line_for
from tests.session.runtime import until
from tests.session.signalstation import nanos
from tests.session.controls import BUTTONS, KEYS, MOTION, Controls


def run(ctx, game):
    controls = Controls(game)
    driver = game.driver

    def input_state():
        assert game.resource(KEYS)["pressed"] == []
        assert game.resource(BUTTONS)["pressed"] == []
        return {"window_focused": controls.window()["focused"]}

    ctx.steps.run("2.input_state", input_state, requires=("0.baseline",))

    def keyboard():
        controls.background()
        start = game.resource("player")["position"]
        before = game.snapshot("keyboard_before")
        controls.key("arrow_right")
        game.unchanged(before, "keyboard_queued")
        assert game.resource(KEYS)["pressed"] == []
        driver.command("input.keyboard.press", {"key": "arrow_right"}, expected_error="key_already_pressed")
        game.advance(1)
        pressed = game.resource(KEYS)
        assert pressed["pressed"] == pressed["just_pressed"] == ["ArrowRight"], pressed
        first = game.resource("player")["position"]
        assert math.dist(first, [start[0] + 0.075, start[1]]) < 0.0001, first
        game.advance(3)
        held = game.resource(KEYS)
        assert held["pressed"] == ["ArrowRight"] and held["just_pressed"] == [], held
        position = game.resource("player")["position"]
        assert math.dist(position, [start[0] + 0.3, start[1]]) < 0.0001, position
        controls.key("arrow_right", False)
        assert game.resource(KEYS)["pressed"] == ["ArrowRight"]
        assert game.resource("player")["position"] == position
        game.advance(1)
        released = game.resource(KEYS)
        assert released["pressed"] == [] and released["just_released"] == ["ArrowRight"], released
        game.advance(2)
        assert game.resource("player")["position"] == position
        assert game.resource(KEYS)["just_released"] == []
        controls.background()
        # The map toggle is an observable one-shot action while holding a key.
        controls.key("g")
        assert (game.resource("view")["mode"] == "Map") is False
        game.advance(1)
        assert (game.resource("view")["mode"] == "Map") is True
        game.advance(3)
        assert (game.resource("view")["mode"] == "Map") is True
        controls.key("g", False)
        game.advance(1)
        controls.key("g")
        game.advance(1)
        assert (game.resource("view")["mode"] == "Map") is False
        return {"start": start, "after_one_tick": first, "after_four_ticks": position,
                "press": pressed, "hold": held, "release": released, "window_focused": False}

    ctx.steps.run("2.keyboard", keyboard, requires=("2.input_state",), precondition=controls.reset)

    def pointer():
        native = controls.window()["internal"]["physical_cursor_position"]
        assert controls.pointer() is None, "Gruppe 2 erwartet einen noch unbenutzten virtuellen Cursor"
        controls.move_by([12, -4])
        assert controls.pointer() is None
        game.advance(1)
        assert controls.pointer() == [732.0, 446.0], controls.pointer()
        assert game.resource(MOTION)["delta"] == [12.0, -4.0]
        controls.move_to([100, 60])
        controls.move_by([7, -3])
        controls.move_by([2, 1])
        assert controls.pointer() == [732.0, 446.0]
        game.advance(1)
        assert controls.pointer() == [109.0, 58.0], controls.pointer()
        assert game.resource(MOTION)["delta"] == [9.0, -2.0]
        controls.move_to([80, 40])
        game.advance(1)
        assert controls.pointer() == [80.0, 40.0]
        assert game.resource(MOTION)["delta"] == [0.0, 0.0]
        controls.move_by([10000, -10000])
        game.advance(1)
        edge = controls.pointer()
        assert 1439.9 < edge[0] < 1440 and edge[1] == 0, edge
        assert game.resource(MOTION)["delta"] == [10000.0, -10000.0]
        controls.move_by([100, -100])
        game.advance(1)
        assert controls.pointer() == edge
        assert game.resource(MOTION)["delta"] == [100.0, -100.0]
        controls.move_to([80, 40])  # HUD blocks floor routing while buttons are checked.
        game.advance(1)
        for button, bevy_name in (("left", "Left"), ("middle", "Middle"), ("right", "Right")):
            controls.button(button)
            assert game.resource(BUTTONS)["pressed"] == []
            game.advance(1)
            state = game.resource(BUTTONS)
            assert state["pressed"] == state["just_pressed"] == [bevy_name], state
            game.advance(2)
            assert game.resource(BUTTONS)["just_pressed"] == []
            controls.button(button, False)
            assert game.resource(BUTTONS)["pressed"] == [bevy_name]
            game.advance(1)
            state = game.resource(BUTTONS)
            assert state["pressed"] == [] and state["just_released"] == [bevy_name], state
        game.advance(1)
        assert game.resource(BUTTONS)["just_released"] == []
        # Real OS movement may arrive while the session is paused. Virtual cursor
        # positions above are the input contract; the native position is evidence,
        # not an invariant requiring the person running the suite to stop moving.
        native_after = controls.window()["internal"]["physical_cursor_position"]
        controls.assert_reset()
        return {"initial_relative_position": [732, 446], "bounded_cursor": edge,
                "full_delta_at_edge": [100, -100],
                "native_cursor_before": native, "native_cursor_after": native_after}

    ctx.steps.run("2.pointer", pointer, requires=("2.input_state",), precondition=controls.reset)

    def click_route():
        target = [-30.0, 0.0, 22.0]
        screen = controls.screen_position(target)
        controls.move_to(screen)
        controls.button()
        assert game.resource("player")["route"] == []
        game.advance(1)
        route = game.resource("player")["route"]
        assert route and math.dist(route[-1], target) < 0.02, (screen, route)
        controls.button(down=False)
        before = game.resource("player")["position"]
        game.advance(8)
        after = game.resource("player")["position"]
        assert math.dist(after, [target[0], target[2]]) < math.dist(before, [target[0], target[2]])
        controls.key("arrow_left")
        game.advance(1)
        player = game.resource("player")
        assert player["route"] == [] and player["position"][0] < after[0], player
        controls.key("arrow_left", False)
        game.advance(1)
        assert game.resource("player")["position"] == player["position"]
        return {"floor_target": target, "screen": screen, "route": route,
                "after_route": after, "after_keyboard_takeover": player["position"]}

    ctx.steps.run("2.click_route", click_route, requires=("2.input_state",), precondition=controls.reset)

    def time_equivalence():
        results = []
        for mode in ("single", "warp", "think_pauses", "paced"):
            controls.reset()
            begin = game.resource("time")
            controls.key("arrow_right")
            if mode == "single":
                for _ in range(12):
                    game.advance(1)
            elif mode == "think_pauses":
                for index in range(3):
                    game.advance(4)
                    paused = game.snapshot(f"{mode}_{index}")
                    time.sleep(0.25)
                    game.unchanged(paused, f"{mode}_{index}_after_wait")
            else:
                game.advance(12, {"kind": "ticks_per_second", "target": 24} if mode == "paced" else None)
            player = game.resource("player")
            elapsed = nanos(game.resource("time")["elapsed"]) - nanos(begin["elapsed"])
            assert elapsed == 200_000_004, (mode, elapsed)
            assert math.dist(player["position"], [-31.1, 22.0]) < 0.0001, (mode, player)
            assert abs(player["walked"] - 0.9) < 0.0001, (mode, player)
            if results:
                assert player == results[0]["player"], (mode, player, results[0])
            results.append({"mode": mode, "ticks": 12, "elapsed_nanos": elapsed, "player": player})
            controls.key("arrow_right", False)
            game.advance(1)
            assert game.resource("player")["position"] == player["position"]
        return results

    ctx.steps.run("2.time_equivalence", time_equivalence, requires=("2.input_state",),
                  precondition=controls.reset)

    def warp_control():
        before = game.snapshot("warp_before")
        driver.command("tick.warp.start", {"ticks": 0}, expected_error="invalid_tick_count")
        for command, arguments in (
            ("tick.warp.start", {"ticks": 1, "pace": {"kind": "ticks_per_second", "target": 0}}),
            ("tick.warp.set_pace", {"pace": {"kind": "ticks_per_second", "target": -1}}),
        ):
            if ctx.interface == "repl":
                # REPL short forms reject invalid pace before submitting a request.
                requests = {e.get("request_id") for e in driver.session.poll()}
                start = len(driver.lines)
                driver.send(line_for(command, arguments))
                error = "input error: pace must be finite and positive"
                driver.wait(lambda: error in driver.lines[start:], "REPL akzeptiert ungültiges pace")
                assert not any(" pending" in line for line in driver.lines[start:])
                assert {e.get("request_id") for e in driver.session.poll()} == requests
            else:
                driver.command(command, arguments, expected_error="invalid_pace")
        assert driver.command("tick.warp.stop", {}) == {"was_running": False}
        game.unchanged(before, "warp_rejected")
        start = game.tick_count()
        pending = driver.submit("tick.warp.start", {
            "ticks": 100000, "pace": {"kind": "ticks_per_second", "target": 2}})
        try:
            until(lambda: game.tick_count() > start, "Warp führt keine Ticks aus", timeout=10)
            driver.command("tick.warp.start", {"ticks": 1}, expected_error="warp_already_running")
            pace = {"kind": "ticks_per_second", "target": 10}
            assert driver.command("tick.warp.set_pace", {"pace": pace}) == {"pace": pace}
            changed_at = game.tick_count()
            until(lambda: game.tick_count() > changed_at, "Warp steht nach pace-Änderung", timeout=10)
            assert driver.command("tick.warp.stop", {}) == {"was_running": True}
            outcome = check_result(driver.outcome(pending, "tick.warp.start"))
            assert outcome["requested_ticks"] == 100000 and outcome["outcome"] == "stopped", outcome
            assert 0 < outcome["executed_ticks"] < 100000, outcome
            assert game.tick_count() - start == outcome["executed_ticks"], outcome
            game.explicit_ticks += outcome["executed_ticks"]
            stopped = game.snapshot("warp_stopped")
            time.sleep(0.25)
            game.unchanged(stopped, "warp_still_stopped")
            assert driver.command("tick.warp.stop", {}) == {"was_running": False}
            return outcome
        finally:
            driver.command("tick.warp.stop", {})

    ctx.steps.run("2.warp_control", warp_control, requires=("2.input_state",), precondition=controls.reset)

    def invalid_inputs():
        controls.move_to([80, 40])
        game.advance(1)
        before = game.snapshot("invalid_inputs_before")
        keys, buttons, cursor = game.resource(KEYS), game.resource(BUTTONS), controls.pointer()
        for command, arguments, code in (
            ("input.keyboard.press", {"key": "not-a-key"}, "invalid_key"),
            ("input.keyboard.release", {"key": "arrow_right"}, "key_not_pressed"),
            ("input.pointer.move_to", {"position": [-1, 0]}, "pointer_position_out_of_bounds"),
            ("input.pointer.move_to", {"position": [1440, 900]}, "pointer_position_out_of_bounds"),
            ("input.pointer.press", {"button": "extra"}, "invalid_pointer_button"),
            ("input.pointer.release", {"button": "left"}, "pointer_button_not_pressed"),
        ):
            driver.command(command, arguments, expected_error=code)
        controls.key("arrow_right")
        driver.command("input.keyboard.press", {"key": "arrow_right"}, expected_error="key_already_pressed")
        controls.key("arrow_right", False)
        driver.command("input.keyboard.release", {"key": "arrow_right"}, expected_error="key_not_pressed")
        controls.button()
        driver.command("input.pointer.press", {"button": "left"}, expected_error="pointer_button_already_pressed")
        controls.button(down=False)
        driver.command("input.pointer.release", {"button": "left"}, expected_error="pointer_button_not_pressed")
        game.unchanged(before, "invalid_inputs_rejected")
        assert (game.resource(KEYS), game.resource(BUTTONS), controls.pointer()) == (keys, buttons, cursor)
        game.advance(1)
        assert game.resource(KEYS)["pressed"] == game.resource(BUTTONS)["pressed"] == []
        assert controls.pointer() == cursor
        controls.assert_reset()
        return {"rejections": 10, "applied_cursor": cursor, "keys_pressed": [], "buttons_pressed": []}

    ctx.steps.run("2.invalid_inputs", invalid_inputs, requires=("2.input_state",), precondition=controls.reset)
    ctx.steps.run("2.restore", lambda: game.snapshot("movement_restored"),
                  requires=("2.input_state",), precondition=controls.reset)
