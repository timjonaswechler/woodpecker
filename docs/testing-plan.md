# Plan der Woodpecker-Abnahmesuite

Arbeitsstand und verbleibende Aufgaben des Neuaufbaus ab `f9a39ed`.
Ausführbare Aufrufe, Einzeltests und Abhängigkeiten stehen in der
[Testanleitung](../tests/README.md); diese Datei bleibt die zentrale Arbeitsliste.

## Full-stop-Abnahme und offene Checkpoints

- [x] **1. Bestand prüfen:** Altbestand analysiert und auf Nutzerauftrag entfernt.
- [x] **2. Zielstruktur festlegen:** öffentliche Zugänge, Testform und Hilfen abgestimmt.
- [x] **3. Pilot abschließen:** Basisfälle, vier Spielabläufe und Negativkontrolle belegt.
- [x] **4. Abnahme vervollständigen:** alle Produkttests einschließlich 4.82 umgesetzt
  und ausgeführt; lokaler Gesamtlauf und echter GitHub-Nachweis bestanden.
- [x] **5. Bereinigung abschließen:** Struktur, Einstiegsdokumentation und Inventar abgeglichen.
- [x] **6. Anwender-Beispiele:** Bewegung, Text und Replay über die öffentliche CLI belegt.
- [x] **7. Mutationsprüfungen:** vier Produktfehler isoliert fachlich erkannt, Äquivalenzkontrolle getrennt.

**Umfang:** 94 Produkttests: 75 Python, zehn öffentliche Rust-Integrationstests,
neun interne Rust-Tests. Davon 93 lokal und ein externer GitHub-Fall.
Kontrolle 3.18 ist kein weiterer Produkttest, aber ein zusätzlicher Inventareintrag:
insgesamt 95. Die nun 49 Harness-Prüfungen zählen separat.

**Produktnachweise 4.80/4.81:** `de5d795` mit den dokumentierten
Live-Harness-Korrekturen im Arbeitsbaum. Lokaler und externer Nachweis stammen
bewusst aus getrennten Läufen:

| Checkpoint | Ergebnis | Lokaler Laufbeleg |
| --- | --- | --- |
| **4.80** | 93 Produkttests plus Kontrolle 3.18, elf Builds; keine Skips oder erzwungenes Harness-Cleanup. Auch 3.12 bestanden. | [Gesamtlauf](../target/test-runs/acceptance-wf1norm_/summary.json) |
| **4.81** | Echtes `Created → Existing` derselben Referenz #3, vollständiger Markdown geprüft, kein Fallback; Issue verifiziert geschlossen. | [GitHub-Lauf](../target/test-runs/acceptance-_29tva6l/summary.json) |

Der lokale Lauf dauerte 447,470 Sekunden; 158 Cleanup-Nachweise ohne Eskalation,
Quellzustand während des Laufs unverändert. Der GitHub-Lauf nutzte genau zwei
Submits mit unabhängiger Listen-Sichtbarkeit vor dem zweiten. Auch die beiden
Issues des früheren Fehlversuchs sind verifiziert geschlossen; Repository-
Description restauriert und keine Produkt-/Fixture-Prozesse übrig.

[Abschlussnachweis](../target/test-runs/complete-through-4-81-oXL4fw/completion.json)
mit Quellhashen, Cleanup-Audit und Reviewumfang. Die Laufartefakte unter `target/`
sind lokale, nicht versionierte Belege. Der separate GitHub-Aufruf endet trotz
bestandener Auswahl mit Exit 2, weil ihm allein die lokalen Nachweise fehlen.

