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
- Die PNG-Tests `screenshot/capture.py`, `screenshot/overwrite.py` und
  `headless/basis.py` benötigen Pillow: bei Bedarf `python3 -m pip install Pillow`.
  Andere Tests und Discovery benötigen es nicht.
- Grafische Läufe einschließlich Wiederholungen zuerst mit Tests, Appstarts,
  Fenstern, Eingriffen und Tickbudgets freigeben. Die konkreten Varianten und
  Aktionen stehen im jeweiligen Test. `WOODPECKER_TEST_RENDERED=1` und
  `--rendered` bestätigen nur technisch eine bereits erteilte Freigabe.
- Native bleibt Default: Desktop wach halten, Spielfenster zumindest teilweise
  sichtbar lassen. Größe, Skalierung und Bildschirmzuordnung während einer Session
  nicht ändern. Kein stiller Headless-/Offscreen-Ersatz; der explizite L1-Modus
  steht unten. Keine OS-Eingaben oder persönlichen Settings.
- Einige Fälle erzeugen absichtlich Panics, Abort, Exit, Prozesssignale,
  Dateigrößenfehler und Symlinkwechsel. Nur testeigene Prozesse und Dateien
  betroffen; normale Läufe veröffentlichen keine echten GitHub-Issues.
- Starts, Inputs, Ticks und Resets erfolgen nur als ausdrücklich dokumentierte
  Test- oder Fixture-Aktionen und werden protokolliert; keine Aktionswiederholungen.
  Tests laufen seriell; eine fehlende Voraussetzung oder ein Skip ist kein Grün.

## Einstieg

```sh
# Nur Auswahl und tatsächliche Cargo-Befehle anzeigen; keine Builds oder Prozesse.
python3 -m tests.acceptance --list

# Vollständigen lokalen Umfang nur bauen; keine Spielfenster. Erwarteter Exit: 2.
python3 -m tests.acceptance --build-only

# Fensterlos: 15 Python-, 14 interne Rust- und ein öffentlicher Rust-Fall.
# Erwarteter Exit auch bei bestandener Auswahl: 2, da lokale Grafiknachweise fehlen.
python3 -m tests.acceptance --non-graphical

# Derselbe Teilumfang mit zusätzlicher Feature-Matrix, wie in CI.
python3 -m tests.acceptance --non-graphical --feature-matrix

# Offline-Harness-Prüfungen: keine Cargo-, Spiel- oder GitHub-Starts.
python3 -m unittest discover -s tests/runner_checks -t . -p '*.py'
```

**Erst nach passender Grafikfreigabe:**

```sh
# Nativer lokaler Sollumfang: 98 Produkttests plus Kontrolle 3.18, kein echtes gh.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --rendered

# Pilot: 16 Produkttests, keine Gesamtabnahme.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --pilot --rendered

# Nur die absichtlich falsche Erwartung aus 3.18; kein zusätzlicher Produkttest.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --negative-control --rendered
```

Der vollständige native lokale Lauf bereitet 13 Schritte einschließlich Feature-Matrix
und separatem Abort-Build vor.
Teilumfänge bauen nur ihre benötigten Abhängigkeiten; `--feature-matrix` ergänzt
explizit die Library-Kombinationen und die normale App. `--list` und `--build-only`
lassen sich mit den Auswahlflags kombinieren. Der externe Driver wird nur für
`--external-github` gebaut, nie im lokalen Standard.

### Einzeltest, Kategorie und vorbereitete Programme

`--test` akzeptiert eine exakte Inventar-ID oder einen Python-Modulnamen.
`--category` akzeptiert einen Python-Ordnernamen sowie `rust` oder `internal`.
Beide Optionen sind jeweils wiederholbar; unbekannte oder leere Auswahlen sind
Fehler. Echte GitHub-Fälle bleiben ausschließlich unter `--external-github`.

```sh
python3 -m tests.acceptance --test tests.keyboard.movement --list
python3 -m tests.acceptance --category keyboard --category pointer --list

# Einmal vorbereiten, ohne Spielfenster; erwarteter Exit 2.
python3 -m tests.acceptance --category keyboard --category pointer --category text --build-only
```

