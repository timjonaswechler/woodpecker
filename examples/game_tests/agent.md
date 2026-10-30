# Let an agent choose the next action

Start with [individual CLI control](control.md#2-you-decide-individual-cli-calls).
The agent uses exactly those tools; Woodpecker supplies neither a model nor an
agent runtime. You supply the coding agent and decide which local tools it may
use. No model credentials belong in game commands, recordings or journals.

## Prepare the handoff

1. Obtain permission for this graphical run. Start your local server and create
   **one fresh session** using the [shared setup](control.md#setup-once).
2. Wait for `EngineReady`, explicitly prepare until `LevelLoad` is `Ready`,
   then leave the REPL with `quit`. Do not keep another controlling client active.
3. Record the full confirmed ID, absolute repository root, loopback address and
   current `Player.position` and `SimulationTicks.ticks`. Give those to the agent.
   Only that ID is in scope; do not ask it to discover sessions by listing.
4. Replace all angle-bracket placeholders in the prompt below. It authorizes
   only the remaining movement and cleanup of this already-created session.

## Copy-ready task

```text
Use the local public Woodpecker CLI from <absolute repository root>:
target/debug/woodpecker --address <loopback address>

You control only session <full confirmed session ID>, already EngineReady with
LevelLoad Ready, no held keys and no other controller. Initial Player.position:
<initial position>; initial SimulationTicks.ticks: <initial tick count>.

Goal: move right until position[0] >= initial position[0] + 0.2 world units,
then release and demonstrate one still tick. Allow at most four moving ticks
and one released tick. No other inputs, startup ticks, reset, session creation,
model-service calls or server shutdown are authorized.

Use session submit and session poll. For every submission, remember its
request_id and command; wait for its matching completed event before acting.
Continue polls with the returned cursor. Each command has a 10-second deadline.
Pending is not success. Never resubmit an unknown outcome; stop on a Gap,
rejection, failure, timeout, or an unreadable/unexpected resource.

Observe Player with inspect.query:
{"command":"inspect.query","arguments":{"source":"resources","selector":{"kind":"type","type_path":"test_app::player::Player"},"projection":{"kind":"value"}}}
Read SimulationTicks the same way using test_app::woodpecker::SimulationTicks.

At every decision, report: current x, target x, ticks used, next action and why.
First verify the supplied initial state with fresh read-only queries. Press
arrow_right once with input.keyboard.press, then use tick.warp.start {"ticks":1}
and query position after each completed tick. Keep the key held between ticks.
Stop moving as soon as the goal is reached. Fail if x does not increase or the
other position coordinate changes; do not keep ticking into a blocked path.
If four moving ticks are exhausted without reaching the goal, report failure.

Release arrow_right with input.keyboard.release and explicitly tick once;
verify unchanged position and total tick delta = moving ticks + 1.
On failure do not invent success or blindly repeat inputs. In all cases stop
only this owned session with session stop, once, and observe session inspect
for Ended for at most 40 seconds. Failed or unknown cleanup remains an error.
Never kill processes or stop the server. Keep artifact_dir and command evidence.

Report success only if the goal, stillness, tick count and Ended were observed.
State that this proves the small movement task, not general autonomous gameplay.
```

This uses structured game state so decisions are easy to check. Screenshot-based
decisions would additionally need screenshot capability, a visual goal and their
own evaluation; a picture alone cannot prove tick counts or released input.