**Abschnitt 5 geprüft:** 32 Harness-Prüfungen, der tatsächliche fensterlose
CI-Schritt mit 25 Produkttests, elf lokale Builds samt kompilierten Testlisten,
Workspace-Check, Clippy und Format bestanden. Der aktualisierte CLI-Walkthrough
lief mit einer echten App, 13 Ticks, ohne Inputs und ohne erzwungenes Cleanup.
Inventar (95 Einträge), Zuständigkeiten und lokale Dokumentationslinks abgeglichen.
Keine fachliche Test-/Produktänderung: nur ungenutzter Import, überholter Kommentar
und leerer Platzhalter entfernt; Quellhash-/AST-Abgleich hält den Bezug zu den
obigen vollständigen Laufbelegen fest. Kein neuer vollständiger Grafik- oder
GitHub-Lauf für diese Bereinigung.
Belege: `target/test-runs/step-5-tamjub3t/`.

**Abschnitt 6 geprüft:** Drei eigenständige [CLI-Beispiele](../examples/README.md)
mit eigener Konfiguration und Seed, ohne Imports aus der Abnahmesuite. Bewegung
und Text mit je acht Aktionsticks; Replay mit fünf Ticks in A und fünf in einer
frischen B-Session. Vier serielle Appstarts, maximal ein Spiel gleichzeitig,
je drei Startup-Ticks und keine zusätzlichen Spielbereitschaftsticks; alle
PIDs/Gruppen verschwunden, keine Harness-Eskalation oder persönlichen Settings-
Änderungen. Ein erster Versuch scheiterte vor Aktionen mit `window surface=false`;
die Desktop-Abfrage bestätigte danach einen schlafenden Bildschirm. Nach
verifiziertem Aufwecken bestanden dieselben Beispiele mit unveränderten Fristen
und ohne Codeänderung. Beide Versuche bleiben erhalten:
`target/test-runs/steps-6-7-ab8235by/`, Erfolgsbeleg unter
`example-proof-attempt-2/examples-proof.json`. Das sind keine zusätzlichen Produkttests.

**Abschnitt 7 geprüft:** Quellpin `121a536`, vier echte grüne Baselines und
sechs erfolgreiche Cargo-Builds in einer Nicht-Git-Quellkopie mit privatem
COW-Cache. Jeder Produktfehler scheiterte mit Exit 101 an seiner vorgesehenen
fachlichen Assertion, nicht am Build oder Cleanup:

| Mutation | Bestehender Owner | Tatsächlich erkannter Fehler |
| --- | --- | --- |
| Generation-Bit verloren | 4.69 | Generation `0` statt `2147483648`, `test_handle_encoding.rs:47` |
| Native Tastaturmessages behalten | 4.71 | `KeyboardInput` nach Capture nicht leer, `test_native_input.rs:159` |
| Text zum aktuellen Fokus umgeleitet | 4.72 | B enthält `beta:Grüße 🦉` statt `beta:`, `test_target_binding.rs:58` |
| Request-Zähler nicht erhöht | 4.77 | zweiter Send liefert nicht `RequestIdExhausted`, `test_request_id_exhaustion.rs:83` |

Vier erkannte Fehler, null Überlebende; `u32 | 0` blieb als vorab typbegründete
Äquivalenzkontrolle grün. Quellen/Oracles restauriert, 35.804 Cachedateien ohne
Parent-Hardlinks/Symlinks geprüft, ursprüngliche Quellen/Binaries unverändert,
keine Apps, Grafik, GitHub-Aufrufe oder erzwungenes Cleanup. Zwei diagnostizierte
Toolfehler bleiben separat erhalten und zählen nicht als Erkennung. Die
[Mutationsanleitung](../tests/mutations.md) erklärt Umfang und Reproduktion;
Rohbelege stehen unter `target/test-runs/steps-6-7-ab8235by/mutation-evidence/`,
Erfolg in `mutations-p6vb0sa4/summary.json`. Diese Stichprobe beweist keine
universelle Fehlersensitivität und ändert keine historischen Qualitätsnoten.
Am integrierten Stand bestanden außerdem alle 49 Offline-Harness-Prüfungen,
der echte fensterlose CI-Schritt mit 25 Produkttests, Workspace-Check, Clippy,
Format und Link-/Inventarabgleich. Keine dauerhafte Änderung an SDK oder Oracles.