Build-only meldet zusätzlich `Produkte: …/products.json`. Diesen konkreten Pfad
anstelle von `MANIFEST` verwenden; grafische Ausführung weiterhin zuerst freigeben:

```sh
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance \
  --category keyboard --category pointer --category text --rendered --artifacts MANIFEST

# Derselbe vorbereitete Stand, nur ein Test, ohne Buildphase.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance \
  --test tests.keyboard.movement --rendered --artifacts MANIFEST
```

Ohne Auswahlflag gilt die native lokale Suite. Ein dafür vorbereitetes Manifest
muss sämtliche benötigten Programme und die Feature-Matrix abdecken. `--artifacts`
prüft kompilierte Quellen und Assets einschließlich uncommitteter Dateien,
Manifeste/Lockfiles, Build-Rezepte, Toolchain, Plattform und relevante
Build-Einstellungen sowie Programm-Hashes.
Fehlende, veränderte oder veraltete Produkte ergeben einen Fehler, keinen Neubuild.
Die Programme liegen inhaltadressiert unter `target/test-products/`; gleiche
Programme werden nicht pro Test kopiert. Auf macOS werden die zugehörigen
Debug-Symbole beim Vorbereiten gesichert. Der Projektcheckout bleibt für Assets
und Cargo-Metadaten erforderlich; Metadaten-/Versionsabfragen sind keine Builds.

### L1: expliziter Headless-Teilumfang mit GPU

Headless ist **nicht** `--non-graphical`: Dieser Modus benötigt die echte GPU,
aktuell das vereinbarte Metalprofil, aber kein natives Fenster. Erst nach eigener
Freigabe dieses Umfangs ausführen; Flags erteilen keine Freigabe.

```sh
python3 -m tests.acceptance --headless --list
# Nur vorbereiten, keine App-/GPUausführung; erwarteter Exit 2.
python3 -m tests.acceptance --headless --build-only
# MANIFEST durch den ausgegebenen products.json-Pfad ersetzen.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance \
  --headless --rendered --artifacts MANIFEST
```

`--headless` wählt 13 Fälle: die zehn ursprünglichen Keyboard-/Pointer-/Textowner
mit unveränderten Assertions und drei eigene Headlessowner. [basis.py](headless/basis.py)
prüft echten Appstart, Image-Ready, RGB-/Pause-/UI-/Unicode-/Overlaybilder und einen
PID-gefilterten OS-Audit ohne native Fenster. Für `Time<Real>` liest der Owner
einen ausdrücklich aktivierten Snapshot tatsächlicher Skalare/Updatezeitpunkte:
bei jeder Extraction, auch während Pause, und mit Fortschrittskontrolle nach
expliziten Ticks; kein Eingriff in den Clock oder zusätzliche Anwendungsschedules. [startup.py](headless/startup.py)
führt die ursprünglichen Manual-/Engine-/vier Fehler-Startupvarianten mit lediglich
geänderter Konfiguration aus. [terminal.py](headless/terminal.py) zerstört nach
healthy Capture nur das eigene logische Gerät und verlangt typisiertes Ende und
vollständiges Reaping. Die zehn Inputowner teilen eine
zurücksetzbare Signalstation: Manual-Handshake, maximal 120 Spielbereitschaftsticks
in zehn Sekunden, dann vor jedem Owner F9 plus drei Reset-Ticks. Aktionsticks
bleiben unverändert zusätzlich. Der Basisfall besitzt eine eigene Session und
seinen eigenen Engine-Startupvertrag. Zusätzlich zu den Startup-Ticks verwendet
Basis genau drei Map-, acht Fokus-/Text- und zwei Overlay-Aktionsticks; keine
Reparaturticks. Startup- und Terminalowner besitzen getrennte Sessions und
Fehlerbudgets. Das ist **keine Gesamtabnahme**; auch ein vollständig bestandener
L1-Teilumfang endet mit Exit **2**. Native Standardläufe wählen die drei eigenen
Headlessowner nicht aus. Öffentliche Konfiguration: [headless/app.toml](headless/app.toml).

