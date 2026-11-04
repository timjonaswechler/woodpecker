# Testgruppen für möglichst wenige App-Fenster

## Zweck und Stand

Planungsgrundlage für die Wiederverwendung echter App-Instanzen. Zuerst die
thematische Zuordnung und ihre Voraussetzungen festhalten; **eine verbindliche
Reihenfolge ist noch nicht beschlossen**. Kein Runner- oder Produktionsumbau
ist Bestandteil dieses Dokuments.

Das aktuelle [Runner-Inventar](../tests/acceptance.py) enthält **96 Einträge**:
75 Python-Tests, zehn öffentliche Rust-Verträge, zehn interne Rust-Tests und
die typisierte Negativkontrolle `control::3.18`. Davon sind 95 lokal und einer
extern. Jeder Python-Modulname unten bezeichnet genau einen bestehenden Test;
Rust-/Control-IDs entsprechen dem Inventar. Alle 96 Einträge sind genau einmal
zugeordnet. Varianten innerhalb eines Tests sind keine zusätzlichen Test-IDs.

„Gemeinsam möglich“ bezeichnet die beabsichtigte Wiederverwendung **nach** den
genannten Anpassungen, nicht unveränderte Ausführbarkeit im heutigen Runner.
Aktuell teilen nur die zehn Keyboard-/Pointer-/Textfälle eine App:
[gemeinsame Fixture](../tests/_shared.py),
[bisheriger Persistenzmodus](../tests/interactive.py) und
[Ausführungsanleitung](../tests/README.md#gemeinsame-laufende-anwendung).
Die Gruppenanzahlen sind Testeinträge, **keine zugesagte Mindestfensterzahl**.

## Gemeinsame Haupt-App: 35 Kandidaten

Diese Fälle können nach dem Harness-Umbau dieselbe Manual-App und dieselbe
Session verwenden. Das ist eine quellenbasierte Kompatibilitätsplanung, kein
bereits bestandener gemeinsamer Grafiklauf. Nur die zehn Eingabetests sind
heute an die gemeinsame Fixture angeschlossen.

Die App erhält ein ausdrücklich gewähltes gemeinsames Profil: feste Tickdauer,
lokale Reports, Diagnostics, Prozessprobe, Reflection und Test-Fixture;
Tracing für die betreffenden Fälle aktiviert. Overlay zunächst aus.
Startprüfungen laufen ohne Warm-up; Spielbereitschaft und spätere Resets bleiben
sichtbare, gezählte Aktionen. Die thematischen Untergruppen sind **keine
Ausführungsreihenfolge**.

Bei `recording.file_contract` und `reports.tracing_report` bleibt zusätzlich eine
Abschlussprüfung nach dem tatsächlichen Serverende registriert. Die App dafür
nicht nach jedem Fall schließen, die Abschlussprüfung aber auch nicht streichen.

### Start vor dem ersten Tick

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.startup.manual_startup`](../tests/startup/manual_startup.py) | Manual-Start bleibt bei null Ticks und null Zeit. | Nur am echten Manual-Start, vor jedem Tick und Reset. |
| [`tests.screenshot.before_first_tick`](../tests/screenshot/before_first_tick.py) | Capture vor dem ersten Tick lehnt ab und erhält Dateien, Zeit und Spielzustand. | Gleiche kalte Manual-App; niemals nach Warm-up oder Reset. |

### Inspect und Reflection

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.inspect.resource_value`](../tests/inspect/resource_value.py) | Settings stimmen vollständig mit dem vorgegebenen Seed überein. | Settings-Baseline erhalten oder ausdrücklich wiederherstellen. |
| [`tests.inspect.resource_metadata`](../tests/inspect/resource_metadata.py) | Resource-Metadaten, Auswahl und nicht verfügbare Typen. | Reflection-Fixture beim Start aktivieren; nur Beobachtung. |
| [`tests.inspect.entity_query`](../tests/inspect/entity_query.py) | Entity-Filter und vollständige, geordnete Component-Projektionen. | Reflection-Fixture; während der Abfragen keine Ticks. Aktuelle Nulltick-Annahme berücksichtigen. |
| [`tests.inspect.reflection_values`](../tests/inspect/reflection_values.py) | Maps, Sets, Enums, Asset-Handles und Gründe für nicht verfügbare Werte. | Reflection-Fixture und ihre sessionlokale Handle-Referenzdatei. |
| [`tests.inspect.hierarchy_handles`](../tests/inspect/hierarchy_handles.py) | Hierarchietiefe, Kinderreihenfolge und Ablehnung ersetzter Entity-Handles. | F10 ersetzt die Hierarchie; neue Handles pro Fall erfassen, Tickdifferenzen prüfen. |

### Kontrolle der Testaussage

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`control::3.18`](../tests/inspect/resource_value.py) | Absichtlich falscher Spielername muss die genau erwartete Assertion auslösen. | Nur typisierte Erwartungsabweichung zulassen; kein Appfehler und kein neuer Start nötig. |

