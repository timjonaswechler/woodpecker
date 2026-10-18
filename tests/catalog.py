"""Behavioral contracts, selected by feature, session mode and public interface."""
from dataclasses import dataclass
from importlib import import_module

MODES = ("rendered",)


@dataclass(frozen=True)
class Scenario:
    name: str
    feature: str
    function: object
    commands: tuple
    capabilities: tuple = ()
    modes: tuple = MODES
    interfaces: tuple = ("json", "repl")
    programs: tuple = ()
    independent: bool = False
    features: tuple = ()


SCENARIOS = []


def scenario(feature, name, commands=(), capabilities=(),
             modes=MODES, interfaces=("json", "repl"),
             programs=(), independent=False, features=()):
    def register(function):
        identifier = f"{feature}.{name}"
        if any(s.name == identifier for s in SCENARIOS):
            raise ValueError(f"Doppeltes Szenario: {identifier}")
        SCENARIOS.append(Scenario(identifier, feature, function, tuple(commands),
                                  tuple(capabilities), tuple(modes), tuple(interfaces),
                                  tuple(programs), independent, tuple(features)))
        return function
    return register


def scenarios():
    for module in (
        "connection.lifecycle", "connection.clock", "connection.startup", "interfaces.contracts",
        "workflows.signalstation", "workflows.replay", "workflows.isolation",
        "connection.protocol_faults.contracts", "connection.activity_retention.contracts",
        "recording.failures", "replay.failures", "reports.observation", "reports.panic_abort.contracts",
    ):
        import_module(f"tests.{module}")
    return tuple(SCENARIOS)


def select(features=None, modes=None, interfaces=None, names=None):
    if modes is not None and set(modes) - set(MODES):
        raise ValueError("--mode unterstützt nur rendered")
    available = scenarios()
    known = {feature for s in available for feature in (s.feature, *s.features)} | {"rust"}
    if features is not None and set(features) - known:
        raise ValueError(f"Unbekannte Features: {', '.join(sorted(set(features) - known))}")
    if names is not None and set(names) - {s.name for s in available}:
        raise ValueError(f"Unbekannte Szenarien: {', '.join(sorted(set(names) - {s.name for s in available}))}")
    selected = []
    for item in available:
        if features is not None and not set(features).intersection((item.feature, *item.features)):
            continue
        if names is not None and item.name not in names:
            continue
        for mode in (None,) if item.independent else modes or MODES:
            for interface in interfaces or ("json", "repl"):
                if (item.independent or mode in item.modes) and interface in item.interfaces:
                    selected.append((item, mode, interface))
    return selected
