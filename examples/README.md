# Control your game with Woodpecker

Use these how-tos to control the existing Signalstation game in `test_app/`.
Start with one small task: move the player right, release the key and check that
the player stops. Then choose the workflow you need.

## Choose a task

1. [Move the player from the REPL](game_tests/control.md#1-move-the-player-from-the-repl).
2. [Send individual CLI commands](game_tests/control.md#2-move-the-player-with-individual-cli-commands).
3. [Run a movement sequence from JSON files](game_tests/control.md#3-run-the-movement-from-json-scripts).
4. [Choose the next move from game state in Python](game_tests/control.md#4-choose-the-next-move-in-python).
5. [Let your coding agent choose the next move](game_tests/agent.md).
6. [Replay a recorded movement in a fresh session](game_tests/control.md#6-replay-a-recorded-movement).

For more detailed automation, [check movement, enter Unicode text or replay
inputs with the supplied Python programs](game_tests/README.md).

## Before you start

[Build the game and start a local server](game_tests/README.md#prepare-the-game-and-server).
The how-tos use real game windows and isolated settings. They do not replace
Signalstation's movement, UI or other gameplay with test logic.

To connect your own game first, follow the
[Signalstation integration guide](../docs/user-guide.md). For command formats and
error meanings, use the [API reference](../docs/api/usage.md).