### Tastatur

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.keyboard.movement`](../tests/keyboard/movement.py) | Press/Hold/Release bewegen den Spieler nur auf expliziten Ticks. | Bereits gemeinsame Fixture; Spiel- und Eingabereset. |
| [`tests.keyboard.single_action`](../tests/keyboard/single_action.py) | Map-Toggle genau einmal je Press-Flanke. | Bereits gemeinsame Fixture; View-/Spielreset. |
| [`tests.keyboard.invalid_transitions`](../tests/keyboard/invalid_transitions.py) | Ungültige Tokens und Übergänge verändern keine akzeptierten Inputs. | Bereits gemeinsame Fixture; virtuelle Eingaben leer. |

### Maus und Pointer

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.pointer.buttons`](../tests/pointer/buttons.py) | Drei Maustasten bleiben unabhängig gedrückt und werden korrekt gelöst. | Bereits gemeinsame Fixture; Pointer-/Picking-Reset. |
| [`tests.pointer.relative_motion`](../tests/pointer/relative_motion.py) | Relative Bewegung, FPV-Kamera und unabhängiges Cursor-Clamping. | Bereits gemeinsame Fixture; Kamerapose und Projektion zurücksetzen. |
| [`tests.pointer.scroll`](../tests/pointer/scroll.py) | Vorzeichen, Achsen, Akkumulation und Leeren des Scroll-Deltas. | Bereits gemeinsame Fixture; Scroll-/Kamerareset. |
| [`tests.pointer.ui_click`](../tests/pointer/ui_click.py) | Absolute Pointerposition aktiviert den echten UI-Button erst nach einem Tick. | Bereits gemeinsame Fixture; gelayoutete UI und Grundzustand. |
| [`tests.pointer.invalid_input`](../tests/pointer/invalid_input.py) | Ungültige Positionen, Buttons und Übergänge beschädigen keine gültigen Inputs. | Bereits gemeinsame Fixture; Pointer-/Inputreset. |

### Texteingabe

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.text.unicode_input`](../tests/text/unicode_input.py) | Unicode und Zeilenumbrüche erreichen das fokussierte Notes-Widget. | Bereits gemeinsame Fixture; Text, Fokus und Settings zurücksetzen. |
| [`tests.text.input_limits`](../tests/text/input_limits.py) | Fokus, leerer Text, UTF-8-Bytegrenze und Widget-Limit unterscheiden sich. | Bereits gemeinsame Fixture; Text-/Fokusreset. |

### Ticks und Ausführungstempo

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.tick.exact_ticks`](../tests/tick/exact_ticks.py) | Exakte Tickzahlen und feste simulierte Zeit; danach kein Fortschritt. | Tick-/Zeitdifferenzen; keine Rücksetzung der SDK-Zeit. |
| [`tests.tick.stop_warp`](../tests/tick/stop_warp.py) | Paced- und Maximaltempo-Warp lassen sich früh stoppen und bleiben danach idle. | Keine laufende Arbeit am Fallende; Tickdifferenzen. |
| [`tests.tick.warp_errors`](../tests/tick/warp_errors.py) | Ungültige Warps und Pace-Werte beschädigen weder aktive Arbeit noch Idle. | Pending-Ergebnisse vollständig abholen; Pace-/Input-Baseline sichern. |
| [`tests.tick.set_pace`](../tests/tick/set_pace.py) | Request-Pace, Laufzeitänderung und Default-Pace ändern nicht die feste Tickdauer. | Render-/Spielbereitschaft; ursprüngliche Default-Pace wiederherstellen. |

### Screenshots

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.screenshot.capture`](../tests/screenshot/capture.py) | Echtes PNG zeigt die vorbereitete Szene ohne zusätzliche Simulation. | Geladene Standardszene; eigenes Capture-Ziel; Overlay aus. |
| [`tests.screenshot.overwrite`](../tests/screenshot/overwrite.py) | Zwei Captures ersetzen einen eigenen Sentinel unter demselben Dateinamen. | Gleiche Szene und eigener Dateibereich; kein neuer Prozess erforderlich. |
| [`tests.screenshot.invalid_path`](../tests/screenshot/invalid_path.py) | Ungültige Pfade und ausbrechende Symlinks verändern keine fremden Ziele. | Eigene Sentinel-/Symlink-Namen; danach vollständig entfernen. |

### Recording

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.recording.barriers`](../tests/recording/barriers.py) | Recording-Grenzen und Ablehnung bei aktiver oder offener Arbeit. | Recording und Warp am Ende beendet; eigener Dateibereich. |
| [`tests.recording.file_contract`](../tests/recording/file_contract.py) | Exaktes JSONL, Akzeptanzreihenfolge und typisierte Outcomes. | Tracing an; eigener Cursor/Dateibereich. Unveränderte Datei nach Serverende zusätzlich nachprüfen. |

