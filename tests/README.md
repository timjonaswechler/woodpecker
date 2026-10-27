# Woodpecker-Abnahmesuite

Diese Suite prüft Woodpeckers öffentliche Verträge, nicht die Missionen der
Signalstation. [Anwender-Beispiele](../examples/README.md) liegen getrennt unter
`examples/` und importieren keine Hilfen dieser Suite.
[Arbeitsstand, Belege und offene Aufgaben](../docs/testing-plan.md) stehen nur im
Testplan; diese Datei beschreibt Auswahl, Ausführung und Zuständigkeiten.
Die [begrenzte Mutationsprüfung](mutations.md) nutzt vier bestehende fensterlose
Owner in isolierten Quellkopien; sie ist ein separates Werkzeug, keine weitere
Produktsuite oder Gesamtabnahme.

## Voraussetzungen und Sicherheitsgrenzen

- Vom Repository-Root auf Unix ausführen; Rust gemäß `Cargo.toml`, Cargo, Git
  und Python **3.11+** müssen verfügbar sein.
- Nur die PNG-Tests `screenshot/capture.py` und `screenshot/overwrite.py` benötigen
  Pillow: bei Bedarf `python3 -m pip install Pillow`. Andere Tests und Discovery
  benötigen es nicht.
- Grafische Läufe einschließlich Wiederholungen zuerst mit Tests, Appstarts,
  Fenstern, Eingriffen und Tickbudgets freigeben. Die konkreten Varianten und
  Aktionen stehen im jeweiligen Test. `WOODPECKER_TEST_RENDERED=1` und
  `--rendered` bestätigen nur technisch eine bereits erteilte Freigabe.
- Desktop wach halten, Spielfenster zumindest teilweise sichtbar lassen.
  Größe, Skalierung und Bildschirmzuordnung während einer Session nicht ändern.
  Kein Headless-/Offscreen-Ersatz, keine OS-Eingaben oder persönlichen Settings.
- Einige Fälle erzeugen absichtlich Panics, Abort, Exit, Prozesssignale,
  Dateigrößenfehler und Symlinkwechsel. Nur testeigene Prozesse und Dateien
  betroffen; normale Läufe veröffentlichen keine echten GitHub-Issues.
- Keine versteckten Starts, Inputs, Ticks, Resets oder Aktionswiederholungen.
  Tests laufen seriell; eine fehlende Voraussetzung oder ein Skip ist kein Grün.

## Einstieg

```sh
# Nur Auswahl und tatsächliche Cargo-Befehle anzeigen; keine Builds oder Prozesse.
python3 -m tests.acceptance --list

# Vollständigen lokalen Umfang nur bauen; keine Spielfenster. Erwarteter Exit: 2.
python3 -m tests.acceptance --build-only

# Fensterlos: 15 Python-, neun interne Rust- und ein öffentlicher Rust-Fall.
# Erwarteter Exit auch bei bestandener Auswahl: 2, da lokale Grafiknachweise fehlen.
python3 -m tests.acceptance --non-graphical

# Derselbe Teilumfang mit zusätzlicher Feature-Matrix, wie in CI.
python3 -m tests.acceptance --non-graphical --feature-matrix

# Offline-Harness-Prüfungen: keine Cargo-, Spiel- oder GitHub-Starts.
python3 -m unittest discover -s tests/runner_checks -t . -p '*.py'
```

**Erst nach passender Grafikfreigabe:**

```sh
# Gesamter lokaler Sollumfang: 93 Produkttests plus Kontrolle 3.18, kein echtes gh.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --rendered

# Pilot: 16 Produkttests, keine Gesamtabnahme.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --pilot --rendered

# Nur die absichtlich falsche Erwartung aus 3.18; kein zusätzlicher Produkttest.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --negative-control --rendered
```

Der vollständige lokale Lauf baut elf Schritte einschließlich Feature-Matrix.
Teilumfänge bauen nur ihre benötigten Abhängigkeiten; `--feature-matrix` ergänzt
explizit die Library-Kombinationen und die normale App. `--list` und `--build-only`
lassen sich mit den Auswahlflags kombinieren. Der externe Driver wird nur für
`--external-github` gebaut, nie im lokalen Standard.

### Ergebnisse und Exitcodes

| Exit | Bedeutung |
| --- | --- |
| **0** | Vollständiger lokaler Erfolg einschließlich 3.18; bei `--list` nur gültige Planung. |
| **1** | Unerwarteter Build-, Test- oder Cleanup-Fehler. |
| **2** | Fehlende Voraussetzung oder unvollständiger Gesamtnachweis, auch bei bestandener Teilauswahl oder Build-only. |

