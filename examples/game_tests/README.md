# Advanced game-testing recipes

**New to controlling a game?** Start with the [step-by-step control guide](control.md):
REPL → individual CLI calls → fixed scripts → Python feedback → agent handoff →
replay. Each adds one idea to the same small movement task; this page covers the
more detailed checks and their shared build/server setup.

Three standalone Python **3.11+** recipes for game developers, using only the
[public CLI](../../docs/api/usage.md) and standard library. These are not another
acceptance suite; the [product tests](../../tests/README.md) own that coverage.

## Run from the repository root

**Obtain scoped permission before each graphical run, including repeats.** These
commands open real Signalstation windows. `--rendered` is a technical acknowledgement,
not permission. Keep the desktop awake and each window partly visible; do not
resize it, change scaling, move it between displays, or use native input.

1. Build the CLI and app with the features used by [app.toml](app.toml):

   ```sh
   cargo build --locked --features cli --bin woodpecker
   cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app
   ```

2. In terminal 1, start an externally managed local server **from this root** and
   wait for its `Ready` line:

   ```sh
   target/debug/woodpecker --address 127.0.0.1:4100 server start --artifact-dir target/game-example-sessions
   ```

3. In terminal 2, run the permitted examples serially:

   ```sh
   python3 examples/game_tests/movement.py --address 127.0.0.1:4100 --rendered
   python3 examples/game_tests/text.py --address 127.0.0.1:4100 --rendered
   python3 examples/game_tests/replay.py --address 127.0.0.1:4100 --rendered
   ```

   Each prints `PASS` only after its assertions and owned-session cleanup succeed.
   Exit 0 means that example passed, not full product acceptance. Any action,
   observation or cleanup failure exits nonzero. Do not use Python `-O` (rejected).
   `python3 examples/game_tests/movement.py --help` (also `text.py` / `replay.py`)
   starts nothing and requires no running server or built binaries.

## Exact graphical scope

Each session starts one unfocused, fixed **1440 × 900** window at scale 1, with
isolated settings from [settings.json](settings.json), not personal settings.

| Example | App starts / windows over the run | Actions after readiness |
| --- | --- | --- |
| `movement.py` | 1 / 1 | One `arrow_right` press; four individually checked held ticks; one release; one still tick, then three more still ticks: **8 ticks**, 2 keyboard commands. |
| `text.py` | 1 / 1 | Discover/click Settings, discover/click the existing Notes widget: move, press, release and one tick each, **6 ticks**. Submit `Grüße 🦉\n東京`, then one edit tick and one app-sync tick: **8 ticks total**, 6 pointer commands and 1 text command. Exact `Settings.notes` and unchanged other settings are asserted. |
| `replay.py` | 2 / 2, serial | A records press, 4 ticks, release, 1 tick: **5 ticks**, 2 keyboard commands. Stop A before creating B. Copy only A's closed recording into B's public `artifact_dir`; B replays those **5 ticks** and inputs. Compare final position and tick delta. |

In addition, **each session** explicitly selects engine startup capped at
**3600 ticks / 60 seconds** in the TOML. Read-only CLI readiness observation is
bounded at 90 seconds after confirmed creation, while launch/startup progress. `EngineReady` is not level readiness:
each script visibly requests at most **120 further one-tick warps**, checking the
app's `LevelLoad` between them and checking a 10-second deadline before another
warp. No preparation ticks are hidden in a helper. Individual command/result
waits are bounded at 10 seconds, so these are not hard total wall-clock promises.

All three commands together start **4 apps / 4 windows**, at most one live at a
time, with **26 action ticks** plus the per-session preparation budgets. There
are no OS inputs, resets, automatic action retries, display changes or hidden
warmups. Replay uses fresh resource queries, never transplanted entity handles;
while Replay is active, its result is observed only through Activity polls.

## Ownership and retained evidence

- Use an idle server address you manage. Do not let another client control these
  example-owned sessions. The scripts never start, stop or kill the server and
  never stop sessions discovered through a list or an ID prefix.
- Every create, input, warp and recording/replay command is visible in its example.
  [_cli.py](_cli.py) only provides CLI subprocess transport/journaling, bounded
  correlated observation, read-only resource queries and full-ID owned cleanup.
- Normal completion explicitly calls public `session stop` for each owned full ID
  and observes `Ended`. Failure cleanup does the same for confirmed creates not
  already stopped, with a **40-second budget per session**, including a stop call
  capped at 5 seconds. It never resubmits an attempted stop or falls back to kills.
  `Failed`, timeout or loss of confirmation is an error, not successful cleanup;
  an original failure is preserved alongside cleanup errors. This observes public
  lifecycle state, not an independent OS-process or report-provider audit.
- A transport failure, Activity Gap or unconfirmed submission fails the example:
  no resubmit, reconnect repair or guessed outcome. A create whose ID was never
  confirmed may have started an app; it cannot be safely claimed or stopped here.
  The server owner must resolve that uncertainty using the retained evidence.
- Client commands, stdout/stderr, preparation counts, outcomes and cleanup stay in
  the printed `target/game-example-runs/<example>-<unique>/cli.jsonl`. Session files
  stay in the absolute `artifact_dir` printed from public `session inspect`.
  Nothing is broadly cleaned or deleted; replay retains both recording copies.

## Adapt to your game

Replace the manifest/package/features and app arguments in `app.toml`. A relative
manifest resolves from the **server's working directory**, not beside the TOML.
The launched app runs in the manifest directory, so this seed argument is
`../examples/game_tests/settings.json` relative to **test_app/**. Keep those two
path bases distinct.

Replace Signalstation's public reflected resource paths (`SimulationTicks`,
`LevelLoad`, `Player.position`, `Settings.notes`) and the expected settings with
your own game oracle. Text discovers a button by child `Text` (`Einstellungen [M]`,
whitespace-normalized), then `TextInput` by public `Name` (`Notizen`), and uses
`ComputedNode` / `UiGlobalTransform` to derive logical-pixel coordinates. Adapt
those queries, layout bounds and explicit edit/sync ticks to your existing UI;
do not invent fixture widgets or tick until an expected answer appears. See the
[Inspect/input/replay contracts](../../docs/api/target.md).
