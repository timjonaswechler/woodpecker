# Session-Laufzeit der Tests

`application.py` liest die Konfiguration unter `tests/application`, prüft Modi
und Fähigkeiten und bestimmt den App-Build. `runtime.py` startet den echten
Server, öffnet Sessions und räumt deren Prozesse und Artefakte auf.

Command-, Interface- und Workflow-Verträge verwenden dieselbe Laufzeit.
Diese Module erzeugen keine impliziten Ticks und wiederholen keine Commands.
Assertions zum Produktverhalten stehen in den jeweiligen Feature-Ordnern.

Modusunabhängige Fehlerszenarien wählen mit `Context.program` ausdrücklich einen
Protokollpartner oder eine Fehlerquelle. Auch diese verwenden denselben Server,
dieselben Sessions und die echten JSON-/REPL-Zugänge. Ein fehlgeschlagener
App-Start wird über die Session-Verwaltung geprüft, bevor ein Zugang bindet.