Auswahl, lokale Gesamtabnahme und GitHub-Ergebnis werden getrennt gemeldet.
Ein bestandener externer Einzeltest ersetzt keine lokalen Fälle im selben
Runner-Aufruf. CI akzeptiert Exit 2 nur mit vollständig bestandener ausgewählter
fensterloser Auswahl, nicht bei Fehlern oder Skips.

Nach Fehler/Skip stoppt der gemeinsame Einstieg nach dem Cleanup; keine
automatischen Wiederholungen. Erzwungenes Harness-Cleanup ist immer Fehler,
auch wenn danach alle Prozesse verschwunden sind. Absichtlich erzwungenes
**Produktende** in einem Shutdown-/Drop-Test ist davon zu unterscheiden.

Kontrolle 3.18 liest zuerst den vollständigen richtigen Settings-Wert und prüft
danach einen absichtlich falschen Spielernamen. Nur genau diese typisierte
`ExpectationControlFailure` ohne weiteren Fehler bestätigt die Kontrolle.
Seed und normale Erwartung bleiben unverändert. Der eigenständige Kontrolllauf
ist weiterhin keine Gesamtabnahme und endet bei Bestätigung mit Exit 2.

### Artefakte und Buildcache

Jeder Lauf meldet seinen Ordner unter `target/test-runs/`:

- `summary.json`: Commit/Dirty-Zustand, Versionen, Konfiguration, Sollumfang,
  Auswahl, Ergebnisse, Dauer und Artefakte je Fall/Feature.
- `build-*.log`, `rust-*.log`, `unittest.log`: vollständige Standardausgaben.
- Fallordner: Commands/Activity, erzeugte Konfiguration, Reports/Bilder/Recordings,
  Prozess- und Cleanup-Nachweise. Fehlbelege werden nicht überschrieben.

Normale Builds verwenden `target/`; der echte `panic=abort`-Fall zusätzlich
`target/panic-abort` mit `CARGO_PROFILE_DEV_PANIC=abort`. Das ist ein eigener
wiederverwendbarer Cache, kein anderes normales Buildprofil. Der erste Build
braucht entsprechend Platz. Der Runner führt kein `cargo clean` aus und löscht
weder Caches noch frühere Belege. Laufartefakte sind lokal, nicht versioniert.

## Einzeltests

Python-Dateien haben **kein `test_`-Präfix**. Bei Discovery deshalb `-p '*.py'`
oder einen konkreten Dateinamen verwenden; `-t .` hält den Repository-Root als
Importwurzel. Tests importieren gemeinsame Hilfen, keine anderen ausführbaren Fälle.

```sh
cargo build --locked --features cli --bin woodpecker

# Eine Datei ohne Spielfenster.
python3 -m unittest -v tests.cli.command_errors
python3 -m unittest discover -s tests/session -t . -p 'invalid_launch_config.py' -v

# Öffentlicher Rust-Vertrag ohne Session oder Fenster.
cargo test --locked --test handle_encoding -- --nocapture

# Alle neun internen Owner-Tests ohne Spielfenster.
cargo test --locked --features cli,ui,screenshot --lib -- --nocapture --test-threads=1
```

**Grafische Einzelfälle erst freigeben und mit denselben App-Features bauen:**

```sh
cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app

# Ein Fenster; begrenzte Engine-/Spielvorbereitung, danach acht Aktionsticks.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.keyboard.movement

# Ein Manual-Fenster, keine Ticks/Inputs, nur lokaler Provider.
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli,ui,screenshot \
  --test local_provider -- --nocapture --test-threads=1
```

`SessionReady` bedeutet Steuerungsbereitschaft, nicht fertig geladenes Spiel.
Engine-Fälle erlauben bis zu 3600 Startup-Ticks/60 Sekunden und gegebenenfalls
120 ausdrücklich angeforderte Spielbereitschaftsticks/zehn Sekunden;
Abweichungen und anschließende Aktionsticks stehen im jeweiligen Test.
Zwei-Session-Fälle verwenden `WINDOW_POSITIONS` aus `_support.py`. Die anfängliche
Versetzung ändert keine Fokus-, Größen- oder Skalierungsoptionen und garantiert
keine Sichtbarkeit gegenüber anderen Desktopfenstern.

## Echter GitHub-Nachweis 4.81 — separat freigeben

[`providers/github_live.py`](providers/github_live.py) ist standardmäßig ausgeschlossen.
**Vor dem Lauf** Repository, einen Manual-Appstart ohne Ticks/Inputs, zwei Submits
und das Schließen eigener Issues freigeben. Erwartet wird ein Issue; bei defekter
Deduplizierung können höchstens zwei run-eigene Issues entstehen. Es wird nichts gelöscht.

