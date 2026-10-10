# Verhaltensbasierte Testsuite

Die Suite ist nach Server/Session-Verbindungen, Command-Zugängen, Commands,
Recording, Replay, Reports und gemeinsamen Abläufen aufgebaut. Verträge werden
über Session-Modus und Zugang ausgewählt. Die bisherigen Testart-Ordner und
szenenspezifischen Bevy-Apps sind entfernt.

[tests/README.md](../tests/README.md) beschreibt Einstieg und Feature-Matrix.
Der [App-Vertrag](../tests/application/README.md) beschreibt die gemeinsame
Bevy-App, die separat entwickelt wird.

Die bisherigen eigenständigen Rust-Verhaltensprüfungen sind übertragen:

| Bisheriger Nachweis | Aktueller Vertrag |
| --- | --- |
| Session: Protokollfehler, EOF, Exit, Pipe-Drain, Startfehler | `tests/connection/protocol_faults/contracts.py`, JSON/REPL und Launch-Verwaltung |
| Session: Recording-Barrieren, Reihenfolge, Protokollfehler, Prozessabbruch | `tests/recording/contracts.py` und `failures.py` |
| Session: Replay-Vorvalidierung, effektive Ticks, Exklusivität, Abbruch, fehlgeschlagene Commands | `tests/replay/contracts.py` und `failures.py` |
| Activity-Limits, langsame Clients, nicht lesender Socket und große Reports | `tests/connection/activity_retention/contracts.py` |
| Panic-/Tracing-Ursprünge, Filter und Fehler vor Ready | `tests/reports/observation.py` |
| Echter Panic-Abort, Failure vor Ended und genau ein Report | `tests/reports/panic_abort/contracts.py` |
| Rust-Pending-Ownership, abgegebene Handles, Fortschritt ohne Consumer und begrenzte Event-Queue | `src/session/api_tests.rs`, gezielte direkte Rust-Verträge |
| Unveränderliche Rust-Report-Objekte, Startkonfiguration und explizite Provider-Aufrufe einschließlich isoliertem GitHub-CLI | `src/report/api_tests.rs`, gezielte direkte Rust-Verträge |

`tests.run --layer system --faults-only` führt die Fehlerfälle ohne gemeinsame
App aus und ist ein CI-Pflichtcheck. Die vier alten Cargo-Test-Targets sind
entfernt. Cargo prüft die Rust-Module unter `src/`; `tests/` enthält die
gemeinsamen Verhaltensverträge und deren Fehlerprogramme.

Die verbleibenden Schritte sind:

1. Die gemeinsame App mit beobachtbarem Tick-Zustand anbinden.
2. Input-/Entity-/UI-Beobachtungen und erforderliche Modi ergänzen. Die benötigten Activity- und Fehlerfähigkeiten der bisherigen technischen Bevy-Programme übernehmen und diese nach gleichwertigem Nachweis entfernen. Für echtes `panic=abort` einen separaten Build derselben App prüfen.
3. Die Matrix am finalen App-/Produktstand vollständig ausführen und Headless als CI-Pflichtcheck aktivieren.
4. Render-/Headless-Diagnostik erneut untersuchen, besonders die erste Bildverfügbarkeit. Die bisherigen Diagnosequellen bleiben erhalten.
5. Coverage einschließlich Kindprozessen erfassen und kritische Verträge mit gezielten Mutationen prüfen.

Neue Features erweitern vorhandene Verträge oder gemeinsame Abläufe im selben
Änderungspaket. Ein erfolgreiches Feature-Szenario ersetzt keine erforderliche
Gesamtprüfung. Fehlende App-Fähigkeiten sind unvollständige Nachweise.
