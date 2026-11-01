# Let your coding agent move the player

Give your existing coding agent a small game task: observe the player's position,
move right until x has advanced by 0.2 world units, then release and verify that
the player stops. The agent uses the same public CLI as the
[individual-command how-to](control.md#2-move-the-player-with-individual-cli-commands).
Woodpecker supplies the game-control tools, not a model or agent runtime.

## 1. Prepare one session

1. [Build the game and start your local server](README.md#prepare-the-game-and-server).
   Authorize the graphical run if an agent will execute it.
2. [Create and prepare one fresh session](control.md#prepare-a-fresh-session-for-manual-control).
   Wait for `EngineReady` and `LevelLoad: Ready`. Do not send gameplay input yet.
3. Read `test_app::player::Player` in the REPL and record its `position`, then
   enter `quit`. Keep the session running, with no other controller active.
4. Record the absolute repository path, loopback address and full session ID.
   Use only the ID returned by your own create, not an ID found by listing sessions.

## 2. Give the task to your agent

Enable the local shell tool your coding agent needs to invoke the CLI.
Replace **all four angle-bracket placeholders** below with your recorded values,
then paste the task into that agent. You do not need to add a model dependency
to the game or put model credentials in a command, recording or journal.

```text
Work from <absolute repository root>.
Use target/debug/woodpecker --address <loopback address>.
Control only session <full confirmed session ID>.
Its initial Player.position is <initial position array>.
The session is EngineReady, LevelLoad is Ready, no keys are held and no other
client is controlling it. Do not create a session or stop the server.

Goal: increase position[0] by at least 0.2 world units, release arrow_right,
and demonstrate one tick with unchanged position. Allow at most four moving
ticks and one released tick. Do not send other inputs, preparation ticks,
resets or screenshots. Do not change files or add a model-service integration.

Use these CLI operations:
- session inspect SESSION_ID
- session submit SESSION_ID --command 'COMMAND_JSON'
- session poll SESSION_ID --wait-ms 1000
- session poll SESSION_ID --cursor 'CURSOR_JSON' --wait-ms 1000
- session stop SESSION_ID

Replace SESSION_ID with the authorized full ID, COMMAND_JSON with the command
object and CURSOR_JSON with the entire cursor object from the preceding poll.
For each submit, remember request_id and command. Wait for its matching
completed event before the next action. Pending is not success. Continue polls
with the returned cursor and allow at most 10 seconds per command. Never
resubmit to wait for a result. Stop the task on rejection, failure, a Gap,
timeout or an unknown outcome.

First inspect the lifecycle and verify LevelLoad and the supplied position
with fresh read-only queries. Use this command form for each resource:
{"command":"inspect.query","arguments":{"source":"resources","selector":{"kind":"type","type_path":"test_app::player::Player"},"projection":{"kind":"value"}}}
For LevelLoad, replace type_path with test_app::level::LevelLoad.
Require one readable resource value and the expected initial state.

Press once:
{"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}
Then repeatedly submit one tick and read Player.position:
{"command":"tick.warp.start","arguments":{"ticks":1}}
Keep the key held between ticks. Fail if x does not increase or position[1]
changes. Release as soon as x reaches the initial x + 0.2. If four moving
ticks are exhausted without reaching it, report failure.

Release:
{"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}
On success, explicitly tick once more and verify unchanged position. Count
executed_ticks from completed warps, not from a game-side tick counter.
At each decision, report current x, target x, ticks used, next action and why.

On success or failure, stop only this session once and inspect until Ended,
for at most 40 seconds. Do not retry an uncertain stop or kill processes.
Failed or unknown cleanup remains an error. Keep artifact_dir and the command
results for diagnosis. Leave the server running for its owner.

Report success only after observing the goal, one still tick, the expected
executed-tick count and Ended. Otherwise report the failed step and evidence.
```

## 3. Check the result

The agent should report an increased x coordinate, the number of moving ticks,
one released tick without movement and the session's `Ended` state.
A list of submitted commands alone is not evidence that the task completed.
If the agent stops on an error, use its recorded results and artifact directory
to diagnose the problem before another attempt.

This task uses structured game state. For image-based decisions, separately
define the visual goal and enable screenshots. A screenshot alone cannot show
whether the correct number of ticks ran or whether a held key was released.

When finished, [stop your server](README.md#stop-your-server) if you own it and
no other sessions need it.
