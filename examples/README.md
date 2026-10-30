# Game-control examples

Start with [one small task, six ways to control it](game_tests/control.md): move
right, then release. Simple control comes before comprehensive game checks.

1. [REPL](game_tests/control.md#1-you-decide-the-repl): enter one command at a time.
2. [CLI](game_tests/control.md#2-you-decide-individual-cli-calls): submit and observe individual commands.
3. [Scripts](game_tests/control.md#3-a-file-decides-the-sequence-native-scripts): run a fixed sequence from JSON files.
4. [Python](game_tests/control.md#4-your-program-decides-python-feedback): choose the next action from current position.
5. [Agent](game_tests/agent.md): hand the same public tools to a bounded agent loop.
6. [Replay](game_tests/control.md#6-a-recording-decides-replay): repeat recorded inputs in a fresh session.

Then explore the [advanced public-CLI recipes](game_tests/README.md): per-tick
movement assertions, real Unicode text editing and recording/replay isolation.

All examples use Signalstation-specific reflected data and a server you manage.
[Build/server setup and graphical constraints](game_tests/README.md#run-from-the-repository-root)
apply to every path. These are examples, not full product acceptance; the
[acceptance suite](../tests/README.md) owns that coverage and the
[test plan](../docs/testing-plan.md) owns its execution evidence.