### Replay

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.replay.invalid_recording`](../tests/replay/invalid_recording.py) | Ungültige Dateien und Pfade dürfen keinen gültig wirkenden Prefix ausführen. | Vorher-/Nachher-Zustand statt pauschal null Ticks; gültige Fixture-Clock beibehalten. |
| [`tests.replay.exclusivity`](../tests/replay/exclusivity.py) | Während Replay darf keine externe Arbeit in den Plan oder Inputzustand gelangen. | Replay/Recording beenden; Pending-Ergebnisse und Inputs prüfen. |
| [`tests.replay.stop`](../tests/replay/stop.py) | Stop erhält den ausgeführten Prefix, unterbindet den Tail und erhält wartende Inputs. | Explizite Release-/Cleanup-Ticks beibehalten; eigene Recording-Dateien. |

### CLI und REPL

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.cli.script_validation`](../tests/cli/script_validation.py) | Ungültiger Script-Tail wird vor jedem Prefix-Befehl abgelehnt. | Fallcursor und unveränderter Zustand; kein Shutdown wird tatsächlich ausgeführt. |
| [`tests.repl.commands`](../tests/repl/commands.py) | Help, lokale Parserfehler und lange/kurze REPL-Formen nutzen denselben Vertrag. | Neue Clientprozesse sind erlaubt; App bleibt gleich. Pace, Recording und Inputs bereinigen. |
| [`tests.repl.terminal`](../tests/repl/terminal.py) | PTY bleibt während Pending-Arbeit bedienbar und stellt nur sein eigenes Terminal wieder her. | Nur Test-PTYs; Warp stoppen, Client schließen; keine Desktop-Eingaben. |

### Client-Lebensdauer

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.session.disconnect`](../tests/session/disconnect.py) | Ein akzeptierter Warp läuft nach dem Ende des einreichenden Clients weiter. | Nur Client wechseln; Prozessprobe und relative Tickstände. |

### Nichtfataler Report

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.reports.tracing_report`](../tests/reports/tracing_report.py) | Tracing-Fehler erzeugt einen lokalen Report, die Session bleibt bedienbar. | Mit Abschlussphase: spätere Duplikate und dauerhafte Datei erst nach echtem Session-/Serverende prüfen. |

## Gemeinsame direkte SDK-App: 4 Kandidaten

Diese vier öffentlichen Rust-Verträge können nach Umbau einen gemeinsamen
direkten `Session`-Owner erhalten. Derzeit startet jeder Vertrag einen isolierten
Testprozess und seine eigene App; die gemeinsame `contracts`-Binary allein teilt
keinen Zustand.

Sie können **nicht einfach an die serververwaltete Haupt-App angehängt werden**.
Ein CLI-Test anstelle eines SDK-Tests würde Pending-/History-/Owner-Verträge nicht
gleichwertig prüfen. Prozessisolation für unterschiedliche globale Umgebungen
bleibt dort bestehen, wo sie benötigt wird. Die lokale Providerprüfung nutzt den
Session-Kontext auch nach Shutdown; ihre Dateimanipulationen gehören in eine
abgeschlossene, eigene Phase.

### Direktes Rust-SDK

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`rust::report_signature`](../tests/rust/test_report_signature.rs) | Signatur, Normalisierung und vollständiges Markdown. | Direkter SDK-Owner; vor anderen SDK-Commands, da der Report leere History nachweist. |
| [`rust::history`](../tests/rust/test_history.rs) | History ist auf 50 Einträge begrenzt; verdrängtes Pending bleibt aktiv. | Direkter SDK-Owner; bekannte Start-History berücksichtigen, keine künstliche History-Löschung. |
| [`rust::pending_drop`](../tests/rust/test_pending_drop.rs) | Drop eines Pending beendet weder Warp noch Recording. | Direkter SDK-Owner mit Prozessprobe; bekannte History-/Tickbasis statt fester Indizes. |
| [`rust::local_provider`](../tests/rust/test_local_provider.rs) | Lokales Create/Existing, konkurrierende Publikation und Pfadschutz. | Lokaler SDK-Kontext; nach Shutdown weiter nutzbar. Eigene Reportdateien/Manipulationen isolieren. |

## Andere Start- oder Serverprofile: 10 Einträge

Ein anderes Profil bedeutet nicht automatisch „ein Fenster pro Test“:
Erfolgreich gestartete Apps sollen anschließend kompatible Prüfungen bedienen.
Echte Startkonfigurationen, Limits und abweichende Quellstände dürfen jedoch
nicht durch einen warmen Reset vorgetäuscht werden. Die folgenden Einträge
enthalten zum Teil mehrere Varianten.

### Startprofile

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.startup.engine_startup`](../tests/startup/engine_startup.py) | Automatischer Engine-Start meldet gezählte Ticks und bleibt danach idle. | Eigener Engine-Start statt Manual-Profil; erfolgreiche App anschließend weiterverwenden. |
| [`tests.startup.startup_failures`](../tests/startup/startup_failures.py) | Startup-Voraussetzungen und Budgets scheitern mit den richtigen Diagnosen. | Vier echte Startvarianten mit erwarteten Fehlern; nicht durch warme App ersetzbar. |

### Application-Clock

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.tick.application_clock`](../tests/tick/application_clock.py) | Anwendungseigener 50-ms-Schritt, doppelte Geschwindigkeit und Pause. | Eigenes Application-Clock-Profil. Drei Modi könnten mit expliziter Test-Fixture in einer App wechseln; heute drei Starts. |

