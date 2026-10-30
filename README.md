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

- **[User guide: integrate your own game and use Woodpecker](docs/user-guide.md)**
- [Target contract](docs/api/target.md)
- [CLI usage](docs/api/usage.md)
- [Historical implementation notes](docs/api/implementation-plan.md)
- [Historical handoff snapshots](docs/api/next-steps.md)
- [Agreed test-suite redesign and overall acceptance plan](docs/testing-plan.md)
- [Playable collection game and shared test app](test_app/README.md)
- [Standalone station model zoo](test_zoo/README.md)

## Build and run

From this repository's root:

```sh
cargo build --features cli --bin woodpecker
target/debug/woodpecker --address 127.0.0.1:4100 \
  server start --artifact-dir target/session-artifacts
```

For your own game, follow the [integration and first-session guide](docs/user-guide.md).
It includes the game-side plugin setup and a complete launch TOML. Once configured,
in another terminal:

```sh
target/debug/woodpecker --address 127.0.0.1:4100 \
  session create --config path/to/application.toml
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

| Feature | Entry points |
| --- | --- |
| `ui` | Virtual legacy UI interaction and focused text input |
| `screenshot` | GPU readback and sandboxed PNG output in rendered applications |
| `server` | Local session management and HTTP/WebSocket serving |
| `client` | Network management and fixed-session access |
| `cli` | The `woodpecker` executable |

## Tests

The [acceptance guide](tests/README.md) is the executable entry point. There are
94 product tests: 93 local cases and one separately authorized real GitHub case.
The common local run also checks the existing deliberate negative control and
the feature/build matrix. Harness checks are separate, not extra product cases.

```sh
python3 -m tests.acceptance --list             # plan only; no builds or processes
python3 -m tests.acceptance --non-graphical    # passing subset still exits 2
```

The latest complete local run and separate real GitHub verification passed.
The [test plan](docs/testing-plan.md#full-stop-abnahme-und-offene-checkpoints)
records their exact source state, evidence and remaining work; this is not a
claim that every future run or all possible application behavior is verified.
Rendered runs require explicit scoped permission, an awake desktop and visible
windows. No headless replacement or automatic retry is used. The
[game-control examples](examples/README.md) start with REPL, CLI and fixed scripts,
then add Python feedback, agent handoff and replay. Advanced recipes cover
movement and Unicode text checks; all examples are separate from acceptance. The bounded
[mutation campaign](tests/mutations.md) checks four existing windowless contracts
in isolated source copies, without claiming universal fault detection.

## Origin

Inspired by [ThePrimeagen's game-development video](https://www.youtube.com/watch?v=tYQyh1tjSFc).
The existing [GitHub repository URL](https://github.com/timjonaswechler/bug_hunter)
and versioned report signature identifiers remain unchanged until the coordinated rename.
