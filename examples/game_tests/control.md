# Control a game, one step at a time

Move Signalstation's player right, then release the key. Start with the REPL;
only add automation when you need it. These examples use the public CLI, not
native keyboard input. A successful input command queues input; only an explicit
simulation tick applies it. A key stays held until released.

## Setup, once

Python 3.11+ and the repository's Rust toolchain are required. Run from the
repository root. [Build the two binaries and start your local server](README.md#run-from-the-repository-root)
using steps 1–2, **not** the three advanced recipes in step 3.
Obtain permission for real game windows before running; keep the display awake
and windows partly visible. Do not resize, change scaling or send native input.

In your second terminal, define a shorthand that only forwards CLI arguments:

```sh
ADDRESS=127.0.0.1:4100
wp() { target/debug/woodpecker --address "$ADDRESS" "$@"; }
```

### A fresh session for each manual example

```sh
wp session create --config examples/game_tests/app.toml
```

Copy the returned **full** `id` into `ID` (replace the placeholder):

```sh
ID=FULL_ID_FROM_CREATE
wp session inspect "$ID"
```

Observe `session inspect` until `state` is `EngineReady`; stop on an error or
`Failed`, or if readiness is still unconfirmed after 90 seconds. The config caps
technical startup at 3600 ticks / 60 seconds and seeds isolated settings. Never
reuse an unknown ID or resubmit a create whose outcome is uncertain.

Technical readiness is not game readiness. Open the REPL:

```sh
wp session repl "$ID"
```

Enter one line at a time and wait for its matching `completed` result:

```text
inspect resource test_app::level::LevelLoad
```

If the readable value is `Loading`, explicitly run `tick warp 1`, wait for
completion and inspect again. Stop on any other value or after 120 preparation
ticks without `Ready`. These are preparation ticks, not part of the movement
below. There are no hidden warmup ticks. Once `Ready`, stay in the REPL for
example 1, or enter `quit` for examples 2–3. `quit` disconnects, it does not stop
the game.

### Finish each manual example

Stop **only** the session whose create returned your `ID`:

```sh
wp session stop "$ID"
wp session inspect "$ID"
```

Observe until `Ended`, for at most 40 seconds. `Failed`, an error or an unknown
outcome is not successful cleanup. Do this on failure too; do not retry an
uncertain stop or kill unrelated processes. Keep the `artifact_dir` reported by
inspect for diagnostics. Finish cleanup before starting the next example.

## 1. You decide: the REPL

Use a fresh, prepared session. Enter these lines **one at a time**, waiting for
the matching `completed` event after each command. `pending` is not completion.

```text
inspect resource test_app::player::Player
inspect resource test_app::woodpecker::SimulationTicks
command {"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}
tick warp 4
inspect resource test_app::player::Player
command {"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}
tick warp 1
inspect resource test_app::player::Player
inspect resource test_app::woodpecker::SimulationTicks
quit
```

**Check:** `Player.position[0]` increased after four ticks; `position[1]` stayed
unchanged. The position after the release tick is identical to the previous
position. `SimulationTicks.ticks` increased by five. Stop the session as above.
The REPL observes results for you; it does not decide what to do next.

## 2. You decide: individual CLI calls

Use a fresh, prepared session and leave its REPL with `quit`. First read initial
position and ticks. Submit each query separately and poll it to completion:

```sh
wp session submit "$ID" --command '{"command":"inspect.query","arguments":{"source":"resources","selector":{"kind":"type","type_path":"test_app::player::Player"},"projection":{"kind":"value"}}}'
wp session poll "$ID" --wait-ms 1000
# After the position query completes:
wp session submit "$ID" --command '{"command":"inspect.query","arguments":{"source":"resources","selector":{"kind":"type","type_path":"test_app::woodpecker::SimulationTicks"},"projection":{"kind":"value"}}}'
wp session poll "$ID" --wait-ms 1000
```

`submit` returns a `request_id`, **not** the outcome. In the poll's `entries`,
find the `completed` event with that request ID and command name. If it is still
pending, poll again with `--cursor` set to the previous reply's entire `cursor`
JSON object. Never repeat the submission to wait for it. Stop on `rejected`,
`failed`, an Activity Gap, an unconfirmed outcome or a 10-second command deadline.

For example, replace this cursor placeholder with the actual object:

```sh
wp session poll "$ID" --cursor 'CURSOR_JSON_FROM_PREVIOUS_REPLY' --wait-ms 1000
```

Now submit these **separately**, polling each matching completion before the
next submission:

```sh
wp session submit "$ID" --command '{"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}'
# Poll to completion.
wp session submit "$ID" --command '{"command":"tick.warp.start","arguments":{"ticks":4}}'
# Poll to completion before releasing: the key must stay held for all four ticks.
wp session submit "$ID" --command '{"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}'
# Poll to completion.
wp session submit "$ID" --command '{"command":"tick.warp.start","arguments":{"ticks":1}}'
# Poll to completion.
```

Repeat the position query after the four-tick warp **before** releasing, and
again after the release tick. Repeat the tick query at the end. Queries do not
advance the simulation.

**Check:** the same increased x, unchanged second coordinate, still release tick
and five-tick delta as example 1. Stop the session. Choose individual CLI calls
when another tool needs explicit control of submission and observation.

## 3. A file decides the sequence: native scripts

Use a fresh, prepared session. Read initial position and ticks as above. Then:

```sh
wp session script "$ID" --file examples/game_tests/hold-right.json
# Continue only after exit 0 and kind "passed".
# Read position here: x must have increased, the other coordinate must not change.
wp session script "$ID" --file examples/game_tests/release-right.json
# Continue only after exit 0 and kind "passed". Read position and ticks again.
```

[hold-right.json](hold-right.json) presses right and advances four ticks.
[release-right.json](release-right.json) releases and advances one tick.
Each CLI invocation waits for all its own results before returning.

**Why two files?** Ordinary commands *within* a script are pipelined, not
awaited one by one. Putting release immediately after a warp in the same list
could release while the warp is still running. The boundary between these two
successful invocations keeps the key held for the entire first warp. Do not
run the second script if the first fails.

**Check:** five total ticks and no movement during the second script. Stop the
session. Scripts have no implicit session shutdown. A fixed sequence is useful
when you already know every action; it cannot branch on a query result.

## 4. Your program decides: Python feedback

[feedback.py](feedback.py) starts its own fresh session. It reads x, chooses
right, advances **one** tick and observes again. Once x has advanced by at least
0.2 world units it releases, ticks once and checks stillness. It allows at most
four moving ticks; blocked movement or an unmet target fails, not retries.

```sh
python3 examples/game_tests/feedback.py --address "$ADDRESS" --rendered
```

**Check:** `PASS` includes initial/final x and the number of moving ticks, and is
printed only after owned-session cleanup. Unlike a fixed script, the next action
depends on current state. The small existing [_cli.py](_cli.py) adapter handles
transport and result correlation; every input and tick stays visible in the
example. It is not a public Python SDK.

## 5. An agent decides: hand off the same tools

Follow the [bounded agent handoff](agent.md). Give your coding agent one freshly
created and prepared session's full ID and the CLI commands from example 2.
It observes, chooses a single next action, waits for completion and observes
again. This is your agent's tool loop, **not** a built-in Woodpecker agent or a
required model service. The example adds no model dependency or paid API call.

## 6. A recording decides: replay

Run the existing [fresh-session replay recipe](replay.py):

```sh
python3 examples/game_tests/replay.py --address "$ADDRESS" --rendered
```

It owns both sessions: record press → four ticks → release → one tick in A,
stop A, copy only the closed recording, then replay in fresh B with the same
initial state. **Check:** `PASS` means final position and five-tick delta match,
and both sessions ended. Replay follows recorded commands; it does not adapt
to a different starting state or make new decisions.

## Next: stronger game checks

The [advanced recipes](README.md) check each individual held tick, discover and
edit a real Unicode text widget, and explain replay isolation in detail. Adapt
Signalstation's reflected resource names and movement expectations to your own
game; Woodpecker does not define your game's readiness or success condition.