1. Genehmigtes `github.com`-Testrepository mit aktivierten Issues, Schreibrecht und
   Description-Marker `woodpecker-acceptance-test-repository` verwenden.
   Der Test selbst verändert keine Repository-Einstellungen.
2. `gh` vorher außerhalb des Tests privat authentifizieren. Keine Zugangsdaten
   in Chat, Argumente oder Artefakte schreiben. Fake-gh-Umgebung ist unzulässig.
3. Nach Freigabe den Einzelumfang ausführen:

```sh
WOODPECKER_TEST_RENDERED=1 \
WOODPECKER_TEST_GITHUB_REPO=OWNER/TEST_REPOSITORY \
WOODPECKER_TEST_GITHUB_CLEANUP=close \
python3 -m tests.acceptance --external-github --rendered
```

`GH_REPO`/`GH_HOST` gelten nur für eigene Kindprozesse. Der öffentliche SDK-
Transport startet oder veröffentlicht nichts implizit. Vor dem zweiten Submit
muss dieselbe paginierte Issue-Liste den vollständigen ersten Run-Body zeigen:
maximal 90 Sekunden erfolgreiche Leseobservationen, keine Wiederholung
fehlgeschlagener Requests oder Publikationen. Erwartet sind `Created/Existing`
mit identischer Referenz und exaktem Markdown, niemals lokaler Fallback.

Cleanup liest bestätigte Referenzen auch bei verzögerter Liste direkt und prüft
Body, Run-UUID, Signatur und URL vor dem Schließen. Ein ungewisser Ausgang wird
nicht als erfolgreich bereinigt verbucht. Vier Offline-Harness-Regressionen
prüfen diese Grenzen ohne echte App oder Netzwerk. `--external-github --build-only`
baut nur Driver und Automation-App ohne GitHub-Anfrage oder Spielstart.

## Inventar und Zuständigkeiten

94 Produkttests: **75 Python + zehn öffentliche Rust + neun interne Rust**.
93 sind lokal, einer extern. Mit Kontrolle 3.18 enthält das Inventar 95 Einträge;
`runner_checks/` gehört nicht zu den Produkttests. Das tatsächliche Inventar
und die Buildauswahl kommen aus `acceptance.py --list`, nicht aus Laufchroniken.

| Python-Ordner | Fälle | Verantwortete Verträge / Plan-IDs |
| --- | ---: | --- |
| [session](session/) | 15 | Lebenszyklus, Isolation, Korrelation, Disconnect, Zugriff und Protokoll; 3.1–3.7, 4.8–4.15 |
| [inspect](inspect/) | 5 | Resource-Werte/Metadaten, Entity-Queries, Hierarchie und Reflection; 3.8, 4.16–4.19 |
| [startup](startup/) | 4 | Manual-/Engine-Startup, Grenzen und Konfiguration; 3.9, 3.11, 4.2–4.3 |
| [tick](tick/) | 5 | Exakte Ticks, App-Clock, Pace, Stop und Fehler; 3.10, 4.4–4.7 |
| [keyboard](keyboard/) | 3 | Bewegung, einmalige Aktion und Übergänge; 3.12, 4.20–4.21 |
| [pointer](pointer/) | 5 | UI-Picking, relative Bewegung, Buttons, Scroll und Ablehnungen; 4.22–4.26 |
| [text](text/) | 2 | Unicode, Fokus und UTF-8-/Widget-Grenzen; 4.27–4.28 |
| [screenshot](screenshot/) | 4 | Bild-Fixture, erster Tick, Overwrite und sichere Pfade; 3.13, 4.29–4.31 |
| [recording](recording/) | 3 | Dateivertrag, Barrieren und I/O-Fehler; 4.32–4.34 |
| [replay](replay/) | 5 | Frische Session, Vorvalidierung, Stop, Exklusivität und Blockierung; 3.14, 4.35–4.38 |
| [cli](cli/) | 5 | Config/JSON, Scripts, Pipeline und ungewisse Ergebnisse; 4.39–4.43 |
| [repl](repl/) | 3 | Eingabeformen, PTY/Terminal und Clientende; 4.44–4.46 |
| [reports](reports/) | 9 | Tracing, Panics/Abort, Exit, Marker und Provider-Fortschritt; 3.16–3.17, 4.47–4.53 |
| [shutdown](shutdown/) | 6 | Vorbedingungen, aktive Arbeit, Starting, Mehrsession, Frist und Signale; 4.58–4.63 |
| [providers](providers/) | 1 | Echter GitHub-Nachweis 4.81, ausschließlich separat |