### Diagnosevarianten

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.reports.caught_panic`](../tests/reports/caught_panic.py) | Abgefangene System-/Task-/Worker-/panic_any-Panics bleiben nichtfatal. | Heute vier Startmodi. Nach explizit umschaltbarer Diagnose-Fixture gemeinsam möglich; Reportabschluss erhalten. |
| [`tests.reports.tracing_config`](../tests/reports/tracing_config.py) | Opt-in, Filter, aktiviertes Tracing, behandelter Fehler und fehlender Layer. | Enabled-Variante ist teilbar; disabled, Filter und fehlender Layer benötigen echte Startkonfigurationen. Handled-Modus separat umschalten. |

### Abweichender Server

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.session.activity_gap`](../tests/session/activity_gap.py) | Verdrängte Activity wird als unbekannt gemeldet; Pending bleibt beobachtbar. | Eigener Server mit 4096-Byte-Store. Kein pauschaler Ersatz des großen Stores der Hauptgruppe. |
| [`tests.recording.io_failure`](../tests/recording/io_failure.py) | Start-/Write-/Footer-I/O-Fehler und Recovery sind unterscheidbar. | Eigener Server mit Dateigrößenlimit und Restore-Launcher; ein echtes Fenster plus zwei fensterlose Peers. |
| [`tests.reports.slow_provider`](../tests/reports/slow_provider.py) | Ein blockiertes Fake-gh hält Spielarbeit und andere Clients nicht auf. | Eigener GitHub-Fake-Provider-Kontext; nach Freigabe weiter nutzbar, finaler Owner-Join bleibt Pflicht. |

### SDK mit eigener Umgebung

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`rust::github_provider`](../tests/rust/test_github_provider.rs) | Fake-gh: Create, Duplikate, Fehler und lokaler Fallback. | Direkter SDK-Owner mit eigener Umgebung und Provider-Konfiguration; alle Varianten bereits in einer App-Lebensdauer. |
| [`rust::report_snapshot`](../tests/rust/test_report_snapshot.rs) | Report bewahrt Paket-/Git-/Config-/History-Snapshot trotz späterer Änderungen. | Eigener gebauter Quellstand mit Version 0.56.1; nicht die normale vorbereitete App. Danach eventuell SDK-Gruppe wiederverwenden. |

## Mehrsession-Fälle: 8 Einträge

Hier sind zwei echte Sessions Teil des Nachweises. Ein kompatibles Paar soll
nach Möglichkeit zwischen nichtterminalen Fällen weiterleben. Das erfordert
geprüften Spiel-, Input- **und Dateizustand** in beiden Workspaces.
Bei terminalen Fällen bleibt höchstens die nicht beendete App wiederverwendbar.

Ein normaler In-Memory-Reset beweist weder Dateisolation noch einen frischen
Replay-Zielkontext. Die beiden Mehrsession-Shutdown-Fälle benötigen außerdem
jeweils ihren eigenen Endablauf und Provider-/Signalzustand. Das SDK-Paar kann
nicht durch das serververwaltete Paar ersetzt werden.

### Session-Isolation

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.session.session_isolation`](../tests/session/session_isolation.py) | Stop von A lässt B bereit und unverändert. | Zwei Sessions; A endet. B kann danach weiterverwendet werden. |
| [`tests.session.input_isolation`](../tests/session/input_isolation.py) | Keyboard, Pointer und Spielerzustand bleiben in ihrer adressierten Session. | Zwei geladene Apps; Paar mit geprüftem Reset für weitere Isolationstests nutzbar. |
| [`tests.session.file_isolation`](../tests/session/file_isolation.py) | Settings-Edit schreibt nur in den Workspace von A. | Zwei getrennte echte Workspaces; Dateireset nötig, In-Memory-Reset allein genügt nicht. |
| [`tests.session.request_correlation`](../tests/session/request_correlation.py) | Out-of-order-Ergebnisse und IDs bleiben im richtigen Client/Session-Kontext. | Zwei Sessions; beide dürfen unterschiedliche Tickstände haben. |

### Replay in zweiter Session

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.replay.recording_roundtrip`](../tests/replay/recording_roundtrip.py) | In A aufgenommene Bewegung reproduziert Checkpoints in B. | Zwei Sessions mit gleichem Ausgangszustand; Frische-/Dateitrennung nicht durch einen bloßen Spielreset ersetzen. |

