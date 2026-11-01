# Automate movement, text input and replay

Run a small Python program against Signalstation to check held-key movement,
enter text in its existing UI or repeat recorded inputs in a fresh session.
Each program uses the public Woodpecker CLI and reports whether its task succeeded.
For commands you enter yourself, use the [REPL and CLI how-tos](control.md).

## Prepare the game and server

You need the repository's Rust toolchain, Python **3.11+**, a renderer and a display.
All commands below run from the repository root in a Unix shell.

**These examples open real game windows.** When an agent runs them, authorize
that graphical run first. Keep the display awake and each game window partly
visible. Do not resize it, change scaling, move it between displays or send
native gameplay input during a controlled session. `--rendered` acknowledges
these conditions but does not grant permission. [Run budgets](#run-budgets)
list the app starts and actions for each program.

1. Build the CLI and the game with its optional Woodpecker integration:

   ```sh
   cargo build --locked --features cli --bin woodpecker
   cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app
   ```

2. In terminal 1, choose a free loopback address and start your server:

   ```sh
   target/debug/woodpecker --address 127.0.0.1:4100 \
     server start --artifact-dir target/game-example-sessions
   ```

   Wait for the JSON line with `"state":"Ready"`. Keep this terminal open.
   If the address is already in use, choose another one and use it consistently.

3. In terminal 2, also from the repository root, set the same address:

   ```sh
   ADDRESS=127.0.0.1:4100
   wp() { target/debug/woodpecker --address "$ADDRESS" "$@"; }
   ```

   The `wp` function only saves typing. It does not create a session or send
   extra commands. Choose **one** task below. Its Python program creates and
   prepares its own sessions, so do not create one manually for it.

## Check movement while a key is held

**Goal:** verify that a single key press keeps moving the player until release.

1. Run the program without Python's `-O` option:

   ```sh
   python3 examples/game_tests/movement.py --address "$ADDRESS" --rendered
   ```

2. Look for `PASS: four moving ticks and four still ticks; owned session ended.`

The program reads `Player.position`, presses `arrow_right` once and checks that
x increases on each of four ticks. It releases the key and verifies four further
ticks without movement. The other position coordinate must stay unchanged.
Use [movement.py](movement.py) as a starting point when you need per-tick checks.

## Enter Unicode text through the existing UI

**Goal:** put `Grüße 🦉`, a newline and `東京` into Signalstation's Notes field.

1. Run:

   ```sh
   python3 examples/game_tests/text.py --address "$ADDRESS" --rendered
   ```

2. Look for `PASS: exact Unicode Settings.notes after eight action ticks; owned session ended.`

[text.py](text.py) finds the Settings button by its displayed text
`Einstellungen [M]`, then finds the existing input named `Notizen`. It computes
click coordinates from the widgets' current layout rather than hard-coding them.
After focusing Notes, it submits the text and advances two ticks: one for Bevy's
edit pass and one for Signalstation to copy the edit into `Settings.notes`.
The program checks the exact text and that all other settings stay unchanged.

## Record inputs and replay them in a fresh session

**Goal:** reproduce the same final position from the same starting state.

1. Run:

   ```sh
   python3 examples/game_tests/replay.py --address "$ADDRESS" --rendered
   ```

2. Look for `PASS: fresh B matched A's final position and five-tick delta; both owned sessions ended.`

[replay.py](replay.py) records press → four ticks → release → one tick in session A.
It closes the recording and stops A before creating B. It copies only that
recording into B's reported artifact directory, replays it, then compares the
final position and five-tick delta. Replay does not restore a world snapshot.
Use this pattern only when you can establish the required starting state yourself.

## Adapt a program to your game

1. [Integrate Woodpecker into your existing game](../../docs/user-guide.md#6-diesen-integrationsweg-auf-dein-eigenes-spiel-übertragen).
   Keep its normal input, movement and UI systems.
2. Edit [app.toml](app.toml): replace the manifest, package, target, features and
   game arguments. The manifest path resolves from the **server's working
   directory**. Relative game arguments resolve from the manifest directory.
   For Signalstation, that is why the seed is `../examples/game_tests/settings.json`.
3. Replace the Signalstation resource paths in [_cli.py](_cli.py) and the program
   with your game's reflected state. Choose a real readiness condition and a
   success condition for your task.
4. For text input, replace the button label and input name in `text.py`. Keep
   layout-based coordinate discovery and use the edit/synchronization ticks
   that your UI actually requires.
5. For a state-driven example without a tick counter, start with
   [feedback.py](feedback.py) and its [how-to](control.md#4-choose-the-next-move-in-python).
   The three programs above also read Signalstation's optional `SimulationTicks`
   measurement resource. Replace or remove those extra measurement assertions
   when adapting them. Do not add a counter to your movement system.

The helpers in `_cli.py` invoke the public CLI and match submitted requests to
results. They are example code, not a public Python SDK. Command behavior and
limits are defined in the [API contract](../../docs/api/target.md).

## Find results and handle a failure

A program prints `PASS` and exits 0 only after its checks and session cleanup
succeed. Each creates its own sessions and stops only those full IDs. The server
stays running. Run programs one at a time and do not let another client control
their sessions.

- Read the printed `Client journal` directory for commands, results and errors
  in `cli.jsonl`. Session files remain in the printed `Session artifacts` directory.
- If readiness fails, inspect the session error and its logs. Do not keep adding
  ticks until the expected answer appears.
- If a command is rejected, fails, times out or loses its result through an
  Activity Gap, stop the task. Do not resubmit an input just to wait for its result.
- If a create or stop has an unconfirmed outcome, retain the logs and resolve it
  before another attempt. Do not guess an ID or claim successful cleanup.

To see options without starting a server or game, run any program with `--help`:

```sh
python3 examples/game_tests/text.py --help
```

These programs show how to automate specific tasks. A successful run is not a
full product acceptance result. The separate [product suite](../../tests/README.md)
covers Woodpecker's contracts.

## Stop your server

Once all your tasks have finished, stop the server **only if you started it and
no other sessions need it**:

```sh
wp server stop
```

Wait for the foreground server in terminal 1 to exit. This stops the server and
any remaining sessions, not just the last example. Logs and recordings remain
on disk.

## Run budgets

Each session uses [app.toml](app.toml) and an isolated copy of
[settings.json](settings.json), not your personal settings. It opens one
unfocused **1440 × 900** window at scale 1.

| Program | Serial app starts | Actions after game readiness |
| --- | ---: | --- |
| `movement.py` | 1 | One press, four moving ticks, one release, four still ticks: **8 ticks**. |
| `text.py` | 1 | Move/press/release for each of two UI clicks, each followed by a tick, then one text input and two ticks: **8 ticks**. |
| `replay.py` | 2 | **5 ticks** and two keyboard commands recorded in A, then the same inputs and **5 ticks** replayed in B. A ends before B starts. |
| `feedback.py` | 1 | One press, at most four moving ticks, one release and one still tick: **at most 5 ticks**. |

In addition, each session allows up to **3600 engine-startup ticks / 60 seconds**.
The programs wait at most 90 seconds for `EngineReady`, then request at most
**120 explicit preparation ticks**, checking `LevelLoad` and a 10-second deadline
before another tick. Commands have 10-second result deadlines. Cleanup allows
40 seconds per session, including a stop call capped at five seconds.
These are individual bounds, not a guaranteed total running time.
