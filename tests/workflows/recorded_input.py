from tests.catalog import scenario
from tests.session.runtime import Context
from tests.recording.contracts import read_recording


@scenario("workflows", "record_input_replay_through_other_interface",
          capabilities=("state", "keyboard"))
def record_input_replay_through_other_interface(ctx):
    with ctx.session() as source:
        source.command("tick.warp.start", {"ticks": 1})
        before = ctx.probe(source)
        relative = "recordings/input.jsonl"
        source.command("recording.start", {"path": relative})
        source.command("input.keyboard.press", {"key": "a"})
        source.command("tick.warp.start", {"ticks": 2})
        source.command("input.keyboard.release", {"key": "a"})
        source.command("tick.warp.start", {"ticks": 1})
        assert source.command("recording.stop", {})["recorded_commands"] == 4
        expected = ctx.probe(source)
        assert expected["ticks"] == before["ticks"] + 3
        assert expected["keyboard"]["press_count"] == before["keyboard"]["press_count"] + 1
        assert expected["keyboard"]["release_count"] == before["keyboard"]["release_count"] + 1
        assert "a" not in expected["keyboard"]["pressed"]
        recording = source.session.artifacts / relative
        read_recording(recording)
        other_interface = "repl" if ctx.interface == "json" else "json"
        other = Context(ctx.server, ctx.app, ctx.mode, other_interface)
        with other.session() as destination:
            destination.command("tick.warp.start", {"ticks": 1})
            initial = other.probe(destination)
            assert initial["process_id"] != expected["process_id"]
            (destination.session.artifacts / "input.jsonl").write_text(recording.read_text())
            destination.command("replay.start", {"path": "input.jsonl"})
            actual = other.probe(destination)
            assert actual["ticks"] == initial["ticks"] + 3
            assert actual["keyboard"] == expected["keyboard"], (expected, actual)
