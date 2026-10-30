"""Observe x, move one tick at a time to x + 0.2, release and check stillness."""
import time

from _cli import CONFIG, LEVEL, PLAYER, TICKS, Run, arguments


def main():
    args = arguments(__doc__)
    with Run(args, "feedback") as game:
        session = game.own(game.cli("session", "create", "--config", str(CONFIG))["id"])
        game.wait_engine(session)
        startup_ticks = game.resource(session, TICKS)["ticks"]

        # Preparation is explicit, separate from the movement budget.
        deadline = time.monotonic() + 10
        for extra_ticks in range(121):
            load = game.resource(session, LEVEL)
            if load == "Ready":
                break
            assert load == "Loading", load
            if extra_ticks == 120 or time.monotonic() >= deadline:
                raise TimeoutError("Level not ready within 120 explicit ticks / 10 s")
            assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
        game.note(preparation={"engine_ticks": startup_ticks, "level_ticks": extra_ticks})

        initial = game.resource(session, PLAYER)["position"]
        before = game.resource(session, TICKS)["ticks"]
        target_x = initial[0] + 0.2
        position = initial
        assert game.call(session, "input.keyboard.press", {"key": "arrow_right"}) is None

        for moving_ticks in range(1, 5):
            # Act once, then observe. No blind multi-tick sequence or repeated press.
            assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
            previous = position
            position = game.resource(session, PLAYER)["position"]
            assert position[0] > previous[0], "Blocked movement; stop instead of retrying"
            assert position[1] == initial[1], position
            reached = position[0] >= target_x
            game.note(decision={"x": position[0], "target_x": target_x,
                                "moving_ticks": moving_ticks,
                                "next": "release" if reached else "hold for another tick"})
            if reached:
                break
        else:
            raise RuntimeError("Target not reached within four moving ticks")

        assert game.call(session, "input.keyboard.release", {"key": "arrow_right"}) is None
        assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
            "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
        }
        assert game.resource(session, PLAYER)["position"] == position, "Release must stop movement"
        assert game.resource(session, TICKS)["ticks"] - before == moving_ticks + 1
        game.note(observation={"initial": initial, "final": position,
                               "target_x": target_x, "moving_ticks": moving_ticks,
                               "action_ticks": moving_ticks + 1})
        game.stop(session)
    print(f"PASS: x {initial[0]} -> {position[0]}, target {target_x}; "
          f"{moving_ticks} moving ticks + one still tick; owned session ended.")


if __name__ == "__main__":
    main()
