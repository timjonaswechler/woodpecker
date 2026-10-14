# Verhaltenssuite

Die Suite prüft Produktverhalten über Session-Modus, Command-Zugang und Feature.
Gemeinsame Verträge laufen über Rendered/Headless und JSON/REPL. Die gemeinsame
Bevy-App ist das [Sammelspiel](../test_app/README.md); ihre [Anbindung](application/README.md)
verwendet `application/app.json`. Fehlende Modi oder Fähigkeiten ergeben keine bestandene Gesamtsuite.

## Einstieg

Python 3.11 oder neuer und die Rust-Toolchain verwenden. Vom Repository-Root:

```sh
python3 -m tests.run --list
python3 -m tests.run --feature warp --mode headless --interface repl
python3 -m tests.run --feature inspect --interface repl
python3 -m tests.run --feature recording,replay --mode headless
python3 -m tests.run --scenario warp.exact_ticks_and_idle --mode headless --interface json
python3 -m tests.run --all --allow-rendered
```

Ohne Einschränkung werden alle Features und beide Modi/Zugänge ausgewählt.
`--all` macht diese Auswahl ausdrücklich. `--list` startet keine Prozesse.
Grafische Sessions benötigen `--allow-rendered` für diesen Lauf sowie einen
wachen Desktop. Ohne angebundene App bleibt die Gesamtmatrix unvollständig.

JSON verwendet den öffentlichen CLI-Eingang für Command-JSON; eigene Szenarien
prüfen zusätzlich Scripts. REPL verwendet einen persistenten echten REPL-Prozess:
Warp, Inspect, Recording und Replay über Kurzformen, weitere Commands über
`command <json>`. Die Ergebnisse stammen aus der Ausgabe dieses REPL.
Ein PTY-Vertrag prüft auch echte Terminaleingabe und Terminal-Restoration.

## Feature-Matrix

| Feature | Eigener Nachweis |
| --- | --- |
| `connection` | Session-Isolation, Request-Zuordnung, Arbeit nach Disconnect, Prozessende und echte Server-Signale |
| `warp` | Exakte Ticks, Stillstand, Ablehnung von null Ticks, Pace-Wechsel und Stop |
| `inspect` | Resource-Wert/Metadaten, unbekannte Type Paths, Entity-Projektionen |
| `keyboard` | Press/Hold/Release, ungültige Zustandswechsel, Verarbeitung erst beim Tick |
| `pointer` | Absolute/relative Bewegung, Buttons, Scroll und ungültige Positionen |
| `text` | Unicode, leere Eingabe, UTF-8-Bytegrenze, Verarbeitung erst beim Tick |
| `screenshot` | Bekannte Pixel, Overwrite, unveränderte Simulation; Headless-Ablehnung |
| `shutdown` | Terminale Completion, genau ein verwaltetes Ende, Prozessbereinigung |
| `recording` | Reihenfolge, aufgezeichnete Ablehnung, ausgeschlossene Controls, Footer beim Stop |
| `replay` | Frische Session, ungültiger Footer ohne Präfixausführung, Stop während Warp |
| `json` | Vorvalidierung, Command-Indizes und Script-Pipeline von Warp/Stop |
| `repl` | Parsefehler, Disconnect, echte Terminaleingabe während fremder Activity |
| `workflows` | Input und Spielabläufe aufnehmen und in frischem Prozess wiedergeben; Bewegung, Treffer, Fehlschüsse und Neustart |
| `reports` | Automatische Reports, Panic-/Tracing-Ursprünge, Filter, Launch-Diagnostik, echtes Panic-Abort und Persistenz nach Stop |
| `rust` | Alle modulnahen/öffentlichen Rust-Nachweise, Doctests, Stand ohne Default-Features |

Command- und Workflow-Verträge laufen über beide Modi/Zugänge. JSON-/REPL-
Syntaxprüfungen laufen nur über den jeweiligen Zugang. Bildverträge unterscheiden
Rendered und Headless. Fehlerprogramme laufen unabhängig vom Session-Modus
einmal je relevantem JSON-/REPL-Zugang. Launch-Fehler und OS-Signale werden über
die Verwaltung geprüft und benötigen keinen verbundenen REPL.

`catalog.py` ordnet Szenarien und Anforderungen zu; Fachassertions stehen in den
Verhaltensmodulen. Ein neuer öffentlicher Command ohne Vertrag lässt die Prüfung
in `runner/contracts.py` scheitern. Die Registrierung ersetzt keinen tatsächlichen Lauf.

## Aufbau