**Keine Merge-Freigabe:** Vor einer PR bleiben die Basisbranch des leeren `origin`
und das Review des um 42 genehmigte Vorgängercommits erweiterten Umfangs zu klären.

## Verbindliche Testregeln

- Primär Python/`unittest` über echte CLI und Command-JSON. Sollwerte am
  öffentlichen Vertrag prüfen, nicht aus dem Istverhalten abschreiben.
- Ein verantwortlicher Test je Verhalten. REPL/Scripts prüfen ihre Besonderheiten;
  Rust-/interne Tests nur für zusätzliche Verträge oder öffentlich nicht
  zuverlässig erzeugbare Risiken. Keine Dummytests oder leeren Testtargets.
- Vorbereitung, Aktionen, Erwartungen, Appstarts, Tickbudgets und Cleanup stehen
  sichtbar im Test. Keine versteckten Inputs, Ticks, Resets oder automatischen
  Wiederholungen, auch nicht nach Activity-Verlust. `EngineReady` ersetzt keine Spielbereitschaft.
- Tests laufen seriell und unabhängig; mehrere isolierte Sessions pro Fall sind
  erlaubt. Fehlerwerkzeuge dürfen nur eigene Prozesse, Dateien und Testrepositories
  betreffen. Keine OS-Eingaben, persönlichen Settings oder Fehler im Projekt-Checkout.
- Vor grafischen Läufen inklusive Wiederholungen Tests, Starts/Fenster, Varianten,
  Eingriffe und Tickbudgets nennen und frische Freigabe einholen.
  `WOODPECKER_TEST_RENDERED=1` ist nur eine technische Sperre.
- Grafik braucht einen wachen Desktop, zumindest teilweise sichtbare Fenster und
  unveränderte Größe/Skalierung/Bildschirmzuordnung. Kein Headless-/Offscreen-Ersatz.
- Pro Lauf Versionen, Commit/Dirty-Zustand, Konfiguration sowie Build-, Test- und
  Cleanup-Belege festhalten. Derselbe gemeinsame Einstieg führt die Einzeltests
  aus, keinen zweiten monolithischen Spielablauf.
- **Bestanden:** alle verpflichtenden Nachweise erfolgreich, keine ungeklärten
  Outcomes oder Prozessreste. **Fehlgeschlagen:** unerwarteter Verhaltens-, Build-
  oder Cleanup-Fehler. **Unvollständig:** fehlende Voraussetzung, Skip oder Abbruch.
  Erwartete Produktfehler zählen nur als ausdrücklich geplanter Fehlerfall;
  erzwungenes Harness-Cleanup bleibt Fehler und verdeckt nie den Erstfehler.
- Echter GitHub-Nachweis nur separat freigegeben: benanntes Testrepository mit
  Description-Marker `woodpecker-acceptance-test-repository`, erlaubte Testdaten
  und Cleanup. Nur eigene Issues nach Prüfung von Body, Run-UUID, Signatur und
  URL schließen, nichts löschen. Keine Publikationswiederholung bei ungewissem Ausgang.

## Zielstruktur und erlaubte Hilfen

Feature-Tests liegen unter `tests/`, öffentliche Rust-Targets unter `tests/rust/`,
begründete interne Owner-Tests unter `src/`. Eine Datei je Test; Python ohne
`test_`-Präfix. `tests/app.toml` und `tests/settings.json` sind die gemeinsame Basis.
Hilfen verwalten Konfiguration, Prozesse, Artefakte, Transport, Fristen und reine
Abfragen, keine Spielaktionen. Referenz-App-Missionen sind keine SDK-Features.
Anwender-Beispiele liegen getrennt unter `examples/`, ohne `tests/_support.py`.

## Qualitätsnachschärfung nach dem Drei-Reviewer-Stand

