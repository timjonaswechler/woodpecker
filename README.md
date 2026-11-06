# `woodpecker`

Control and inspect a Bevy application through an explicit local session.
A loopback-only server owns multiple independent game processes; CLI clients can
disconnect without stopping accepted work.

Local clients are trusted. No access key or environment setup is required.
Remote operation is not supported.

The current integration targets Bevy 0.20.0 and requires Rust 1.97.1 or newer.

## Current implementation

The experimental v3 slice supports tick warps, pace changes, stop, reflected
resource and entity inspection, virtual pointer, keyboard and text input, PNG screenshots,
session management, JSONL recording, replay, reporting, REPL, scripts and shutdown.
Fully covered windows are not guaranteed to be capturable; an unavailable render surface produces an explicit error.
Actual black images are valid. Screenshot comparison is not a product feature.
The v2 API and its `host`/`driver` features have been removed.

Controlled sessions ignore native mouse, touch, keyboard and IME input. Their pointers
do not move the OS cursor or require window focus. Applications using Bevy UI must
enable `woodpecker/ui`, which also enables focused text input. A complete Bevy 0.20
text widget also needs the application's `bevy_ui_widgets` feature and `TextInputPlugin`.

- **[User guide: learn integration on the real Signalstation game](docs/user-guide.md)**
- [Target contract](docs/api/target.md)
- [Headless implementation status and remaining work](docs/headless-plan/STATUS.md)
- [CLI usage](docs/api/usage.md)
- [Historical implementation notes](docs/api/implementation-plan.md)
- [Historical handoff snapshots](docs/api/next-steps.md)
- [Agreed test-suite redesign and overall acceptance plan](docs/testing-plan.md)
- [Playable collection game and shared test app](test_app/README.md)
- [Standalone station model zoo](test_zoo/README.md)

## Build and run

From this repository's root:

```sh
cargo build --locked --features cli --bin woodpecker
cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app
target/debug/woodpecker --address 127.0.0.1:4100 \
  server start --artifact-dir target/session-artifacts
```

The [user guide](docs/user-guide.md) first introduces Signalstation as a normal game,
then explains its actual optional Woodpecker integration and how to transfer that
boundary to your own game. Start the same game in another terminal:

```sh
target/debug/woodpecker --address 127.0.0.1:4100 \
  session create --config examples/game_tests/app.toml
target/debug/woodpecker --address 127.0.0.1:4100 session ls
```

The [walkthrough](docs/api/usage.md#ausführen) continues with Warp, Inspect and shutdown.
The server and CLI currently require Unix process groups.

## Rust integration and features

The library exposes `command`, `handle`, `session` and report configuration.
Use `session::Plugin` in the game and `session::Session` for direct process control.
The default library has no HTTP/CLI dependencies and does not install or require a
renderer. It uses Bevy Render types to guard an application-installed RenderApp:
extraction and rendering begin after the first completed explicit tick, then
continue while simulation is paused. Startup never runs a hidden warm-up tick.

The optional `session::ThroughputPlugin` retains render-asset notifications across
larger maximum-speed Warp batches. Register additional GPU asset types explicitly;
see [Warp throughput and performance diagnostics](docs/performance.md).

| Feature | Entry points |
| --- | --- |
| `ui` | Virtual legacy UI interaction and focused text input |
| `screenshot` | GPU readback and sandboxed PNG output in rendered applications |
| `offscreen` | Image capture and logical headless viewport; qualified Signalstation/Metal profile; includes `screenshot` |
| `server` | Local session management and HTTP/WebSocket serving |
| `client` | Network management and fixed-session access |
| `cli` | The `woodpecker` executable |

The opt-in [`session::OffscreenTarget` adapter](docs/headless-plan/SLICE.md) selects
an application-owned image instead of the primary window. It preserves RGB PNGs,
path isolation and explicit ticks. The additional [Signalstation L1 integration](docs/headless-plan/L1.md)
connects launch configuration, image readiness, real assets, keyboard/pointer/text,
UI and shutdown without a native window. Use the [headless launch guide](docs/api/usage.md#signalstation-headless-auf-metal);
this is a qualified Metal profile, not general platform or effects support.

## Tests

The [acceptance guide](tests/README.md) is the executable entry point. There are
102 product tests: 101 local cases (including three explicit headless owners) and
one separately authorized real GitHub case. The native local run selects 98
product cases plus its deliberate negative control; the headless L1 run selects
13 cases separately, including the ten unchanged original input owners.
The separate [eight-case offscreen SDK proof](tests/fixtures/offscreen/README.md)
requires explicit local GPU permission and is not an automatic acceptance/CI run.
The common local run also checks the existing deliberate negative control and
the feature/build matrix. Harness checks are separate, not extra product cases.

```sh
python3 -m tests.acceptance --list             # plan only; no builds or processes
python3 -m tests.acceptance --non-graphical    # passing subset still exits 2
```

The native feature baseline and scoped headless L1 run passed; external GitHub
verification was not rerun. [L1 evidence](tests/headless/evidence/README.md) and the
[status plan](docs/headless-plan/STATUS.md) record scope and remaining work.
Rendered runs require explicit scoped permission. Native runs need an awake
desktop and visible windows; headless runs still use a real GPU but no native
window. No silent headless replacement or automatic retry is used. The
[game-control examples](examples/README.md) start with REPL, CLI and fixed scripts,
then add Python feedback, agent handoff and replay. Advanced recipes cover
movement and Unicode text checks; all examples are separate from acceptance. The bounded
[mutation campaign](tests/mutations.md) checks four existing windowless contracts
in isolated source copies, without claiming universal fault detection.

## Origin

Inspired by [ThePrimeagen's game-development video](https://www.youtube.com/watch?v=tYQyh1tjSFc).
The existing [GitHub repository URL](https://github.com/timjonaswechler/bug_hunter)
and versioned report signature identifiers remain unchanged until the coordinated rename.