Die Vorbereitung baut die rendererfreie CLI separat und `test_app` ausschließlich
mit `--no-default-features --features automation`, ohne den nativen gemeinsamen
Cargo-Featuregraph. Cargo-Artefakte müssen die passende Appvariante belegen;
`--artifacts` lehnt vertauschte native/Headless-Manifeste ab. Ohne `--artifacts`
bereitet der Runner selbst dieselben inhaltgeprüften Programme vor. Die gemeinsame
Appkonfiguration setzt `launch.no_default_features = true`,
`launch.headless = { width = 1440, height = 900, scale_factor = 1.0 }` und die
vorbereitete ausführbare Datei; `tests/app.toml` bleibt unverändert.
Quellen-, Asset-, Toolchain- und Programm-Hashprüfungen gelten unverändert.
Andere Auswahlflags, interaktiver Host und Feature-Matrix sind nicht Teil dieses
begrenzten L1-Modus. Der Liefervertrag steht in [L1.md](../docs/headless-plan/L1.md).

### Gemeinsame laufende Anwendung

Die zehn Keyboard-, Pointer- und Textfälle teilen im Runner eine App und Session.
Für mehrere aufeinanderfolgende Aufrufe hält [`interactive.py`](interactive.py)
diese App offen; ohne Auswahl laufen dort genau diese zehn Fälle, nicht die Gesamtsuite:

```sh
# Erster Aufruf: vorbereitete Programme verwenden, eine App starten und offen lassen.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.interactive --artifacts MANIFEST --category keyboard

# Weitere Aufrufe verwenden dieselbe App-PID; keine Builds oder neuen Fenster.
WOODPECKER_TEST_RENDERED=1 python3 -m tests.interactive --test tests.pointer.relative_motion
WOODPECKER_TEST_RENDERED=1 python3 -m tests.interactive --category text
python3 -m tests.interactive --status
python3 -m tests.interactive --stop
```

Der Host serialisiert die Aufrufe und beendet die App nur bei explizitem Stop
oder Hostende. Ein fehlgeschlagener Fall sperrt weitere Fälle; die App bleibt zur
Untersuchung offen. Veraltete Programme, geänderter Python-Code oder eine andere
Konfiguration erfordern einen ausdrücklich veranlassten Stop/Neustart, niemals
einen versteckten Ersatzprozess oder Neubuild. Isolierte Fälle werden vor jedem
Appstart abgelehnt. Ein Clientabbruch lässt den aktiven Fall abschließen, startet
aber keinen weiteren Fall. Erfolgreiche Teilläufe behalten Exit **2**.

Ein direkter `unittest`-Einzellauf verwendet dieselbe Fixture mit eigener Lebensdauer.
Der normale `tests.acceptance`-Runner beendet seine gemeinsame App weiterhin,
bevor er andere grafische Fälle startet.
Diese Fälle prüfen Eingaben und Spielverhalten, nicht kalten GPU-Startup:
Manual-Handshake, höchstens 120 ausdrücklich angeforderte Spielbereitschaftsticks
innerhalb von zehn Sekunden,
danach vor jedem Fall F9 und drei Reset-/Aufbereitungsticks. Die Aktionsticks des
jeweiligen Tests kommen zusätzlich hinzu. Es gibt keine OS-Eingaben.

F9 wirkt nur mit `--woodpecker-diagnostics --woodpecker-test-fixture`.
Die Fixture stellt Spielzustand, Settings, Textfelder, Fokus und virtuelle Inputs
wieder her; Zeit und Tickzähler laufen weiter. Der Kamera-Reset setzt Pose und
Projektionsgröße sofort zurück, statt alte Karten-/Zoom-Interpolation fortzusetzen.
Vor jedem Test werden Prozess-PID, Reset-Generation, leere Inputs und die
ursprüngliche Spieler-/View-/Settings-Baseline einschließlich echter Kameratransforms und Projektionen
überprüft. Ein Fehler stoppt die Gruppe. PID, Session und Falllogs stehen in
`summary.json/shared_application`; jeder Fall hat einen eigenen Logordner.
Der gemeinsame Session-Workspace ist ausdrücklich kein Nachweis von Dateisolation.