### Mehrsession-Shutdown

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.shutdown.server_stop`](../tests/shutdown/server_stop.py) | Server-Stop leert unabhängige Sessions und einen gehaltenen Provider. | Zwei Apps plus Fake-gh; beendet den Server und die Apps. Eigenes terminales Szenario. |
| [`tests.shutdown.deadline`](../tests/shutdown/deadline.py) | Ein ursprüngliches Deadline-Budget gilt für alle Sessions und Provider. | Zwei Apps, Fake-gh und gezielte SIGSTOP-/Freigabe; zerstörerischer Deadline-Fall. |

### SDK-Pending-Besitz

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`rust::pending_ownership`](../tests/rust/test_pending_ownership.rs) | Gleiche numerische Request-ID verleiht keinen Besitz am fremden Pending. | Zwei direkte SDK-Sessions; keine CLI-Session als Ersatz. Paar nach Möglichkeit behalten. |

## Terminale Fälle: 11 Einträge

Diese Tests dürfen eine vorher passend genutzte App abschließen, statt immer
nur für ihren letzten Schritt eine neue App zu starten. Sie können aber nicht
alle nacheinander dieselbe App beenden: erfolgreicher Command-Shutdown,
Management-Stop, Panic, Exit und harter Drop sind unterschiedliche terminale
Pfade und brauchen jeweils eine passende Lebensdauer.

Nichtterminale Teilprüfungen dürfen vorgezogen werden. Start-/Endnachweise
bleiben dem ursprünglichen Test zugeordnet; ein Test ist erst nach allen seinen
Phasen und Abschlussprüfungen bestanden. Insbesondere den heutigen explizit
tick-/commandfreien Session-Lifecycle-Vertrag nicht still zu einem warmen
Gameplay-Vertrag umdeuten.

### Session beenden

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.session.session_lifecycle`](../tests/session/session_lifecycle.py) | Create, Inspect, Liste, Stop und tatsächliches Prozessende. | Echte Start- und Endphase derselben App; heutiger Nulltick-/Keine-Commands-Umfang darf nicht still entfallen. |
| [`tests.session.session_binding`](../tests/session/session_binding.py) | Vollständige IDs und eindeutige Prefixe binden vor und nach Stop dieselbe Session. | Kann eine kompatible App durch ihren Prefix-Stop abschließen; Endphase bleibt terminal. |
| [`tests.shutdown.command`](../tests/shutdown/command.py) | Offene Arbeit/Recording blockiert Shutdown; endgültiger Shutdown nutzt reservierte ID. | Nichtterminale Vorbedingungen teilbar; erfolgreicher Command-Shutdown beendet die Lebensdauer. |
| [`tests.shutdown.session_stop`](../tests/shutdown/session_stop.py) | Management-Stop drainiert Replay, Warp und Recording ohne Tail. | Kann eine kompatible App abschließen; vollständiger Footer-/Prozessnachweis. |

### CLI/REPL mit terminalem Ende

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.cli.script_pipeline`](../tests/cli/script_pipeline.py) | Script-Pipeline erhält Indizes, Ergebnisse und Grenzen trotz Rejection. | Enthält erfolgreichen Script-Shutdown: nur als Abschluss einer passenden App. |
| [`tests.repl.disconnect`](../tests/repl/disconnect.py) | quit/EOF/Ctrl+C trennen nur Clients; REPL-Shutdown ist terminal. | Client-Varianten auf derselben App; finaler Shutdown beendet sie. Technischer Peer-Endfall ist fensterlos. |

### Panic und Prozessende

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.reports.panic_exit`](../tests/reports/panic_exit.py) | Echte Panic scheitert am laufenden Request und hinterlässt genau einen Report. | Terminale F11-Aktion; passende App vorher wiederverwendbar, nachher nicht. |
| [`tests.reports.process_exit`](../tests/reports/process_exit.py) | Unerwarteter Exit 0 oder 17 ist weder Shutdown noch Panic. | Zwei echte Prozessenden, daher zwei Lebensdauern; Aktion aktuell beim Start festgelegt. |

### SDK-Ende und Überlauf

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`rust::event_overflow`](../tests/rust/test_event_overflow.rs) | 300 Tracing-Events überfüllen die Queue und beenden die eigene Prozessgruppe. | Direktes SDK ohne Event-Drain durch Server; terminales Spezialprofil. F9 kollidiert mit aktuellem Fixture-Reset. |
| [`rust::session_shutdown`](../tests/rust/test_session_shutdown.rs) | Sauberer SDK-Shutdown und Session-Drop während aktivem Warp unterscheiden sich. | Zwei SDK-Lebensdauern: sauberes Ende und harter Drop. Nur jeweils am Schluss einer passenden Gruppe. |

### Server-Signale

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.shutdown.signals`](../tests/shutdown/signals.py) | Erstes SIGINT/SIGTERM, zweites Ctrl+C und SIGKILL haben verschiedene Folgen. | SIGINT und SIGTERM benötigen je eine App-Lebensdauer. Weitere technische Peer-/Leerserver-Phasen ohne Fenster. |

## Separat freigegebener GitHub-Lauf: 1 Eintrag

Dieser Test bleibt außerhalb der lokalen Standardauswahl. Ein eigener
Freigabe-/Umgebungsbereich ist notwendig; das ist kein Grund für ein Fenster pro
Submit. Heute erfolgen beide Publikationsversuche bereits mit demselben,
zuvor beendeten Session-Kontext.

### Echter GitHub-Provider

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.providers.github_live`](../tests/providers/github_live.py) | Echter GitHub-Provider erstellt ein eigenes Issue und findet es erneut. | Eigene Freigabe, Repository und SDK-Umgebung; kein lokaler Defaultlauf. |

## Ohne natives Spielfenster: 27 Einträge

Diese Tests gehören nicht in die Fenster-Fixture. Einige benötigen Server,
externe Protokoll-Peers, eine rendererfreie Abort-Binary oder eine Bevy-World;
das ist nicht dasselbe wie eine native Fensterinstanz. Ihre Prozessisolation
darf unabhängig von der Fensteroptimierung bestehen bleiben.

