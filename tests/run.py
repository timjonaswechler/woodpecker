"""Run product contracts across session modes and public command interfaces."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import time
import tomllib
import traceback

from tests.catalog import scenarios, select
from tests.session.application import Application, ROOT, Unavailable
from tests.session.runtime import Context, Server

MODES = ("rendered", "headless")
INTERFACES = ("json", "repl")


def csv(value):
    values = value.split(",")
    if not all(values) or len(set(values)) != len(values):
        raise argparse.ArgumentTypeError("Eine nichtleere Liste ohne doppelte Einträge angeben")
    return values


def rust_jobs(features):
    cargo = ["cargo", "test", "--locked", "--all-features"]
    serial = ["--", "--test-threads=1"]
    if features is None or "rust" in features:
        return [("rust.contracts", cargo + ["--lib", "--bins"] + serial),
                ("rust.documentation", cargo + ["--doc"]),
                ("rust.no_default_features", ["cargo", "test", "--locked", "--no-default-features", "--lib"] + serial)]
    modules = {
        "connection": ["server::tests", "session::api_tests"], "warp": ["session::plugin"],
        "inspect": ["session::inspect"], "keyboard": ["session::input::keyboard"],
        "pointer": ["session::input::pointer"], "text": ["session::input::text"],
        "screenshot": ["session::screenshot"], "recording": ["session::recording"],
        "replay": ["session::replay"], "reports": ["report::", "server::tests"],
        "json": ["cli::script"], "repl": ["cli::repl"],
        "shutdown": ["session::plugin"],
    }
    jobs = {}
    for feature in features:
        for module in modules.get(feature, []):
            jobs[f"rust.{module}"] = cargo + ["--lib", module] + serial
    return list(jobs.items())


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--all", action="store_true", help="Alle Verhaltensbereiche und beide Modi/Zugänge")
    parser.add_argument("--feature", type=csv, help="connection,warp,inspect,keyboard,pointer,text,screenshot,shutdown,recording,replay,reports,json,repl,workflows,rust")
    parser.add_argument("--scenario", type=csv)
    parser.add_argument("--mode", type=csv)
    parser.add_argument("--interface", type=csv)
    parser.add_argument("--layer", choices=("all", "system", "rust"), default="all")
    parser.add_argument("--app", type=Path, default=ROOT / "tests/application/app.json")
    parser.add_argument("--cli", type=Path, default=ROOT / "target/debug/woodpecker")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--no-build", action="store_true", help="Bereits gebaute Artefakte verwenden")
    parser.add_argument("--faults-only", action="store_true", help="Nur modusunabhängige Prozess-, Protokoll- und Report-Fehlerfälle")
    parser.add_argument("--allow-rendered", action="store_true", help="Grafische Sessions dieses Laufs ausdrücklich freigeben")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)
    if args.all and (args.feature or args.scenario or args.mode or args.interface or args.faults_only):
        parser.error("--all kann nicht mit einer eingeschränkten Auswahl kombiniert werden")
    if args.layer == "rust" and (args.mode or args.interface or args.scenario):
        parser.error("Rust-Verträge haben keine Session-Modus-/Zugangs-Matrix")
    if args.layer == "rust" and args.faults_only:
        parser.error("--faults-only wählt System-Szenarien; --layer rust ist damit nicht kombinierbar")
    if args.mode and set(args.mode) - set(MODES):
        parser.error("--mode unterstützt rendered,headless")
    if args.interface and set(args.interface) - set(INTERFACES):
        parser.error("--interface unterstützt json,repl")
    try:
        cells = select(args.feature, args.mode, args.interface, args.scenario)
        if args.faults_only:
            cells = [(s, m, i) for s, m, i in cells if s.independent]
    except ValueError as error:
        parser.error(str(error))
    jobs = rust_jobs(args.feature) if args.layer != "system" and not args.scenario and not args.faults_only else []
    if args.layer == "rust":
        cells = []
    signals = (args.layer != "rust" and args.scenario is None
               and (args.feature is None or "connection" in args.feature))
    if args.layer != "rust" and (args.mode or args.interface or args.scenario) and not cells and not signals:
        parser.error("Diese Kombination enthält keine System-Szenarien; Rust-Nachweise haben keine Zugangs-/Modus-Matrix")
    if not cells and not jobs and not signals:
        parser.error("Auswahl enthält keine ausführbaren Verträge")
    names = [name for name, _ in jobs]
    if signals:
        names.append("connection.server_signals [mode-independent]")
    names += [f"{s.name} [{mode or 'mode-independent'}/{interface}]" for s, mode, interface in cells]
    expected_results = {name for name, _ in jobs}
    if signals:
        expected_results.add("connection.server_signals")
    expected_results.update(f"{s.name} [{mode or 'mode-independent'}/{interface}]" for s, mode, interface in cells)
    if args.list:
        print("\n".join(names))
        return 0

    output = (args.output or ROOT / "target/test-runs" /
              datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")).resolve()
    output.mkdir(parents=True, exist_ok=False)
    results = []
    report = {"version": 1, "selection": names, "results": results,
              "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
              "dirty": bool(subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT)),
              "platform": sys.platform, "python": sys.version, "app": str(args.app.resolve())}
    sources = subprocess.check_output(["git", "ls-files", "--cached", "--others", "--exclude-standard"],
                                      cwd=ROOT, text=True).splitlines()
    report["source_sha256"] = {
        path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
        for path in sources if (path.startswith(("src/", "tests/")) or path in ("Cargo.toml", "Cargo.lock"))
        and not path.startswith("tests/diagnostics/") and (ROOT / path).is_file()
        and Path(path).suffix in (".rs", ".py", ".toml", ".lock", ".json")
    }
    report["rustc"] = subprocess.check_output(["rustc", "--version"], text=True).strip()
    if args.app.is_file():
        report["app_sha256"] = hashlib.sha256(args.app.read_bytes()).hexdigest()
    diff = subprocess.run(["git", "diff", "HEAD"], cwd=ROOT, capture_output=True, text=True, check=True)
    (output / "tracked.diff").write_text(diff.stdout)

    def save():
        passed = {r["name"] for r in results if r["status"] == "passed"}
        report["status"] = ("passed" if expected_results <= passed
                            and all(r["status"] == "passed" for r in results) else "incomplete")
        if any(r["status"] == "failed" for r in results):
            report["status"] = "failed"
        (output / "results.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n")

    def execute(name, callback):
        started = time.monotonic()
        result = {"name": name}
        try:
            callback()
            result["status"] = "passed"
        except Unavailable as error:
            result.update(status="unavailable", error=str(error))
        except Exception as error:
            result.update(status="failed", error=str(error), traceback=traceback.format_exc())
        result["seconds"] = round(time.monotonic() - started, 3)
        results.append(result)
        save()
        print(f"{result['status'].upper():11} {name}", flush=True)
        if result.get("error"):
            print(f"  {result['error']}", flush=True)
        return result["status"] == "passed"

    def command(name, arguments, require_tests=False):
        with (output / f"{name.replace(':', '_')}.log").open("w") as log:
            process = subprocess.run(arguments, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        if process.returncode:
            raise AssertionError(f"Exit {process.returncode}: {arguments}; siehe {log.name}")
        if require_tests:
            text = Path(log.name).read_text()
            if not sum(int(n) for n in re.findall(r"running (\d+) tests?", text)):
                raise AssertionError(f"Rust-Auswahl hat keinen Test ausgeführt: {arguments}")

    prepared = True
    app_builds = {}
    if not args.no_build:
        builds = []
        programs = {p for item, _, _ in cells for p in item.programs}
        if jobs:
            programs.update(("protocol", "observation"))
        if signals:
            programs.add("protocol")
        program_builds = {
            "protocol": ["cargo", "build", "--locked", "--manifest-path", "tests/connection/protocol_faults/peer/Cargo.toml"],
            "observation": ["cargo", "build", "--locked", "--example", "observation_fixture"],
            "activity": ["cargo", "build", "--locked", "--example", "activity_fixture"],
            "abort": ["cargo", "build", "--locked", "--manifest-path", "tests/reports/panic_abort/program/Cargo.toml"],
        }
        builds.extend(program_builds[p] for p in sorted(programs))
        if cells or signals:
            builds.append(["cargo", "build", "--locked", "--features", "cli", "--bin", "woodpecker"])
        for index, arguments in enumerate(builds):
            prepared &= execute(f"build.{index}", lambda a=arguments, i=index: command(f"build.{i}", a))
        if any(not s.independent for s, _, _ in cells):
            try:
                app = Application(args.app)
            except (Unavailable, ValueError, KeyError, json.JSONDecodeError):
                app = None  # Each selected cell reports its own missing/invalid prerequisites.
            if app is not None:
                completed_builds = {}
                for mode in dict.fromkeys(m for s, m, _ in cells if not s.independent):
                    if mode == "rendered" and not args.allow_rendered:
                        continue
                    try:
                        arguments = app.build_command(mode)
                    except (Unavailable, ValueError, KeyError, TypeError, tomllib.TOMLDecodeError):
                        continue
                    key = tuple(arguments)
                    if key not in completed_builds:
                        completed_builds[key] = execute(f"build.app.{mode}",
                            lambda a=arguments, m=mode: command(f"build.app.{m}", a))
                    app_builds[mode] = completed_builds[key]

    for name, arguments in jobs:
        def run_job(n=name, a=arguments):
            if not prepared:
                raise Unavailable("Build-Voraussetzungen fehlgeschlagen")
            command(n, a, require_tests="--doc" not in a)
        execute(name, run_job)

    if signals:
        def run_signals():
            if not prepared:
                raise Unavailable("Build-Voraussetzungen fehlgeschlagen")
            from tests.connection.server_shutdown import run
            run(args.cli, output / "server-signals")
        execute("connection.server_signals", run_signals)

    for index, (item, mode, interface) in enumerate(cells):
        name = f"{item.name} [{mode or 'mode-independent'}/{interface}]"
        directory = output / f"{index:03}-{item.name}-{mode or 'independent'}-{interface}"
        def run_cell(s=item, m=mode, i=interface, d=directory):
            if not prepared:
                raise Unavailable("Build-Voraussetzungen fehlgeschlagen")
            app = None
            if not s.independent:
                app = Application(args.app)
                app.require(m, s.capabilities)
                if app_builds.get(m) is False:
                    raise Unavailable(f"Test-App-Build für {m} fehlgeschlagen")
            if m == "rendered" and not args.allow_rendered:
                raise Unavailable("Grafische Sessions benötigen --allow-rendered für diesen Lauf")
            if not args.cli.is_file():
                raise Unavailable(f"CLI-Binary fehlt: {args.cli}")
            with Server(args.cli, d) as server:
                s.function(Context(server, app, m, i))
        execute(name, run_cell)
    save()
    print(f"Ergebnis: {report['status']}; Bericht: {output / 'results.json'}")
    if report["status"] == "passed":
        return 0
    return 1 if report["status"] == "failed" else 2


if __name__ == "__main__":
    sys.exit(main())