[`rust/`](rust/) enthält zehn explizite Cargo-Targets mit je einem öffentlichen
Test: `local_provider`, `github_provider`, `report_snapshot`, `report_signature`,
`pending_ownership`, `pending_drop`, `session_shutdown`, `history`, `event_overflow`,
`handle_encoding`. Nur der letzte läuft ohne Spielfenster; Rust-Ownership,
Pending-Drop und direkte Session-Lebensdauer sind keine doppelten CLI-Tests.

Die neun internen Tests liegen bei ihren tatsächlichen Ownern unter `src/`
(`cfg(test)`): native Inputs, Textzielbindung, Readback-Lebensdauer, Screenshot-
und Recording-/Replay-Pfadaustausch, Session-Identität, Request-ID-Erschöpfung,
Übergangsrennen und Reportabschluss (4.71–4.78, 4.82). Sie steuern reale
World-/Datei-/Koordinatorgrenzen instanzlokal, keine nachgebaute Produktlogik,
keine Winit-/GPU-Abnahme oder neue öffentliche API.

## Hilfen und Konfiguration

- [`_support.py`](_support.py): frische TOML-Daten und Roundtrip-geprüftes Schreiben,
  Serverprozess, CLI-Transport, Logs, Fristen und verifiziertes Prozess-Cleanup.
  `result` beobachtet genau einen angenommenen Request, ohne Resubmission oder
  Gap-Recovery. `call` sendet genau den angegebenen Command, `read_resource`
  genau eine lesende Query. Starts und Spielaktionen bleiben im Test sichtbar.
- [`_builds.py`](_builds.py): Standard-Cargo-Aufrufe, Compilerartefakte und Testlisten;
  [`acceptance.py`](acceptance.py): Discovery, sichere Auswahl und ehrliche Ergebnisse.
- Feature-`__init__.py`: nur gemeinsame Abfragen, Artefaktkodierung und explizites
  Prozess-/Transportmanagement. Keine versteckten Fokus-, Input- oder Tick-Aktionen.
  Der Textordner re-exportiert die gemeinsame reine Pointer-Layoutabfrage.
- [`faults/`](faults/): sieben verwendete externe Werkzeuge für JSONL-/WebSocket-
  Protokoll, Reports, Dateilimit, Fake-gh, interne Wire-Partner und Cleanup.
  Keine Bevy-Welt oder Rendering-Simulation; eigenes Fake-gh ruft nie echtes gh auf.
- [`repl/pty_client.py`](repl/pty_client.py): eigene controlling PTY, Termios vor/nach
  dem Client beobachten, keine Reparatur von Terminalattributen. Nur Kernel-
  Status `PENDIN` ist ausgenommen; das Nutzerterminal bleibt unberührt.
- [`rust/support/mod.rs`](rust/support/mod.rs): Konfiguration, eigene Artefakte/Git-
  Fixtures und reine Beobachtungen; keine versteckten Sessionstarts oder Ticks.
  [`providers/live_driver`](providers/live_driver/Cargo.toml) ist ein separates
  Testwerkzeug, kein öffentliches Woodpecker-Binary.
- [`app.toml`](app.toml)/[`settings.json`](settings.json): Manual-Startup und fester
  Settings-Seed statt persönlicher Werte. Varianten nur in Laufartefakten über
  `load_config`/`write_config`; Pfade gelten vom Repository-Root bzw. beim Seed
  vom App-Projektverzeichnis. Gleiche Features vor direkten Appstarts bauen.

`owned_process` prüft lebende Abstammung und separate Gruppe; `wait_process_gone`
verlangt danach unabhängig PID-/Gruppenabwesenheit. Timeout-Cleanup erfasst vor
jedem Kill die verifizierte eigene Hierarchie neu, Blattgruppen vor Eltern.
Fremde, wiederverwendete oder umgehängte Identitäten werden nicht signalisiert;
erzwungenes Cleanup bleibt ein Fehler.

Öffentliche Reporttests prüfen die dauerhafte Dateimenge bzw. gh-Aufrufzahl nach
Serverexit. Der [4.82-Owner-Test](../src/server/tests/test_report_completion.rs)
hält den echten Worker und prüft denselben Completion-/Join-Pfad: erst alle drei
korrelierten Outcomes, dann Serverabschluss. Weder `Ended` noch eine ruhige
Poll-Phase ist eine öffentliche Providerabschlussbarriere.