Umgesetzt sind unabhängige Sollwerte für Handles, Metadaten und Prozessbesitz,
Prüfungen jedes Hold-Ticks, vollständige Markdown-Payloads und ehrliche
Messvoraussetzungen. Dauerhafte Report-Effekte werden nach Serverende geprüft;
der reale Worker-/Join-Nachweis 4.82 ergänzt die öffentliche Beobachtung.
`Ended` oder eine ruhige Poll-Phase beweist keinen Providerabschluss.
Historische Bewertungen werden dadurch nicht automatisch aufgewertet.

## Abgeschlossene Checkpoints

Die Gruppen bewahren die bisherigen IDs; Code und verpflichtende Ausführung sind
erledigt. Dateizuordnung und Abläufe stehen in der [Testanleitung](../tests/README.md),
das ausführbare Inventar liefert `python3 -m tests.acceptance --list`.
Vorbereitungs-/Arbeitspunkte und Kontrolle sind keine zusätzlichen Produkttests.

- [x] **3.1–3.7** Server-/Session-Lebenszyklus, Isolation und Start-/ID-Fehler.
- [x] **3.8–3.14** Resource-Wert, Manual-/Engine-Startup, exakte Ticks, Bewegung,
  Screenshot und Replay in frischer Session.
- [x] **3.15** Opt-in-Diagnoseaktionen der Referenz-App vorbereiten.
- [x] **3.16–3.17** Tracing-/Panic-Report, Spielwirkung und Prozessende.
- [x] **3.18** Typisierte Negativkontrolle mit absichtlich falscher Erwartung.
- [x] **3.19–3.20** Gemeinsamer Einstieg, ehrliche Zusammenfassung und Pilotabschluss.
- [x] **4.1–4.7** Diagnosevorbereitung, Startup-Grenzen, Konfiguration, Clock,
  Warp-Stop, Pace-Wechsel und Zustandsfehler.
- [x] **4.8–4.15** Sessionbindung, Request-Korrelation, Disconnect, Input-/Datei-
  Isolation, Activity-Lücke, lokaler Zugriff und Protokollfehler.
- [x] **4.16–4.19** Resource-/Entity-Queries, Hierarchie, Handles und Reflection.
- [x] **4.20–4.21** Einmalige Tastaturaktion bei Hold und ungültige Übergänge.
- [x] **4.22–4.26** UI-Picking, relative Bewegung, Pointer-Buttons, Scroll und Fehler.
- [x] **4.27–4.28** Unicode, Fokus und UTF-8-/Widget-Grenzen.
- [x] **4.29–4.31** Screenshot vor erstem Tick, Overwrite und Pfadsicherheit.
- [x] **4.32–4.34** Recording-Dateivertrag, Barrieren und I/O-Fehler.
- [x] **4.35–4.38** Replay-Vorvalidierung, Stop, Exklusivität und technische Blockierung.
- [x] **4.39–4.43** CLI-/JSON-Fehler, Script-Vorvalidierung, Pipeline und unknown.
- [x] **4.44–4.46** REPL-Syntax, PTY/Terminal und Clientende gegenüber Sessionende.
- [x] **4.47–4.57** Tracing/Panic/Exit, Marker, Provider-Fortschritt, lokale und
  simulierte GitHub-Verträge, unveränderlicher Snapshot, Markdown und Signatur.
- [x] **4.58–4.63** Shutdown-Vorbedingungen, aktive Arbeit, Starting, mehrere
  Sessions, gemeinsame Frist und Signale.
- [x] **4.64–4.69** Öffentliche Rust-Verträge: Pending-Ownership/Drop,
  Sessionabschluss, History, Event-Überlauf und Handle-Kodierung.
- [x] **4.70–4.78** Risikoabgleich und interne Nachweise: native Inputs, Textziel,
  Readback, Pfadaustausch, ID-Kollision/-Erschöpfung und Übergangsrennen.
