#!/usr/bin/env python3
"""Explicit local product/SDK proof. Never starts CI or automatically retries."""
import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time

SOURCE = Path(__file__).resolve().parent
ROOT = SOURCE.parents[2]
ARTIFACTS = ROOT / "target/headless-product-proof"
BUILD = Path(os.environ.get("CARGO_TARGET_DIR", ARTIFACTS / "build")).resolve()
BINARY = BUILD / "debug/woodpecker-offscreen-fixture"
CASES = ("healthy", "device-loss", "early-loss", "native-default")
EXECUTION_SOURCES = ("Cargo.toml", "Cargo.lock", "src/main.rs")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_fingerprint():
    paths = [SOURCE / name for name in EXECUTION_SOURCES]
    paths += [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    paths += sorted((ROOT / "src").rglob("*.rs"))
    records = [(str(path.relative_to(ROOT)), sha(path)) for path in paths]
    return hashlib.sha256(json.dumps(records, separators=(",", ":")).encode()).hexdigest()


def child_environment(case_root=None):
    result = os.environ.copy()
    for key in list(result):
        if key.startswith("WGPU_") or key in ("DISPLAY", "WAYLAND_DISPLAY"):
            result.pop(key)
    temp = ARTIFACTS / "tmp"
    temp.mkdir(parents=True, exist_ok=True)
    result.update(CARGO_TARGET_DIR=str(BUILD), RUST_LOG="info", RUST_BACKTRACE="1",
                  TMPDIR=str(temp), TMP=str(temp), TEMP=str(temp))
    if case_root is not None:
        result.update(
            WOODPECKER_OFFSCREEN_TEST_GPU="1",
            WOODPECKER_STARTUP='{"kind":"manual"}',
            WOODPECKER_TICK_CLOCK='{"kind":"fixed","seconds_per_tick":0.016666667}',
            WOODPECKER_TICK_PACE='{"kind":"as_fast_as_possible"}',
        )
    return result


def bounded(argv, log, seconds, case_root=None):
    start = time.monotonic()
    with log.open("xb") as output:
        process = subprocess.Popen(
            argv, cwd=ROOT, env=child_environment(case_root), stdin=subprocess.PIPE,
            stdout=output, stderr=subprocess.STDOUT, start_new_session=True,
        )
        timed_out = False
        try:
            code = process.wait(timeout=seconds)
        except subprocess.TimeoutExpired:
            timed_out = True
            # This directly owned new group only; never find/stop unrelated processes.
            os.killpg(process.pid, signal.SIGKILL)
            code = process.wait(timeout=5)
        except BaseException:
            if process.poll() is None:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
            raise
        finally:
            process.stdin.close()
    return {"argv": list(map(str, argv)), "pid": process.pid, "exit_code": code,
            "timeout": timed_out, "seconds": time.monotonic() - start,
            "reaped": process.poll() is not None, "log": str(log.relative_to(ROOT))}


def parse_args(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("plan", help="list fixed cases; no writes, build or GPU")
    build = commands.add_parser("build", help="bounded isolated Cargo build; no app execution")
    build.add_argument("--online", action="store_true", help="explicitly permit ordinary Cargo downloads")
    run = commands.add_parser("run", help="execute the fixed four-case local Metal campaign")
    run.add_argument("--allow-gpu", action="store_true", help="requires prior explicit local GPU authorization")
    run.add_argument("--name", required=True, help="new artifact directory name; never overwritten")
    run.add_argument("--deadline-utc", help="optional shorter ISO UTC deadline, never longer than two hours")
    args = parser.parse_args(argv)
    if args.command == "run":
        if not args.allow_gpu:
            parser.error("run requires --allow-gpu and explicit local authorization")
        if not re.fullmatch(r"[a-zA-Z0-9][a-zA-Z0-9_-]{0,63}", args.name):
            parser.error("--name must be a new simple directory name")
        if sys.platform != "darwin":
            parser.error("this qualified profile is macOS/Metal only; no fallback or green skip")
    return args


def main(argv=None):
    args = parse_args(argv)
    if args.command == "plan":
        for number, case in enumerate(CASES, 1):
            print(f"{number:02d} {case}")
        return 0
    ARTIFACTS.mkdir(parents=True, exist_ok=True)
    if args.command == "build":
        number = len(list(ARTIFACTS.glob("build-*.json"))) + 1
        command = ["cargo", "build", "--locked", "--manifest-path", str(SOURCE / "Cargo.toml")]
        if not args.online:
            command.insert(2, "--offline")
        fingerprint = source_fingerprint()
        result = bounded(command, ARTIFACTS / f"build-{number}.log", 900)
        if result["exit_code"] == 0 and source_fingerprint() != fingerprint:
            result.update(exit_code=2, preparation_error="sources changed during build")
        if result["exit_code"] == 0:
            result.update(binary_sha256=sha(BINARY), source_fingerprint=fingerprint)
            (ARTIFACTS / "build-latest.json").write_text(json.dumps(result, indent=2) + "\n")
        (ARTIFACTS / f"build-{number}.json").write_text(json.dumps(result, indent=2) + "\n")
        print(json.dumps(result, indent=2))
        return 0 if result["exit_code"] == 0 else 2
    stamp_file = ARTIFACTS / "build-latest.json"
    if not BINARY.is_file() or not stamp_file.is_file():
        raise SystemExit("build prerequisite missing; no automatic build/fallback")
    stamp = json.loads(stamp_file.read_text())
    if stamp["binary_sha256"] != sha(BINARY) or stamp["source_fingerprint"] != source_fingerprint():
        raise SystemExit("stale prepared binary; explicitly rebuild before any GPU execution")
    now = dt.datetime.now(dt.timezone.utc)
    deadline = now + dt.timedelta(hours=2)
    if args.deadline_utc:
        explicit = dt.datetime.fromisoformat(args.deadline_utc)
        if explicit.tzinfo is None:
            raise SystemExit("deadline requires an explicit timezone")
        deadline = min(deadline, explicit)
    if now + dt.timedelta(seconds=120) >= deadline - dt.timedelta(minutes=5):
        raise SystemExit("insufficient budget for a bounded case and shutdown reserve")
    folder = ARTIFACTS / "runs" / args.name
    folder.mkdir(parents=True, exist_ok=False)
    ledger = []
    (folder / "RUN.json").write_text(json.dumps({
        "deadline_utc": deadline.isoformat(), "case_limit_seconds": 120,
         "planned_cases": list(CASES),
        "binary_sha256": sha(BINARY),
    }, indent=2) + "\n")
    for number, case in enumerate(CASES, 1):
        if dt.datetime.now(dt.timezone.utc) + dt.timedelta(seconds=120) >= deadline - dt.timedelta(minutes=5):
            raise SystemExit("campaign budget exhausted; no retry or additional case")
        case_root = folder / f"{number:02d}-{case}"
        case_root.mkdir()
        snapshot = case_root / "source"
        snapshot.mkdir()
        for name in EXECUTION_SOURCES:
            destination = snapshot / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes((SOURCE / name).read_bytes())
        result = bounded([str(BINARY), "verify", case, str(case_root / "session")], case_root / "process.log", 120, case_root)
        result.update(case=case, number=number, binary_sha256=sha(BINARY))
        ledger.append(result)
        (folder / "PROCESSES.json").write_text(json.dumps(ledger, indent=2) + "\n")
        print(json.dumps(result), flush=True)
        if result["exit_code"] != 0:
            return 2  # retain evidence; never retry or silently replace a failed process
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