### CLI, Konfiguration und Server

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.cli.command_errors`](../tests/cli/command_errors.py) | Ungültiges JSON/UTF-8/Command-Format sendet auch keinen gültigen Prefix. | Kein Spielprozess. |
| [`tests.cli.config_errors`](../tests/cli/config_errors.py) | Datei-/TOML-/Versionsfehler erzeugen weder Session noch Cargo-Aufruf. | Kein Spielprozess. |
| [`tests.cli.script_unknown`](../tests/cli/script_unknown.py) | Wire-/Korrelations-/Gap-Fehler erhalten bekannte Script-Ergebnisse ohne Retry. | Externer fensterloser Wire-Server. |
| [`tests.session.invalid_launch_config`](../tests/session/invalid_launch_config.py) | Ungültige Launch-Werte erzeugen keine Session. | Nur Server/Konfigurationsprüfung. |
| [`tests.session.invalid_session_id`](../tests/session/invalid_session_id.py) | Ungültige Session-IDs werden vor Lookup abgelehnt. | Nur Server. |
| [`tests.session.launch_failure`](../tests/session/launch_failure.py) | Launchfehler bleibt inspizierbar; der Server antwortet weiter. | Fehlender Launch, kein erfolgreich gestartetes Spielfenster. |
| [`tests.session.local_access`](../tests/session/local_access.py) | Remote-Bind und Browser-Origin werden abgelehnt, lokale CLI bleibt nutzbar. | Nur Server/HTTP. |
| [`tests.session.protocol_faults`](../tests/session/protocol_faults.py) | Fehlerhafte Handshakes und Antworten werden nicht als Erfolg gemeldet. | Fensterlose externe Peers. |
| [`tests.session.server_lifecycle`](../tests/session/server_lifecycle.py) | Leerer Server startet, listet und stoppt. | Ohne Session/App. |
| [`tests.session.unknown_session`](../tests/session/unknown_session.py) | Unbekannte Session bringt den Server nicht zum Stillstand. | Nur Server. |
| [`tests.startup.invalid_config`](../tests/startup/invalid_config.py) | Ungültige Startup-/Tick-Werte werden vor Session/Launch abgelehnt. | Kein Spielprozess. |

### Protokoll-, Report- und Abbruch-Fixtures

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`tests.replay.blocked`](../tests/replay/blocked.py) | Neuer Protokollfehler blockiert den Tail, historische Fehler sind kein Oracle. | Fensterloser Protokoll-Peer. |
| [`tests.reports.before_ready`](../tests/reports/before_ready.py) | Marker und Exit vor Ready bleiben Launchdiagnostik, kein Report. | Fensterlose externe Peers. |
| [`tests.reports.invalid_marker`](../tests/reports/invalid_marker.py) | Fragmentierte gültige und ungültige Marker bleiben unterscheidbar. | Fensterlose externe Peers. |
| [`tests.reports.panic_abort`](../tests/reports/panic_abort.py) | Echtes panic=abort flusht den Produktionshook vor Prozessende. | Eigene rendererfreie Abort-Binary, kein Spielfenster. |
| [`tests.shutdown.starting`](../tests/shutdown/starting.py) | Stop vor Ready räumt den eigenen Start ohne verspätetes Ready/Report auf. | Fensterloser angehaltener externer Peer. |

### Handle-Encoding

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`rust::handle_encoding`](../tests/rust/test_handle_encoding.rs) | Handle- und Command-Encoding erhalten vollständige Index-/Generationsbits. | Weder App, World noch Renderer. |

### Interne Verträge

| Test | Prüft | Voraussetzung / Anpassung |
| --- | --- | --- |
| [`internal::report_completion_joins_all_jobs_before_server_done`](../src/server/tests/test_report_completion.rs) | Reportjobs sind vor Server-Done vollständig gejoint. | Interner fensterloser Vertrag. |
| [`internal::session_identity_collisions_preserve_roots_and_reject_ambiguous_prefixes`](../src/server/tests/test_session_identity.rs) | ID-Kollisionen erhalten Workspaces und lehnen mehrdeutige Prefixe ab. | Interner fensterloser Vertrag. |
| [`internal::exhausted_ids_never_admit_work_or_lose_pending_and_reserved_shutdown`](../src/session/coordinator/tests/test_request_id_exhaustion.rs) | ID-Erschöpfung verliert weder Pending noch reservierten Shutdown. | Interner fensterloser Vertrag. |
| [`internal::native_input_cannot_enter_controlled_ticks_or_release_virtual_holds`](../src/session/input/tests/test_native_input.rs) | Native Inputs gelangen nicht in kontrollierte Ticks. | Bevy-World ohne natives Fenster. |
| [`internal::accepted_text_stays_bound_across_focus_despawn_and_editability_changes`](../src/session/input/text/tests/test_target_binding.rs) | Akzeptierter Text bleibt an sein ursprüngliches Ziel gebunden. | Bevy-World ohne natives Fenster. |
| [`internal::recording_and_replay_pin_parents_and_reject_swapped_final_entries`](../src/session/recording/file/tests/test_path_replacement.rs) | Recording/Replay pinnen Eltern und lehnen ausgetauschte Ziele ab. | Interner Dateisystemvertrag ohne Fenster. |
| [`internal::failed_or_expired_readbacks_never_write_or_complete_a_new_request`](../src/session/screenshot/capture/tests/test_readback_lifecycle.rs) | Fehlgeschlagene/veraltete Readbacks schreiben nicht in neue Requests. | Interner Capture-Vertrag ohne natives Fenster. |
| [`internal::screenshot_path_replacement_never_reaches_the_outside_sentinel`](../src/session/screenshot/destination/tests/test_path_replacement.rs) | Pfadaustausch erreicht keinen Sentinel außerhalb des Workspace. | Interner Dateisystemvertrag ohne Fenster. |
| [`internal::transition_races_obey_real_file_barriers_and_never_release_cancelled_plans`](../src/session/tests/test_transition_races.rs) | Transition-Races beachten Dateibarrieren und geben abgebrochene Pläne nicht frei. | Interner fensterloser Vertrag. |
| [`internal::render_asset_messages_survive_tick_aging_without_changing_application_readers`](../src/session/throughput/test_asset_messages.rs) | Render-Asset-Nachrichten überleben Tick-Alterung ohne geänderte App-Reader. | Bevy-World ohne natives Fenster. |

## Speed-/Performance-Prüfungen außerhalb des Inventars

Quelle: [Messwerkzeug](../examples/performance.py) und
[Messbedingungen](performance.md#wiederholbar-messen).
Nicht als neue Abnahmefälle mitzählen.

| Prüfung im Werkzeug | Gemeinsame App möglich? | Voraussetzung |
| --- | --- | --- |
| Wiederholte Maximaltempo-Warps, Tickzahl und simulierte Zeit | Ja, innerhalb eines kontrollierten Messprofils. Heute bereits ein Game für alle Repeats. | Gleiche Szene, Clock, Fenster-/Displaykonfiguration und definierter Warm-up; getrennte Messfenster. |
| Pause: null TPS, eingefrorene Zeit/Ticks, weiterlaufendes Rendering | Ja, dieselbe kontrollierte App. | Vorheriger Warp vollständig abgeschlossen; ausreichend getrenntes Samplerfenster. |
| Inspect-/Stop-Latenzen während langer Warps | Ja, dieselbe kontrollierte App. | Offene Warps exakt auflösen; Latenzen nicht mit Build-/Startkosten vermischen. |
| `--verify`: Overlay/F8, Paced-/Maximaltempo-Fortschritt, Captures | Ja, dieselbe kontrollierte App. | Overlay/Pace/Inputs zurückstellen; Captures erst nach der jeweiligen Messung. |
| `--native`: native FPS ohne kontrollierte Session, optional uncapped | Nicht dieselbe laufende kontrollierte App. | Separates natives Startprofil; alle kompatiblen nativen Beobachtungen darin bündeln. |
| Referenz-/aktueller Binary-Vergleich über `--binary` und `--compare` | Nicht dieselbe Prozessinstanz für verschiedene Binaries. | Pro Binary eine Lebensdauer, darin alle Wiederholungen; gleiche Messbedingungen. |

Das Messwerkzeug besitzt die Spiel-Pipes direkt, die Haupt-App dagegen gehört
dem Server. Eine Wiederverwendung **zwischen diesen beiden Einstiegen** braucht
zuerst einen gemeinsamen Owner/geeigneten Mess-Adapter; ein zweiter Leser an
denselben Pipes ist keine Lösung. Bei einer Transportänderung Messdefinition,
Overhead und Baseline neu prüfen. Nicht einfach CLI-Laufzeit mit bisherigen
direkten Warp-Zeiten vergleichen. Bis dahin ist „ein Fenster für alle
Wiederholungen je Messprofil“ der bereits vorhandene sinnvolle Umfang.

## Weitere grafische Anwender-Beispiele

Diese [Beispiele](../examples/game_tests/README.md) sind keine Abnahmefälle und
sollen weiterhin unabhängig von Suite-Hilfen bleiben:

| Beispiel | Einordnung |
| --- | --- |
| [`movement.py`](../examples/game_tests/movement.py) | Bewegungsablauf auf einer vorbereiteten App; mit explizit übergebenem Session-Kontext und passendem Reset prinzipiell teilbar. Heute eigener Create/Stop. |
| [`feedback.py`](../examples/game_tests/feedback.py) | Zustandsabhängige Bewegung; derselbe Wiederverwendungsbedarf wie Movement. |
| [`text.py`](../examples/game_tests/text.py) | Reale UI-/Texteingabe; benötigt eigene Text-/Fokus-/Settings-Baseline. |
| [`replay.py`](../examples/game_tests/replay.py) | Bewusst frischer Zielkontext B nach A; heute zwei aufeinanderfolgende Lebensdauern, nie zwei gleichzeitig offene Fenster. |

Normale Rust-/Harness-/Spiel-Unit-Tests ohne natives Fenster bleiben außerhalb
dieser Fensterplanung. Der [Modell-Zoo](../test_zoo/README.md) ist eine
eigenständige Vorschau-/Export-App, kein automatisierter Abnahmefall.

## Abhängigkeiten für die spätere Reihenfolge

Noch kein Ablaufplan; diese Bedingungen begrenzen die mögliche Reihenfolge:

- **Nulltick-Phase vor dem ersten echten Tick.** Manual-Startup und
  Before-First-Tick-Capture dürfen weder Warm-up noch F9-Reset vor sich haben.
  Nach einem Tick lässt sich diese Bedingung nicht durch Nullsetzen eines Zählers
  wiederherstellen. Ein gezielt gewählter Einzeltest mit dieser Bedingung braucht
  deshalb einen frischen Manual-Start.
- **Manual- und automatischer Engine-Startup sind verschiedene Startverträge.**
  Ein Manual-Start mit späteren normalen Warps ersetzt keinen echten
  Engine-Startup-Nachweis. Beide erfolgreich gestarteten Apps dürfen anschließend
  kompatible Tests tragen; zwei verschiedene Startverträge müssen nicht zu
  ständig neuen Starts für deren Folgetests führen.
- **Vorbedingungen statt Testketten.** Ein einzelner ausgewählter Test soll nur
  seine explizite Vorbereitung erhalten, nicht ungefragt alle thematischen
  Vorgänger ausführen. Resetfähige Fälle innerhalb einer Phase sollen unabhängig
  voneinander bleiben; geänderte Reihenfolgen müssen prüfbar sein.
- **Zähler und Zeit fortführen.** Normale Funktionstests prüfen Differenzen
  gegen den bekannten Fallanfang. Wo echter Nullzustand, leere History oder
  identische Request-IDs Vertragsbestandteil sind, den Originalnachweis erhalten
  und entsprechend platzieren oder eine eigene Lebensdauer verwenden.
- **Spielreset ist kein vollständiger Sessionreset.** Die heutige Fixture setzt
  Gameplay, Settings, Text/Fokus und virtuelle Inputs zurück, aber nicht SDK-Zeit,
  History, Reportdateien/-jobs, Activity oder sämtliche Diagnose-Fixtures.
  Recording/Replay/Pending, Default-Pace, Dateien/Symlinks und Diagnosezustand
  brauchen eigene explizite Vor-/Nachbedingungen. Bekannte kaputte Zustände nicht
  durch Reset verdecken.
- **F9 hat zwei Bedeutungen.** Fixture-Reset und ReportProbe hören heute beide
  auf F9. Nicht einfach alle Diagnoseflags kombinieren: für gemeinsame
  Caught-/Handled-/Overflow-Probes ist eine getrennte, explizite Teststeuerung
  nötig. Eine umschaltbare Test-Fixture ersetzt keine startupabhängigen
  Filter-/Layer-/Clock-Verträge.
- **Beobachtungen je Fall abgrenzen.** Activity-Cursor, Request-Zuordnung und
  eigene Dateibereiche statt kompletter alter Journale verwenden.
  Derselbe Tracing-Fehler kann dieselbe Reportsignatur erzeugen:
  `Created` versus `Existing` darf nicht vom Zufall der Reihenfolge abhängen.
  Den dafür nötigen eigenen Dateizustand explizit herstellen.
- **Reportabschluss nicht vorwegnehmen.** Alle erwarteten Providerjobs vor dem
  nächsten konfliktträchtigen Fall beobachten; dauerhafte Effekte und
  Nicht-Duplizierung zusätzlich nach dem tatsächlichen Owner-/Server-Join
  bestätigen. Bei verschobenen Endprüfungen bleibt der Test bis dahin offen.
- **Messungen nicht verfälschen.** Performance braucht einen definierten
  Szenen-/Messzustand ohne nebenläufige Providerarbeit oder zusätzliche
  Diagnoseaktionen. Native/controlled, VSync/uncapped und andere Binaries getrennt
  nachweisen; keine neue universelle Fensterzahl vorab versprechen.
- **Terminale Aktion zuletzt in ihrer App-Lebensdauer.** Start-/Live-/Endphasen
  getrennt planen und Ergebnisse trotzdem pro ursprünglichem Test ausweisen.
  Eine verbleibende zweite Session nicht unnötig schließen, wenn ihr Zustand für
  weitere Tests nachweislich passt.
- **Fehler bleiben sichtbar.** Unerwarteter Fehler oder unbestätigter Cleanup
  sperrt weitere Nutzung; kein stiller Neustart oder Retry. Die typisierte
  Negativkontrolle ist nur in ihrer exakt erwarteten Form kein Appfehler.
  Einzeltests müssen weiter einzeln auswählbar und belegbar bleiben.

## Prüfung dieser Bestandsaufnahme

- Zuordnung gegen `tests.acceptance.inventory()` abgleichen: 96 Einträge,
  keine unbekannte ID, kein fehlender oder doppelt zugeordneter Test.
- Quellen für Phasen und Einschränkungen sind direkt in den Tabellen verlinkt;
  bestehende Ausführungs-/Freigaberegeln im [Testplan](testing-plan.md#verbindliche-testregeln)
  bleiben unverändert.
- Die gemeinsame Ausführung der zusätzlichen Kandidaten wurde **nicht**
  grafisch erprobt. Aus der Gruppierung folgt noch kein bestandener Lauf und
  kein geänderter Runner.