- [x] **4.79–4.80** Feature-Builds und vollständiger lokaler Gesamtlauf.
- [x] **4.81** Echter GitHub-Provider gegen freigegebenes Testrepository.
- [x] **4.82** Tatsächlicher Report-Owner schließt alle Jobs vor Serverabschluss ab.

## Weitere Schritte — in dieser Reihenfolge

### 5. Struktur und Dokumentation abschließend bereinigen — abgeschlossen

- [x] **5.1** Struktur geprüft: alle zehn Cargo-Testtargets und sieben Fehlerwerkzeuge
  verwendet; keine zusätzliche Testarchitektur nötig. Ungenutzten Import,
  überholten Kommentar und `tests/rust/.gitkeep` entfernt; unabhängige Oracles erhalten.
  - [x] Konfigurations-Teilaufgabe: `load_config`/`write_config` zentralisiert;
    22 Tests ändern TOML-Felder statt Text. Varianten und Roundtrip geprüft.
- [x] **5.2** `tests/README.md`: kurzer ausführbarer Einstieg mit Auswahl,
  Abhängigkeiten, Grafikfreigabe, lokalem/externem Umfang und Gesamturteilen.
- [x] **5.3** Root-/App-/API-Dokumentation abgeglichen: entfernte Fixtures nicht mehr
  als ausführbare Beispiele, alte Entwicklungsberichte klar historisch;
  aktuelle Startup-/Textziel-Semantik und Laufstände berichtigt.
- [x] **5.4** Plan, tatsächliches Inventar und Zuständigkeiten abgeglichen;
  Auswahl-/Buildbefehle, CLI-Walkthrough, Links und frische Prüfungen belegt.

### 6. Anwender-Beispiele — abgeschlossen

Kein zweiter vollständiger Feature-Testbestand; verständliche öffentliche Abläufe
für Spielentwickler ohne Import aus `tests/_support.py`.

- [x] **6.1** `examples/game_tests/movement.py`: Eingabe, explizite Ticks und Bewegung.
- [x] **6.2** `examples/game_tests/text.py`: fokussieren, Text eingeben, ticken, Wirkung prüfen.
- [x] **6.3** `examples/game_tests/replay.py`: Ablauf aufnehmen und in frischer Session wiedergeben.
- [x] **6.4** Einstieg in `examples/` dokumentiert und alle drei Beispiele wirklich
  ausgeführt; gemeinsame Hilfen bleiben auf Transport, Beobachtung und Cleanup begrenzt.

### 7. Fehlersensitivität durch Mutationsprüfungen — abgeschlossen

Erst nach Abnahmesuite, Bereinigung und Beispielen. Keine Behauptung,
alle denkbaren Produktfehler erkennen zu können.

- [x] **7.1** Vier zentrale Risiken gewählt: Generationsbit, native Tastatureingabe,
  Textzielbindung und Request-ID-Wiederverwendung; vorhandene Owner-Tests genutzt.
- [x] **7.2** Nur isolierte Quellkopien und privaten Buildcache verändert;
  Projekt-Checkout, persönliche Settings und ursprüngliche Binaries geschützt.
- [x] **7.3** Owner, erwartete und tatsächliche Assertion sowie Cleanup belegt;
  Setup-/Buildfehler sind keine Erkennung, Überlebende und Äquivalenz bleiben getrennt.

### Bisherige Belege, keine neue Gesamtabnahme

Ältere Etappen und Fehlversuche (u. a. `window surface=false`) stehen im
[Checkpoint-Audit](../target/test-runs/full-stop/checkpoint-audit.json).
Manche frühe Artefakte sind nicht mehr verfügbar; historische Notizen ersetzen
keine aktuellen Läufe. Die ausführliche vorherige Planfassung ist als lokaler
[Snapshot](../target/test-runs/testing-plan-compact-uWkNj6/before.md) erhalten.
Historische Prüfberichte in `docs/api/` zählen nicht als Nachweise des Neuaufbaus.
