"""One app: press right, check each of four held ticks, release, then stay still."""
import time

from _cli import CONFIG, LEVEL, PLAYER, TICKS, Run, arguments


def main():
    args = arguments(__doc__)
    with Run(args, "movement") as game:
        session = game.own(game.cli("session", "create", "--config", str(CONFIG))["id"])
        game.wait_engine(session)
        startup_ticks = game.resource(session, TICKS)["ticks"]

        # EngineReady is technical readiness. The game owns LevelLoad.
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
        assert game.call(session, "input.keyboard.press", {"key": "arrow_right"}) is None
        assert game.resource(session, PLAYER)["position"] == initial
        assert game.resource(session, TICKS)["ticks"] == before

        previous = initial
        for held_tick in range(1, 5):
            # No repeated press: each individual tick must move the player.
            assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
                "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
            }
            position = game.resource(session, PLAYER)["position"]
            assert position[0] > previous[0], f"Right movement lost on held tick {held_tick}"
            assert abs(position[1] - initial[1]) <= 1e-6, position
            assert game.resource(session, TICKS)["ticks"] - before == held_tick
            previous = position

        assert game.call(session, "input.keyboard.release", {"key": "arrow_right"}) is None
        assert game.resource(session, PLAYER)["position"] == previous
        assert game.resource(session, TICKS)["ticks"] - before == 4
        assert game.call(session, "tick.warp.start", {"ticks": 1}) == {
            "requested_ticks": 1, "executed_ticks": 1, "outcome": "completed",
        }
        assert game.resource(session, PLAYER)["position"] == previous
        assert game.call(session, "tick.warp.start", {"ticks": 3}) == {
            "requested_ticks": 3, "executed_ticks": 3, "outcome": "completed",
        }
        assert game.resource(session, PLAYER)["position"] == previous
        assert game.resource(session, TICKS)["ticks"] - before == 8
        game.note(observation={"initial": initial, "final": previous, "action_ticks": 8})
        game.stop(session)
    print("PASS: four moving ticks and four still ticks; owned session ended.")


if __name__ == "__main__":
    main()