Andere Fälle bleiben isoliert: insbesondere Startup-/Shutdown-/Crash-, Mehrsession-,
Datei-/Recording-/Replay- und spezielle Diagnosekonfigurationen. Sie starten im
Normalfall dieselbe vorbereitete App-Binary erneut, nicht erneut den Compiler.
Panic-Abort wird in der Vorbereitung separat gebaut. Die Report-Snapshot-Fixture
ändert absichtlich die Package-Version und baut deshalb weiterhin ihren eigenen
Quellstand; der Dateilimit-Test benötigt seinen isolierten Cargo-Launcher zum
Wiederherstellen der Prozesslimits. Mutationstests bleiben separat.

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
wiederverwendbarer Cache, kein anderes normales Buildprofil. Die kleine enginefreie
Abort-Fixture verwendet das Produktionsprotokoll und echte Panic-Hooks. Feature-
Audits bauen in eigenen temporären Bäumen; ihre vollständigen Logs bleiben erhalten,
ihre Compilerbäume nicht. Unveränderte Audits dürfen ihren inhaltgeprüften Nachweis
wiederverwenden. Native Spiel- und Rust-Verträge verwenden gemeinsam aufgelöste
Dependency-Features; der explizite Headless-Appbuild bleibt davon getrennt.
Vorbereitete Programme behalten nur den neuesten vorbereiteten
Quellstand mit einer Programmversion je Rolle;
auch erneutes Linken desselben Codes darf nicht unbegrenzt Kopien erzeugen.
Gleiche Inhalte werden geteilt. Lese-Leases schützen Programme
laufender Test-Hosts und Runner auch über diese Grenze hinaus. Alte Manifeste und
Testbelege bleiben erhalten; deren nicht mehr vorgehaltene Programme sind kein
wieder ausführbares Archiv. Nicht durch eigene Manifeste bekannte Cacheeinträge
werden nicht automatisch gelöscht. Die isolierten
Einmal-Caches der [Mutationsprüfung](mutations.md#isolation-und-belege) werden
hingegen nach gesicherten Belegen entfernt. Laufartefakte sind lokal, nicht versioniert.

Das Entwicklungsprofil nutzt `debug = "line-tables-only"`; das davon erbende
Testprofil ebenso. Backtraces behalten Datei-/Zeilennummern, aber keine volle
Typ-/Variableninspektion im Debugger. Assertions und Overflow-Checks bleiben an.
Der eigenständige Live-Driver nutzt dieselbe Einstellung; die Report-Fixture
übernimmt die Workspace-Profile. Für gezielte Debuggersitzungen aktiviert
`CARGO_PROFILE_DEV_DEBUG=2` (bei `cargo test`: `CARGO_PROFILE_TEST_DEBUG=2`)
volle Debuginfo erneut und erzeugt damit zusätzliche Cachevarianten.

Normale Worktree-Caches bleiben getrennt: ein gemeinsames `target/` kann bei
parallelen Builds App-Binaries überschreiben. Nach der Profilumstellung bleiben
alte Artefakte zunächst liegen. Nur bei Bedarf einmal bereinigen; ein regelmäßiges
`cargo clean` würde den kompletten Neuaufbau immer wieder erzwingen.

Für angesammelte ältere Compiler-/Profilvarianten gibt es eine explizite,
selektive Bereinigung auf Basis einer erfolgreichen vollständigen Vorbereitung:

```sh
# SUMMARY ist die summary.json eines vollständigen --build-only-Laufs.
python3 -m tests._cargo_cache --preparation SUMMARY          # nur Löschplan
python3 -m tests._cargo_cache --preparation SUMMARY --apply  # geplante Rust-Ausgaben entfernen
```

Sie behält sämtliche in diesen Compilerlogs referenzierten Dependency-Ausgaben,
zwei neueste Incremental-Konfigurationen je noch benötigter Crate und die
vorbereiteten Programme mit Debug-Symbolen. Incremental-Caches nicht mehr im
aktuellen Compilergraph enthaltenen Crates werden vollständig entfernt. Die von Cargo nicht im JSON genannten gehashten Cachekopien
benötigter Binärtargets bleiben konservativ ebenfalls erhalten, damit der nächste
Prepare-Lauf nicht unnötig neu linkt. Sie greift nicht auf Quellcode, Testbelege,
andere Worktrees oder den globalen Cargo-Cache zu und lehnt aktive Builds in
diesen Debugcaches ab. Andere, nicht
in der Vorbereitung enthaltene Buildkonfigurationen müssen später gegebenenfalls
neu kompilieren. Für alle Aufrufe dieselbe Toolchain und Profile verwenden.

### 5-GB-Ruhebudget ohne Compileränderungen

```sh
RUSTUP_TOOLCHAIN=1.97.1 python3 -m tests.acceptance --build-only --compact
# Danach Programme über das ausgegebene products.json direkt verwenden:
RUSTUP_TOOLCHAIN=1.97.1 python3 -m tests.acceptance --category cli --artifacts MANIFEST
```

`--compact` bereinigt bei vollständiger Vorbereitung belegbar obsolete Compiler-
Varianten und packt die verbleibenden Cargo-Ausgaben verlustfrei. Es zählt danach
**das gesamte `target/` einschließlich früherer Belege** mit `du -sk`; mehr als
5.000.000.000 Bytes sind ein fehlgeschlagenes Speicherbudget. Programme und
macOS-Symbole bleiben unverpackt, laufende Verbraucher behalten ihre Leases.
Weder Profile noch Compilerflags werden dafür umgestellt. Der gemessene Ruhebestand
liegt nach freigegebener Bereinigung alter Logs unter dieser Grenze; den genauen
Stand dokumentiert der [Testsuite-Plan](../docs/testing-plan.md#frühere-erweiterung-vorbereitete-programme-und-kleinerer-buildgraph).

Das ist **kein 5-GB-Spitzenlimit beim Bauen**. Vor neuem Kompilieren braucht der
Cache seinen normalen entpackten Platz. Die Vorbereitung stellt ihn automatisch
wieder her; vor direktem `cargo build/check/test/clippy` ist ausdrücklich auszuführen:

```sh
python3 -m tests._compiler_cache --restore
```

Bytes, Dateimodi und Nanosekunden-Mtimes werden wiederhergestellt, damit unveränderte
Dependencies frisch bleiben. Prüfsummen und Cargo-Sperren schützen das Packen und
Wiederherstellen. Ein beschädigter Checkpoint wird weder verwendet noch gelöscht.
Die absichtlich selbst kompilierenden Snapshot-/Dateilimit-Verträge stellen den
Cache bei Bedarf vor ihrem Fall wieder her; nach sicherem Cleanup packt der
Runner ihn erneut. Normale vorbereitete Fälle benötigen diesen Schritt nicht.

Activity-Beobachtung nutzt fortlaufende Cursor und hält korrelierte, noch nicht
abgeholte Ergebnisse separat. Dadurch kopiert ein langer gemeinsamer Lauf nicht
bei jedem Command seine gesamte bisherige Activity erneut in die Falllogs.
Explizite Cursor und Gap-Fehler werden weiterhin unverändert respektiert.

## Einzeltests

Python-Dateien haben **kein `test_`-Präfix**. Bei Discovery deshalb `-p '*.py'`
oder einen konkreten Dateinamen verwenden; `-t .` hält den Repository-Root als
Importwurzel. Tests importieren gemeinsame Hilfen, keine anderen ausführbaren Fälle.

```sh
cargo build --locked -p woodpecker_cli --bin woodpecker

# Eine Datei ohne Spielfenster.
python3 -m unittest -v tests.cli.command_errors
python3 -m unittest discover -s tests/session -t . -p 'invalid_launch_config.py' -v

# Öffentlicher Rust-Vertrag ohne Session oder Fenster.
cargo test --locked --features cli --test contracts handle_encoding::handle_encoding_contract -- --exact --nocapture

# Alle 14 internen Owner-Tests ohne Spielfenster.
cargo test --locked --features cli,ui,screenshot,offscreen --lib -- --nocapture --test-threads=1
```

**Grafische Einzelfälle erst freigeben und mit denselben App-Features bauen:**

```sh
cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app

# Eigene Fixture: Spielbereitschaft, F9/3 Reset-Ticks, danach acht Aktionsticks.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.keyboard.movement

# Ein Manual-Fenster, keine Ticks/Inputs, nur lokaler Provider.
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli,ui,screenshot \
  --test contracts local_provider::local_provider_contract -- --exact --nocapture --test-threads=1
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

102 Produkttests: **78 Python + zehn öffentliche Rust + 14 interne Rust**.
101 sind lokal (davon drei zusätzliche Headlessowner), einer extern.
Mit Kontrolle 3.18 enthält das Inventar 103 Einträge;
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
| [headless](headless/) | 3 | Echte App-/UI-/Text-/Capturebilder, ursprüngliche Startup-/Fehlerowner und realer typisierter Geräteverlust; nur `--headless` |
| [screenshot](screenshot/) | 4 | Bild-Fixture, erster Tick, Overwrite und sichere Pfade; 3.13, 4.29–4.31 |
| [recording](recording/) | 3 | Dateivertrag, Barrieren und I/O-Fehler; 4.32–4.34 |
| [replay](replay/) | 5 | Frische Session, Vorvalidierung, Stop, Exklusivität und Blockierung; 3.14, 4.35–4.38 |
| [cli](cli/) | 5 | Config/JSON, Scripts, Pipeline und ungewisse Ergebnisse; 4.39–4.43 |
| [repl](repl/) | 3 | Eingabeformen, PTY/Terminal und Clientende; 4.44–4.46 |
| [reports](reports/) | 9 | Tracing, Panics/Abort, Exit, Marker und Provider-Fortschritt; 3.16–3.17, 4.47–4.53 |
| [shutdown](shutdown/) | 6 | Vorbedingungen, aktive Arbeit, Starting, Mehrsession, Frist und Signale; 4.58–4.63 |
| [providers](providers/) | 1 | Echter GitHub-Nachweis 4.81, ausschließlich separat |

[`rust/`](rust/) enthält ein Cargo-Target `contracts` mit zehn öffentlichen
Tests: `local_provider`, `github_provider`, `report_snapshot`, `report_signature`,
`pending_ownership`, `pending_drop`, `session_shutdown`, `history`, `event_overflow`,
`handle_encoding`. Jeder Fall läuft auch bei `cargo test` in einem eigenen,
seriell gestarteten Prozess; globale Hooks und Lifecycle-Zustand werden nicht
geteilt. Nur der letzte läuft ohne Spielfenster; Rust-Ownership,
Pending-Drop und direkte Session-Lebensdauer sind keine doppelten CLI-Tests.

Die 14 internen Tests liegen bei ihren tatsächlichen Ownern unter `src/`
(`cfg(test)`): native Inputs, Textzielbindung, Readback-Lebensdauer, Screenshot-
und Recording-/Replay-Pfadaustausch, Session-Identität, Request-ID-Erschöpfung,
Übergangsrennen, Reportabschluss (4.71–4.78, 4.82) und die getrennten Asset-
Nachrichtenströme für Tick und Renderframe. Dazu kommen drei fensterlose Ownerfälle
für den finalen Publicationcheck, Offscreen-Terminalguards und den Renderfehler-
Wirevertrag sowie ein CPUcheck des öffentlichen Headless-Viewporthelpers für
HiDPI, Imageidentität und Ablehnung vorhandener Windows. Der
[achtfache lokale GPU-/SDK-Nachweis](fixtures/offscreen/README.md)
ist separat und zählt nicht als automatische Standardausführung. Sie steuern reale
World-/Datei-/Koordinatorgrenzen instanzlokal, keine nachgebaute Produktlogik,
keine Winit-/GPU-Abnahme oder neue öffentliche API.

## Hilfen und Konfiguration

- [`_support.py`](_support.py): frische TOML-Daten und Roundtrip-geprüftes Schreiben,
  Serverprozess, CLI-Transport, Logs, Fristen und verifiziertes Prozess-Cleanup.
  `result` beobachtet genau einen angenommenen Request, ohne Resubmission oder
  Gap-Recovery. `call` sendet genau den angegebenen Command, `read_resource`
  genau eine lesende Query. Starts und Spielaktionen bleiben im Test sichtbar.
- [`_builds.py`](_builds.py): Standard-Cargo-Aufrufe, Compilerartefakte und Testlisten;
  [`_artifacts.py`](_artifacts.py): vorbereitete, inhaltgeprüfte Programme;
  [`_shared.py`](_shared.py): ausdrücklich begrenzte gemeinsame App-Fixture;
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
  `load_config`/`write_app_config`; letzteres wählt für die Standard-App eine
  vorbereitete Binary, lässt eigene Launch-Fixtures unverändert. Pfade gelten
  vom Repository-Root bzw. beim Seed
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
