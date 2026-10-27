# Game-testing examples

Start with [the three public-CLI recipes](game_tests/README.md):

- [Movement](game_tests/movement.py): press, held ticks, release and stillness.
- [Text](game_tests/text.py): find/focus a real widget, enter Unicode, tick, inspect.
- [Replay](game_tests/replay.py): record a short input sequence and replay it in a fresh session.

These examples use Signalstation-specific reflected data and a server you manage.
They do not import the project's test helpers or certify full product acceptance.
Build/server commands, exact app/tick budgets, scoped graphical permission and
owned-session cleanup are described in the linked guide. Keep the desktop awake
and the game windows partly visible before starting. No automatic retry or
headless replacement.

The [test plan](../docs/testing-plan.md) owns execution evidence and progress;
the [acceptance suite](../tests/README.md) owns comprehensive feature coverage.
