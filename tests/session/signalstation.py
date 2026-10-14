"""Observe Signalstation through inspect; all progress uses explicit commands."""
import json
import time

NAME = "bevy_ecs::name::Name"
TRANSFORM = "bevy_transform::components::transform::Transform"
CAMERA = "bevy_camera::camera::Camera"
RESOURCES = {
    "load": "test_app::level::LevelLoad",
    "player": "test_app::player::Player",
    "view": "test_app::camera::ViewState",
    "expedition": "test_app::expedition::Expedition",
    "practice": "test_app::shooting::Practice",
    "coins": "test_app::coins::CoinRun",
    "ui": "test_app::ui::UiState",
    "settings": "test_app::settings::Settings",
    "time": "bevy_time::time::Time<()>",
}


def resource_query(type_path, projection="value"):
    return {"source": "resources", "selector": {"kind": "type", "type_path": type_path},
            "projection": {"kind": projection}}


def entities(projection=None, entity=None, with_types=(), without=()):
    return {"source": "entities", "entity": entity, "with": list(with_types),
            "without": list(without), "projection": projection or {"kind": "summary"}}


def components(*paths):
    return {"kind": "components", "selection": {"kind": "listed", "type_paths": list(paths)}}


def nanos(duration):
    return duration["secs"] * 1_000_000_000 + duration["nanos"]


class Signalstation:
    def __init__(self, driver, application):
        self.driver = driver
        self.application = application
        self.artifacts = driver.session.artifacts
        self.start_ticks = None
        self.ready_ticks = None
        self.explicit_ticks = 0

    def query(self, arguments, expected_error=None):
        return self.driver.command("inspect.query", arguments, expected_error=expected_error)

    def resource(self, name):
        path = RESOURCES.get(name, name)
        result = self.query(resource_query(path))
        assert len(result["items"]) == 1, result
        item = result["items"][0]["result"]
        assert item["type_path"] == path and item["value"]["status"] == "readable", result
        return item["value"]["value"]

    def tick_count(self):
        clock = self.resource("time")
        step = self.application.data["tick_nanos"]
        assert nanos(clock["delta"]) == step, clock
        ticks, remainder = divmod(nanos(clock["elapsed"]), step)
        assert remainder == 0, clock
        return ticks

    def advance(self, ticks, pace=None):
        before = self.tick_count()
        result = self.driver.command("tick.warp.start", {"ticks": ticks,
                                     "pace": pace or {"kind": "as_fast_as_possible"}})
        assert result == {"requested_ticks": ticks, "executed_ticks": ticks, "outcome": "completed"}, result
        self.explicit_ticks += ticks
        assert self.tick_count() == before + ticks
        return result

    def engine_ready(self):
        detail = self.driver.session.detail()
        assert detail["state"] == "EngineReady", detail
        events = self.driver.session.poll()
        states = [e["state"] for e in events if e["kind"] == "lifecycle"]
        assert states.count("SessionReady") == states.count("EngineReady") == 1, states
        assert states.index("SessionReady") < states.index("EngineReady"), states
        ready = [e["event"] for e in events
                 if e["kind"] == "event" and e["event"]["kind"] == "engine_ready"]
        assert len(ready) == 1 and ready[0]["ticks"] > 0, ready
        self.start_ticks = ready[0]["ticks"]
        assert self.tick_count() == self.start_ticks
        return {"session_id": self.driver.session.id, "startup_ticks": self.start_ticks,
                "artifact_dir": str(self.artifacts), "states": states}

    def wait_ready(self):
        budget = self.application.data["game_ready"]
        deadline = time.monotonic() + budget["timeout_seconds"]
        for tick in range(budget["max_ticks"] + 1):
            state = self.resource("load")
            if state == "Ready":
                self.ready_ticks = self.tick_count()
                return {"state": state, "additional_ticks": tick, "ready_ticks": self.ready_ticks}
            assert state == "Loading", {"load": state, "ui": self.resource("ui")}
            if tick == budget["max_ticks"] or time.monotonic() >= deadline:
                raise AssertionError(f"Spiel wurde nicht Ready: {tick} Ticks, Zustand {state}")
            self.advance(1)

    def snapshot(self, label):
        state = {name: self.resource(name) for name in RESOURCES}
        state["cameras"] = self.query(entities(components(NAME, TRANSFORM), with_types=(CAMERA,)))
        assert state["cameras"]["items"], "Keine Kamera im Spiel"
        directory = self.artifacts / "checks"
        directory.mkdir(exist_ok=True)
        (directory / f"{label}.json").write_text(json.dumps(state, indent=2, ensure_ascii=False) + "\n")
        return state

    def unchanged(self, expected, label):
        actual = self.snapshot(label)
        changed = [key for key in expected if actual[key] != expected[key]]
        assert not changed, f"Simulation wurde verändert: {changed}; siehe checks/{label}.json"
        return {"snapshot": f"checks/{label}.json", "ticks_since_ready": self.tick_count() - self.ready_ticks}