```text
tests/
  run.py                 Gemeinsamer Einstieg und Ergebnisbericht
  catalog.py             Auswahl der Verhaltensverträge
  application/           Konfiguration und Anforderungen der gemeinsamen Bevy-App
  session/               App-Anbindung, Server-/Session-Laufzeit und Aufräumen
  runner/                Verträge für Auswahl und Ergebnisbewertung des Runners
  connection/            Server, Session und Verbindungen
  interfaces/            JSON/REPL und zugangsspezifische Verträge
  commands/              Warp, Inspect, Keyboard, Pointer, Text, Screenshot, Shutdown
  recording/             Aufnahmeverträge
  replay/                Wiedergabeverträge
  reports/               Failure- und Report-Verträge
  workflows/             Zusammenspiel mehrerer Features
  diagnostics/           Unveränderte historische Untersuchungen
```

Die Kategorien `acceptance`, `integration`, `unit` und `support` entfallen.
Besondere Testprogramme liegen bei ihren Verhaltensverträgen:
`connection/protocol_faults/peer` erzeugt EOF, defekte Protokolle und hängende
Prozesse; `reports/panic_abort/program` verwendet einen echten Abort-Build.
`connection/activity_retention/producer.rs` erzeugt große Activity-Daten und
`reports/failure_source.rs` gezielte Fehler.
Ihre benötigten Bevy-Fähigkeiten sollen in die gemeinsame App übernommen werden.
Die bisherigen Bevy-Programme entfallen, sobald dieselben Verträge mit der
gemeinsamen App nachgewiesen sind. Der fehlerhafte Protokollpartner bleibt für
Fälle erforderlich, die eine reguläre Bevy-App nicht erzeugt.
Die bisherigen Spiel-/UI-Szenen und ihre spezifischen Tests sind entfernt.
Die Diagnostik bleibt unverändert und wird weder entdeckt noch gestartet.
Ihre alten App-/Importpfade sind historisch; ihre Anpassung gehört zur späteren
Untersuchung von Rendered/Headless.

## Rust-Module und Fehlerfälle

Unter `tests/` gibt es keine eigenständigen Rust-Testprogramme mit `#[test]`
mehr. Die verbleibenden Rust-Dateien sind ausführbare Fehlerquellen für die
Session-Szenarien. Direkte Rust-Verträge stehen unter `src/`, insbesondere
`session/api_tests.rs` für Pending-Ownership, Drop und Event-Queues und
`report/api_tests.rs` für unveränderliche Report-Objekte und explizite
Provider-Aufrufe. `cargo test` prüft diese zusammen mit den übrigen Modultests.

`--layer system` wählt Session-Szenarien, `--layer rust` nur Modulprüfungen.
`--faults-only` führt die modusunabhängigen Fehlerfälle ohne gemeinsame Bevy-App
aus. Ein erfolgreicher Fehlerlauf ersetzt nicht die App-Matrix:

```sh
python3 -m tests.run --layer rust
python3 -m tests.run --feature recording,replay --layer rust
python3 -m tests.run --feature warp --mode headless --interface repl --layer system
python3 -m tests.run --layer system --faults-only
python3 -m tests.run --feature reports --layer system --faults-only --interface repl
python3 -m unittest tests.runner.contracts -v
```

`tests.run` ist der Einstieg für die Verhaltenssuite. `runner/contracts.py`
prüft den Runner selbst, etwa ungültige Auswahl und fehlende Voraussetzungen.
Es ist kein zweiter Suite-Runner.

Der Runner baut technische Prozesse, Root-Examples, CLI und konfigurierte
App-Targets mit Lockfiles. `--no-build` verwendet vorhandene Artefakte.
`--cli` und `--app` erlauben andere Binary-/Konfigurationspfade.

## Ergebnisse und Feature-Übernahme

Jeder Lauf bewahrt `results.json`, Command-/REPL-Journale, Serverlogs und
Session-Artefakte unter `target/test-runs/`. `--output` bestimmt einen anderen
neuen Ergebnisordner. Unabhängige Szenarien laufen nach Fehlern weiter und
bekommen jeweils eigene Server und frische Sessions.

Exit `0` bedeutet eine bestandene vollständige Auswahl. Exit `1` bedeutet
Fehler, Exit `2` fehlende Voraussetzungen oder eine ungültige Auswahl.
Fehlende Fähigkeiten, fehlende Modi und nicht freigegebene Grafik gelten
niemals als bestanden. Keine Capture-Retries oder versteckten Ticks.

Neue Features erweitern den passenden Command-Vertrag oder Workflow und dessen
Registrierung. Die erforderlichen Kombinationen und der Gesamtlauf müssen am
finalen Stand bestehen. Eigenständige Negativnachweise bleiben gezielt testbar.

CI prüft Rust-Verträge, Suite-Auswahl und die Fehlerfälle über JSON/REPL. Ein
separat auslösbarer Headless-Matrix-Job verwendet das gemeinsame Sammelspiel.
Dieser Job ist noch kein automatischer Pflichtcheck. Technische CI allein ist
keine bestandene Gesamtabnahme; Grafik bleibt eine ausdrücklich bediente Prüfung.
