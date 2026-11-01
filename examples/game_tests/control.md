# Move the player with Woodpecker

Move Signalstation's player right, release the key and check that it stops.
Choose the REPL for interactive use, individual CLI calls for tool integration,
JSON scripts for a known sequence, or Python for decisions based on game state.
You can also hand the task to your agent or replay recorded inputs.

These how-tos use the real game in `test_app/`. They read `Player.position` and
completed warps, not a game-side tick counter. Input commands queue input.
An explicit tick applies it, and a pressed key stays held until released.

## Before you start

[Build the game and start a server](README.md#prepare-the-game-and-server).
Use the second terminal and its `ADDRESS` variable and `wp` function throughout
this page. The shared setup also explains the real-window requirements.

For sections 1–3, create a fresh session as described below. Python feedback and
replay create their own sessions. For agent control, follow the separate
[handoff steps](agent.md).

## Prepare a fresh session for manual control

1. Create a session:

   ```sh
   wp session create --config examples/game_tests/app.toml
   ```

   Replace `FULL_ID_FROM_CREATE` with the complete `id` returned by this call:

   ```sh
   ID=FULL_ID_FROM_CREATE
   wp session inspect "$ID"
   ```

2. Repeat `session inspect` until `state` is `EngineReady`. Stop on an error or
   `Failed`, or after 90 seconds without readiness. The configuration uses
   isolated settings and limits technical startup to 3600 ticks / 60 seconds.
   Do not repeat a create whose result is uncertain.

3. Check the game's own readiness in the REPL:

   ```sh
   wp session repl "$ID"
   ```

   Enter this line and wait for its matching `completed` result:

   ```text
   inspect resource test_app::level::LevelLoad
   ```

   If the value is `Loading`, enter `tick warp 1`, wait for completion and inspect
   again. Stop on any other value or after 120 preparation ticks without `Ready`.
   Technical `EngineReady` alone does not mean the level is ready.

4. Once `LevelLoad` is `Ready`, stay in the REPL for section 1. For sections 2–3,
   enter `quit` to return to the shell. `quit` disconnects the REPL without
   stopping the session.

Preparation is separate from the movement task. After each manual task,
[stop that session](#stop-a-manual-session) before creating another one.

## 1. Move the player from the REPL

**Goal:** move while holding right for four ticks, then verify one still tick.

1. Use a fresh, prepared session. Enter these lines **one at a time**. After each
   game command, wait for `completed` with the matching request ID and command name.
   `pending` confirms acceptance, not completion.

   ```text
   inspect resource test_app::player::Player
   command {"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}
   tick warp 4
   inspect resource test_app::player::Player
   command {"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}
   tick warp 1
   inspect resource test_app::player::Player
   quit
   ```

2. Compare the three positions. After the four-tick warp, `position[0]` must
   have increased and `position[1]` must be unchanged. After release and one
   more tick, the position must be identical to the second reading.
3. Check that the warp results report four and one executed ticks, then
   [stop the session](#stop-a-manual-session).

## 2. Move the player with individual CLI commands

**Goal:** submit each action from a shell or another tool and wait for its result.
Use a fresh, prepared session and leave its REPL with `quit`.

1. Read the initial position:

   ```sh
   wp session submit "$ID" --command '{"command":"inspect.query","arguments":{"source":"resources","selector":{"kind":"type","type_path":"test_app::player::Player"},"projection":{"kind":"value"}}}'
   wp session poll "$ID" --wait-ms 1000
   ```

   `submit` returns a request ID, for example:

   ```json
   {"kind":"pending","request_id":7,"command":"inspect.query"}
   ```

   In the poll response's `entries`, find the event whose `request_id` and
   `command` match your submission and whose `kind` is `completed`.
   For this query, the position is at
   `event.output.items[0].result.value.value.position`. Require
   `event.output.items[0].result.value.status` to be `readable` before using it.

2. If the result has not arrived, poll again. Replace the placeholder below
   with the **entire `cursor` JSON object** from the preceding poll reply:

   ```sh
   wp session poll "$ID" --cursor 'CURSOR_JSON_FROM_PREVIOUS_REPLY' --wait-ms 1000
   ```

   Continue with each returned cursor. Do not repeat `submit` to wait for a
   result. Stop on `rejected`, `failed`, an Activity Gap, an unknown outcome or
   a 10-second command deadline. Use this submit-and-wait procedure for every
   action and query below.

3. Press right, then advance four ticks. Submit these separately and wait for
   each matching completion before continuing:

   ```sh
   wp session submit "$ID" --command '{"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}'
   # Poll until this press completes.
   wp session submit "$ID" --command '{"command":"tick.warp.start","arguments":{"ticks":4}}'
   # Poll until this warp completes.
   ```

4. Repeat the position query from step 1. Check that x increased and the other
   position coordinate stayed unchanged. Keep the key held until this reading
   is complete. Then release and advance one tick:

   ```sh
   wp session submit "$ID" --command '{"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}'
   # Poll until this release completes.
   wp session submit "$ID" --command '{"command":"tick.warp.start","arguments":{"ticks":1}}'
   # Poll until this warp completes.
   ```

5. Query the position once more. It must match step 4. Check the two warp
   results for four and one executed ticks, then [stop the session](#stop-a-manual-session).

Inspect does not advance the simulation. For the command schema and error codes,
see the [API reference](../../docs/api/usage.md).

## 3. Run the movement from JSON scripts

**Goal:** run a fixed movement sequence without submitting each action yourself.
Use a fresh, prepared session. Read its initial position using section 2.

1. Run the held-key part:

   ```sh
   wp session script "$ID" --file examples/game_tests/hold-right.json
   ```

   Continue only if the command exits 0 and reports `kind: "passed"`.
   Query the position again. Its x coordinate must have increased, with the
   other position coordinate unchanged.

2. Release the key and check stillness:

   ```sh
   wp session script "$ID" --file examples/game_tests/release-right.json
   ```

   Again require exit 0 and `kind: "passed"`. Query the position. It must be
   unchanged from step 1. Check that the warps report four and one ticks.
3. [Stop the session](#stop-a-manual-session).

[hold-right.json](hold-right.json) contains press and a four-tick warp.
[release-right.json](release-right.json) contains release and a one-tick warp.
Keep them separate: ordinary commands inside one script are pipelined, so a
release after a warp in the same file could run before that warp finishes.
The successful end of the first invocation creates the required boundary.
Do not run the second file if the first fails. Scripts do not stop the session
or branch on observed game state.

## 4. Choose the next move in Python

**Goal:** move until x has advanced by at least 0.2 world units, then stop.
No manually created session is needed.

1. With your server running, execute:

   ```sh
   python3 examples/game_tests/feedback.py --address "$ADDRESS" --rendered
   ```

2. Read the `PASS` line. It includes the initial and final x coordinates and
   the number of moving ticks. It appears only after the program has checked
   one released, still tick and ended its own session.
3. To adapt this to another task, open [feedback.py](feedback.py). Replace the
   resource paths and target condition with your game's state and goal.
   Keep the pattern: observe → act once → wait for completion → observe again.

The program presses right once and checks position after each tick. It releases
as soon as it reaches the target, allowing at most four moving ticks. A blocked
move or an unmet target fails rather than repeating actions. It does not query
`SimulationTicks` or require new instrumentation in the game's movement system.
[_cli.py](_cli.py) handles CLI transport and result matching, not game decisions.

## 5. Let your agent choose the next move

[Prepare a bounded agent task](agent.md) using the same commands as section 2.
You supply the coding agent and its local tools. Woodpecker does not include a
model, agent runtime or required model-service API.

## 6. Replay a recorded movement

**Goal:** repeat recorded inputs in a fresh session with the same starting state.
No manually created session is needed.

1. Run:

   ```sh
   python3 examples/game_tests/replay.py --address "$ADDRESS" --rendered
   ```

2. Require `PASS` confirming the same final position and five-tick delta in
   both sessions, with both sessions ended. See the
   [recording and replay how-to](README.md#record-inputs-and-replay-them-in-a-fresh-session)
   to adapt the program.

Replay repeats recorded commands. It does not reset the game, restore a snapshot
or choose new actions when the starting state differs.

## Stop a manual session

Run this after a manual task, including when it fails. Stop only the full ID
returned by your own create:

```sh
wp session stop "$ID"
wp session inspect "$ID"
```

Repeat the read-only inspection until `Ended`, for at most 40 seconds.
`Failed`, an error or an unknown outcome is not successful cleanup. Do not retry
an uncertain stop, guess another ID or kill unrelated processes. Keep the
`artifact_dir` from inspection for diagnosis.

When you have finished all tasks, you can [stop your server](README.md#stop-your-server).
For UI text input and per-tick movement checks, continue with the
[Python how-tos](README.md).
