# Abnahmesuite

Diese Tests sichern Woodpecker für uns als Projekt ab. Verständliche Anwender-
Tests für das eigene Spiel folgen erst nach der Abnahmesuite getrennt unter
`examples/`; sie sind nicht der aktuelle Zweck dieses Ordners.

Jeder Test liegt in einer eigenen Datei. Aktuell sind 94 Abnahmefälle geschrieben:
75 Python-Tests, zehn öffentliche Rust-Integrationstests und neun interne Rust-
Modultests. 93 gehören zum lokalen Sollumfang, 4.81 separat zum echten GitHub-
Nachweis. 3.18 verwendet dieselbe Resource-Testimplementierung als zusätzliche
Kontrolle, kein weiterer Testfall. 4.79 integriert den gemeinsamen Einstieg;
4.80 (Gesamtlauf) und die echte Ausführung von 4.81 bleiben offen.
Aktuelle Einzelbelege und verbleibende Gates stehen im
[full-stop-Abnahmeabschnitt](../docs/testing-plan.md#full-stop-abnahme-und-offene-checkpoints).
Die Laufnotizen in den folgenden Modulbeschreibungen dokumentieren frühere Etappen,
nicht den neuesten Gesamtstand. Kein bestandener gemeinsamer Finalstand wird behauptet.
Sieben Pilotfälle prüfen die Session-Verwaltung:

- [`session/server_lifecycle.py`](session/server_lifecycle.py):
  Start → leere Session-Liste → Serverstop, ohne Anwendung oder Fenster.
- [`session/session_lifecycle.py`](session/session_lifecycle.py):
  Signalstation anlegen → `SessionReady` abwarten → Zustand und Liste
  abfragen → Sessionstop → `Ended` abwarten → Serverstop. App-PID/Gruppe vor
  Stop verifizieren und danach unabhängig als verschwunden prüfen. Dieser Test öffnet ein
  Fenster, sendet aber keine Eingaben oder Tick-Aufträge. `SessionReady` bedeutet
  hier Steuerungsbereitschaft, nicht ein fertig geladenes oder geprüftes Spielbild.
- [`session/session_isolation.py`](session/session_isolation.py):
  Zwei versetzte Manual-Sessions starten, nur A beenden; B muss danach einen
  frischen Inspect-Spielcommand beantworten und weiterhin null App-Ticks zeigen.
  B separat beenden. Keine Inputs/Ticks. **Verschärfte Fassung grafisch offen;**
  frühere Lebensdauerbelege gelten nicht als neuer Lauf dieser Fassung.
- [`session/unknown_session.py`](session/unknown_session.py):
  Eine gültig formatierte, nicht vorhandene Session-ID abfragen; Exitstatus 1 und
  `session_not_found` prüfen. Der Server muss danach weiterhin abfragbar sein und
  sauber stoppen. Ohne Anwendung oder Fenster; **live geprüft**.
- [`session/invalid_session_id.py`](session/invalid_session_id.py):
  Zu kurze, zu lange und nicht-hexadezimale IDs mit `invalid_session_id` ablehnen;
  Server bleibt nutzbar. Ohne Anwendung oder Fenster; **live geprüft**.
- [`session/invalid_launch_config.py`](session/invalid_launch_config.py):
  Gültiges TOML mit leerem Package-Namen vor Session-Erzeugung ablehnen; die Liste
  bleibt leer. Ohne Anwendung oder Fenster; **live geprüft**.
- [`session/launch_failure.py`](session/launch_failure.py):
  Angegebene, aber fehlende Manifestdatei führt nach Annahme zu `Failed`; Diagnose
  bleibt abfragbar und der Server läuft weiter. Ohne Spielstart; **live geprüft**.

- [`inspect/resource_value.py`](inspect/resource_value.py):
  Signalstation mit dem festen Settings-Seed bis `SessionReady` starten;
  `inspect.query` einmal einreichen und das zugehörige terminale Ergebnis abfragen.
  Erwartet wird der vollständige lesbare Settings-Wert, einschließlich des vom
  App-Default abweichenden Spielernamens. Öffnet ein Fenster, ohne Input oder
  Tick-Auftrag. **Im nutzergestarteten Pilotlauf bestanden.**

- [`startup/manual_startup.py`](startup/manual_startup.py):
  Nach `SessionReady` Anwendungsticks und Bevy-`Time<()>` lesen, 0,25 reale
  Sekunden warten und erneut lesen. Erwartet werden anfänglich null Ticks/Zeit
  und anschließend unveränderte Werte. Der reflektierte App-Zähler
  `SimulationTicks` zählt Last-Schedule-Durchläufe unabhängig vom privaten
  SDK-Zähler. Öffnet ein Fenster ohne Input- oder Tick-Aufträge.
  **Im nutzergestarteten Pilotlauf bestanden.**

- [`tick/exact_ticks.py`](tick/exact_ticks.py):
  Drei Ticks maximal schnell und sieben mit 20 Ticks/s anfordern. Completion,
  unabhängigen App-Zähler und feste simulierte Zeitdifferenzen prüfen; nach jedem
  Warp 0,25 reale Sekunden warten und Stillstand prüfen. Insgesamt zehn explizite
  Ticks in einem Fenster, keine Inputs oder Warp-Stop-Commands.
  **Im nutzergestarteten Pilotlauf bestanden.**

- [`startup/engine_startup.py`](startup/engine_startup.py):
  Engine-Startup ausdrücklich konfigurieren, `EngineReady` samt Tickzahl und
  unabhängigem App-Zähler prüfen; danach Stillstand. Spielbereitschaft getrennt
  über `LevelLoad` prüfen, nötige zusätzliche Ticks sichtbar anfordern.
  **Im separat freigegebenen Einzellauf mit freier Fenstermitte bestanden:**
  drei Startup-Ticks, keine zusätzlichen Bereitschaftsticks, normales Cleanup.
  Test und Konfiguration einschließlich `--start-unfocused` blieben unverändert.
  [Belege einschließlich früherer Timeouts im Arbeitsplan](../docs/testing-plan.md#bisherige-belege-keine-neue-gesamtabnahme).
- [`keyboard/movement.py`](keyboard/movement.py):
  Press/Hold bewegt den Spieler, Release stoppt ihn; vorher keine Wirkung.
  Vollständige Warp-Outcomes und unabhängiger App-Zähler prüfen die Phasen
  +1/+2/+3/+4/+5/+8; auf jedem einzelnen Hold-Tick muss Bewegung sichtbar sein. Acht Aktionsticks nach sichtbarer Engine-/Spielvorbereitung,
  kein Reset oder OS-Input. **Verschärfte Fassung grafisch offen.**
- [`screenshot/capture.py`](screenshot/capture.py):
  Nach expliziter Vorbereitung genau ein PNG ohne Simulationsfortschritt.
  Bewusst feste Bild-Fixture: Signalstation-Startszene, 1440×900, Skalierung 1,
  unveränderte HUD-/Boden-/Startpad-Regionen und RGB-Grenzen. Version und Grenzen
  werden protokolliert; visuelle App-Änderungen erfordern eine überprüfte
  Fixture-Änderung, keine gelockerten Bänder. **Aktuelle Fassung grafisch offen;**
  frühere Pixel-/Sichtbelege bleiben historisch.

Engine-Startup, Tastaturbewegung und Screenshot erzeugen ihre Engine-Variante aus
`app.toml` in den jeweiligen
Laufartefakten: höchstens 3600 Startup-Ticks in 60 Sekunden, anschließend höchstens
120 ausdrücklich angeforderte Spielbereitschaftsticks in zehn Sekunden. Der
Bewegungstest fordert danach acht Aktionsticks an, der Screenshot keine weiteren.
Jeder Fall startet genau eine App und benötigt seine passende Grafikfreigabe.

- [`replay/recording_roundtrip.py`](replay/recording_roundtrip.py):
  Zwei frische Sessions mit gleicher Clock, Settings und Spielerstartposition,
  aber räumlich versetzten Fenstern vorbereiten;
  fünf Ticks samt Eingaben und drei Checkpoints in A aufnehmen, Datei sichtbar
  nach B kopieren und dort wiedergeben. Eine zweite Recording belegt die realen
  Replay-Zwischenstände, ohne verbotene externe Inspects während Replay. Relative
  Tick-/Zeitdifferenzen und Endposition vergleichen. Keine Entity-Handles werden
  wiederverwendet. Beide Sessions haben die oben genannten Vorbereitungsbudgets.
  **Im freigegebenen Restblock mit beiden Positionsvarianten bestanden.**
- [`reports/tracing_report.py`](reports/tracing_report.py):
  Manual, Diagnose/tracing, kindlokal RUST_LOG=error, F12 mit zwei Ticks.
  Lokalen Report samt Signatur, weitere Nutzbarkeit und Haltbarkeit prüfen.
  Danach Session bei noch laufendem Server ausdrücklich stoppen und Activity
  bis Ended begrenzt auf weiterhin einen Failure/Report prüfen. Ended ist keine
  Providerbarriere: finale Dateien nach Serverexit, Queue-Abschluss intern in 4.82.
  **Verschärfte Fassung grafisch offen; keine externe Veröffentlichung.**

- [`reports/panic_exit.py`](reports/panic_exit.py):
  Eine Manual-Session mit App-Diagnose starten, F11 drücken und genau einen Tick
  anfordern. Dieser Request muss terminal fehlschlagen; genau ein Panic-Failure,
  sein lokaler Report und der App-Exit 101 müssen beobachtbar sein. Kein weiterer
  ProcessExit-Failure. Server-Exit 1 mit `shutdown_incomplete` ist wegen der
  gespeicherten `Failed`-Session erwartet; erzwungenes Harness-Cleanup nicht.
  Der Report bleibt nach Serverende lesbar. Keine Release-/Reparatur-Ticks.
  **Neue PID-/Gruppen- und finale Dateimengenprüfung grafisch offen; historischer
  Restblock einschließlich absichtlichem App-Absturz bestanden.**

Die Diagnoseaktion in `test_app/src/diagnostics.rs` ist abgeschlossene Vorbereitung
3.15, kein weiterer Test. Ihre Wirkung wird in 3.16/3.17 geprüft. Ohne `automation`, kontrollierte Session und
`--woodpecker-diagnostics` ist sie deaktiviert. **F11 löst bei Opt-in eine
beabsichtigte Panic aus**. Beide Diagnosewirkungen sind durch 3.16/3.17 live belegt.

Vorbereitung 4.1 ergänzt getrennte opt-in-Argumente für Startup-Grenzfälle, eine
App-eigene Clock und Reflection-Beispieldaten; die
[Argumente und Daten](../test_app/README.md#vorbereitung-weiterer-abnahmefälle-41)
stehen bei der Referenz-App. Startup-/Clock-Modi sind durch 4.2/4.4 live geprüft.
Die Reflection-Modi sind gebaut; ihre Funktionsnachweise in 4.16–4.19 bleiben
offen. Bestehende Pilotbelege ersetzen diese Nachweise nicht.

### Ergänzungen aus Schritt 4

- [`startup/startup_failures.py`](startup/startup_failures.py), **4.2**:
  Vier Startup-Diagnosevarianten nacheinander, je eigener Server/App: fehlender
  Asset-Scope, unbekannte Asset-Liste, fehlendes Asset und deaktivierte Kameras.
  Passender StartupFailed-Event/Fehler, kein EngineReady; maximal 3600 Startup-Ticks
  in zehn Sekunden je Variante, pending nur zwei Ticks. Keine externen Ticks oder
  Eingaben. Erwarteter Server-Exit 1 wegen Failed-Session, kein erzwungenes Cleanup.
  **Im freigegebenen Lauf bestanden:** alle vier Diagnosen nach 0/2/1/3 Ticks,
  kein EngineReady; vier erwartete Server-Exits 1 ohne erzwungenes Cleanup.
- [`startup/invalid_config.py`](startup/invalid_config.py), **4.3**:
  15 ungültige Startup-/Tick-Werte bei unveränderter gültiger Launchkonfiguration
  ablehnen. Die Session-Liste bleibt leer; ein Cargo-Tripwire im Server-Kindprozess
  verhindert einen Appstart auch bei einer Regression und darf nicht aufgerufen
  werden. Nichtendliche TOML-Werte dürfen bereits an der JSON-Grenze scheitern.
  **Ohne Spielstart bestanden; falsche Erwartung und Startbarriere separat geprüft.**
- [`tick/application_clock.py`](tick/application_clock.py), **4.4**:
  Drei frische Manual-Sessions nacheinander mit SDK-Clock application und App-Clock
  steady/double/paused. Je drei Ticks ergeben 0,15/0,30/0 Sekunden virtuelle Zeit;
  Zähler, Delta, Pause und relative Geschwindigkeit prüfen. Ohne Warp bleibt
  nach 0,25 Sekunden alles unverändert. Keine Inputs oder Vorbereitungsticks.
  **Im freigegebenen Lauf bestanden:** alle drei Clock-Varianten, neun echte
  App-Ticks, erwartete Zeit/Policy und anschließender Stillstand; normales Cleanup.

- [`tick/stop_warp.py`](tick/stop_warp.py), **4.5**:
  Eine Manual-Session ohne Inputs. 600 Ticks bei 10 Ticks/s und anschließend
  50.000 maximal schnelle Ticks anfordern, nach echtem App-Fortschritt ausdrücklich
  stoppen. Stopped-Teilzahl muss zum App-Zähler passen; nach je 0,25 Sekunden keine
  weiteren Ticks oder Zeitänderungen. Natürlicher Abschluss vor Stop belegt keine
  Unterbrechbarkeit und führt nicht zu einem automatischen Wiederholungsversuch.
  Nur vollständig gezählte natürliche Completion bei idle Stop ergibt
  Unvollständig nach Cleanup; partieller/ignorierter Stop bleibt rot.
  **Neue Klassifikation grafisch offen; historisch bestanden:** nach 3/600 bzw. 4/50.000 Ticks gestoppt;
  App-Zähler und anschließender Stillstand bestätigt, normales Cleanup.
- [`tick/set_pace.py`](tick/set_pace.py), **4.6**:
  Engine-/Startupbudget unverändert, maximal 124 weitere Ticks ohne Inputs.
  Vorhandene atomare App-Dateiprobe ausdrücklich aktivieren: 16 Kalibrierungsticks
  bei 8/s, 100er-Warp mit Override 2/s und Wechsel auf 8/s früh stoppen,
  acht Ticks mit geerbtem Default. Zwei 2,5-Sekunden-Messfenster verwenden nur
  Datei-Beobachtungen mit monotonen Leseklammern, keine CLI-Laufzeiten.
  Unter 6 beobachteten Ticks/s oder bei zu wenigen beobachteten Intervallen
  **Unvollständig/Skip**, niemals Erfolg. Die spätere Untergrenze verwendet die
  gemessene Basiskapazität, nicht blind den idealen Zielwert. Mindestdurchsatzannahmen sind
  keine Produktgarantie: nicht trennbare Rate-/alte-Pace-Hypothesen bleiben
  unvollständig. Obergrenzen mit einer Tickphase statt breiter Prozenttoleranz.
  Fixed-Zeit bleibt pro echtem Tick gleich.
  **Geänderte Fassung grafisch offen.**
- [`tick/warp_errors.py`](tick/warp_errors.py), **4.7**:
  Eine Manual-Session ohne Inputs. Null-Ticks sowie Request-Pace 0/−1 im Idle
  ablehnen; bei aktivem 100er-Warp mit 2 Ticks/s zweiten Start und SetPace 0/−1
  ablehnen. Der ursprüngliche Warp muss danach weiterarbeiten. Ausdrücklich
  stoppen; erneuter Idle-Stop verändert weder Ticks noch Zeit. Ein frischer Tick
  ohne Override belegt weitere Nutzbarkeit. Maximal 101 gültig angeforderte Ticks.
  **Im freigegebenen Lauf bestanden:** sechs passende Ablehnungen, ursprünglicher
  Warp nach drei Ticks gestoppt, Idle-No-op und frischer Tick bestätigt;
  normales Cleanup.

- [`session/session_binding.py`](session/session_binding.py), **4.8**:
  Eine Manual-Session ohne Ticks/Inputs über volle ID und eindeutiges
  Acht-Zeichen-Präfix verwalten und per WebSocket ansprechen. Nach ausdrücklich
  angefordertem Sessionstop bleibt derselbe Ended-Eintrag über beide Selektoren
  lesbar; keine neue Session oder Umwahl.
  **Im freigegebenen Lauf bestanden; normales Cleanup.**
- [`session/request_correlation.py`](session/request_correlation.py), **4.9**:
  Zwei versetzte Manual-Sessions, keine Inputs. A fordert 60 Ticks bei 2 Ticks/s
  an und wird früh gestoppt; B führt drei maximal schnelle Ticks aus. Frische
  CLI-Prozesse reichen Requests ein: gleiche Zahlenwerte in beiden Sessions,
  aber unterscheidbare Warp- und Inspect-Ergebnisse. A-Inspect schließt vor dem
  früher angenommenen A-Warp ab. Annahme-/Abschlusszuordnung je Session prüfen.
  **Im freigegebenen Lauf bestanden:** A-Abschlüsse [2,1,3], B [1,2],
  trotz gleicher IDs richtige Werte/Outcomes; normales Cleanup.
- [`session/disconnect.py`](session/disconnect.py), **4.10**:
  Manual, zwölf Ticks bei 2/s, keine Inputs, vorhandene App-Dateiprobe.
  Der Einreicher endet mit Pending. Nach mindestens zwei App-Ticks während
  0,75 Sekunden ohne CLI-/REPL-Verbindung ausschließlich den atomaren Dateizähler
  lesen; Fortschritt vor jedem Reconnect bestätigen. Danach genau eine
  ursprüngliche Annahme/Completion und zwölf natürliche Ticks prüfen. Fehlender
  Fortschritt im kurzen Messfenster bleibt trotz späterer Completion Unvollständig.
  **Verschärfte Fassung grafisch offen.**
- [`session/input_isolation.py`](session/input_isolation.py), **4.11**:
  Zwei versetzte Sessions; je höchstens 3600 Startup-Ticks in 60 Sekunden und
  120 explizite Spielbereitschaftsticks in zehn Sekunden. Danach zwei Aktionsticks
  je Session. Nur A erhält arrow_right und Pointer-MoveTo [720,450] mit mittlerer
  Press/Release. B jeweils zuerst ticken: Spieler, Keyboard-/Mouse-ButtonInput und
  Pointerzustände dürfen sich nur in A ändern. Reine öffentliche Inspect-Abfragen;
  keine neue App-Fixture, keine OS-Eingabe und keine fremden Entity-Handles.
  **Im freigegebenen Lauf mit beiden Positionsvarianten bestanden:** je drei
  Startup-/zwei Aktionsticks; nur A verändert, B unverändert; normales Cleanup.

- [`session/file_isolation.py`](session/file_isolation.py), **4.12**:
  Zwei versetzte Sessions mit je höchstens 3600 Startup-/120 Bereitschaftsticks.
  Eigene Workspace-Dateien vor der Änderung bestätigen; nur A erhält Tab und
  Texteingabe „!“ mit vier Aktionsticks. Ein B-Tick, App-Werte und Dateien zeigen
  Isolation; persönliche Settings nur per Hash prüfen, niemals schreiben.
  **Im freigegebenen Lauf bestanden:** A-Name Woodpecker-Abnahme! gespeichert,
  B und persönliche Settings unverändert; Dateien nach normalem Shutdown erhalten.
- [`session/activity_gap.py`](session/activity_gap.py), **4.13**:
  Manual, 4096 Byte Activity, 100er-Warp bei 2/s früh stoppen, keine Inputs.
  Höchstens 64 sichtbare Metadata-Queries in 20 Sekunden bis zu belegtem Gap
  und verdrängtem Warp-Pending, keine feste Annahme über 24 Eventgrößen.
  Neuer REPL-Snapshot muss die konkrete Pending-ID und ihren Command zeigen;
  Überschriften sind kein Oracle. Expliziter Resume-Cursor, keine Resubmission
  oder automatische Reparatur. **Geänderte Fassung grafisch offen.**
- [`session/local_access.py`](session/local_access.py), **4.14**:
  Wildcard-Bindung ablehnen; reservierter Loopback-Port verhindert selbst bei
  Regression einen exponierten Listener. Drei Origin-Varianten auf Verwaltungs-
  und Stop-Route ablehnen; normale lokale CLI bleibt nutzbar. Keine Session/App.
  **Fensterlos bestanden; isoliert falscher HTTP-Sollstatus wurde erkannt.**
- [`session/protocol_faults.py`](session/protocol_faults.py), **4.15**:
  Zehn externe Protokollfehler über die echte CLI: vier Handshakefehler sowie
  beschädigte, falsch benannte/typisierte, unbekannte und doppelte Antworten.
  [`faults/protocol_peer.py`](faults/protocol_peer.py) ist ein reiner JSONL-Peer,
  kein Bevy-Spiel. Nur der externe Cargo-run-Aufruf für ein temporäres Sentinel-
  Paket wird ersetzt; Metadaten stammen von echtem Cargo. Kein Game-Fallback.
  **Alle zehn Varianten fensterlos bestanden, erwartete Server-Exits 1/0;
  sämtliche Fehlerprozesse ohne erzwungenes Cleanup eingesammelt.**

- [`inspect/resource_metadata.py`](inspect/resource_metadata.py), **4.16**:
  Eine Manual-/Reflection-Session ohne Ticks/Inputs. Resource-All/Type und
  Metadata/Value prüfen: Sortierung, wertfreie Metadaten auch für NaN/opake Werte,
  registriert-fehlend gegenüber unbekannt und nur explizit auswählbare nicht
  reflektierbare Resources. Beide All-Projektionen müssen unabhängig alle fünf
  bekannten Fixture-Resources einschließlich UnsupportedMapKey enthalten;
  zusätzliche Bevy-Resources bleiben erlaubt. Für All-Value ausdrücklich 16 MiB Activity.
  **Vorherige Fassung live bestanden; verschärfte Mengen-Assertions offline an
  gespeicherten Antworten geprüft, erneuter grafischer Lauf offen.**
- [`inspect/entity_query.py`](inspect/entity_query.py), **4.17**:
  Eine Manual-/Reflection-Session ohne Ticks/Inputs. Entity-Summary, Namen,
  All/Listed, with/without und unbekannte Pfade prüfen. Entdeckte unregistrierte
  Components, nicht reflektierbare und fehlende Werte bleiben unterscheidbar;
  interne Resource-Entities fehlen, vollständige World-Summary bleibt unverändert.
  **Im freigegebenen Lauf bestanden, null Ticks/Inputs.**
- [`inspect/hierarchy_handles.py`](inspect/hierarchy_handles.py), **4.18**:
  Eine Manual-/Reflection-Session. Bekannten Vier-Entity-Baum bei Tiefen 0/1/2/255
  abfragen, Children-Reihenfolge gegen Handle-Sortierung prüfen. F10-Press/ein Tick
  ersetzt den Baum, Release/ein Tick beendet die Flanke. Neue Handles, gleiche
  Beziehungen; alte/fehlende Handles dürfen nicht auf andere Entities zeigen.
  **Im freigegebenen Lauf bestanden, genau zwei Ticks und fünf Handle-Ablehnungen.**
- [`inspect/reflection_values.py`](inspect/reflection_values.py), **4.19**:
  Eine Manual-/Reflection-Session ohne Ticks/Inputs. Maps, sortierte Sets, Enum-
  Formen sowie Pfad-/UUID-/flüchtige Handles und Typed/Untyped-Konsistenz prüfen.
  Zwei verschiedene reservierte IDs gegen native Fixture-Bits aus
  `reflection-handle-bits.json`, nicht nur auf Format/Gleichheit testen.
  NaN, nicht serialisierbare Map-Keys und opake Werte liefern begründetes
  Unavailable; serialisierbares opakes Null bleibt ein lesbarer Nullwert.
  **Neue Zwei-Handle-Assertion grafisch offen; historischer Lauf mit einem
  flüchtigen Handle bestand bei null Ticks/Inputs und unveränderter Zeit.**

- [`keyboard/single_action.py`](keyboard/single_action.py), **4.20**:
  Eine Session mit höchstens 3600 Startup-/120 Bereitschaftsticks, danach sieben
  Aktionsticks mit g: Gesamtkarte einschalten, drei Held-Ticks einzeln prüfen,
  loslassen, erneut drücken und loslassen. Nur Press-Flanken ändern den echten
  View-Modus, Hold/Release nicht. Keine Wiederholung des Bewegungstests.
  **Im freigegebenen Lauf bestanden: drei Startup-, keine Bereitschafts-, sieben Aktionsticks.**
- [`keyboard/invalid_transitions.py`](keyboard/invalid_transitions.py), **4.21**:
  Eine Manual-Session, vier Ticks, gültige ShiftLeft-Press/Release. Doppelte Press,
  unbegründete ControlLeft-Release und unbekannte Tokens bei vorgemerkter und
  angewandter Press ablehnen, ohne ButtonInput oder vorgemerkte Arbeit zu ändern.
  Doppelte vorgemerkte Release ist die neunte erwartete Ablehnung; danach
  korrekte Release-Flanke und leerer Idle-Zustand, kein Fortschritt durch Fehler.
  **Geschrieben; grafischer Funktionsnachweis offen.**

Die Pointer-Ergänzungen sind geschrieben, aber noch nicht grafisch ausgeführt:
- [`pointer/ui_click.py`](pointer/ui_click.py), **4.22**: Echten Kartenbutton
  über vorhandenen Text „Gesamtkarte [G]“ und öffentliche Hierarchie/Layout finden,
  nicht über TabIndex. MoveTo/Press/Release in drei Ticks; Vorabwirkung,
  Pressed-Component und echte Map-Aktion prüfen. **Grafisch offen.**
- [`pointer/relative_motion.py`](pointer/relative_motion.py), **4.23**: Neun
  Aktionsticks; vollständige Motion-/FPV-Sollwerte, Summierung/Null und beide
  Cursorgrenzen. Der positive Rand muss weniger als 0,001 logische Pixel unter
  der exklusiven Obergrenze liegen; kein bestimmter f32-Nachbarwert erforderlich.
  MoveTo erzeugt keine relative Motion, Idle wiederholt nichts. **Grafisch offen.**
- [`pointer/buttons.py`](pointer/buttons.py), **4.24**: Manual, acht Ticks.
  Left/Right/Middle jeweils einzeln drücken und anwenden, dann Hold, drei
  einzelne Releases und Idle. Nach jeder Press und Release Picking-/Bevy-Zustand
  einschließlich Flanken prüfen; vertauschte Einzelzuordnung darf nicht in der
  Gesamtmenge verschwinden. **Grafisch offen.**
- [`pointer/scroll.py`](pointer/scroll.py), **4.25**: Manual, sechs Ticks.
  Vorzeichen, Achsen, Null und Addition in AccumulatedMouseScroll mit Line-Einheiten;
  Idle löscht Delta, Position/Buttons bleiben unverändert.
- [`pointer/invalid_input.py`](pointer/invalid_input.py), **4.26**: Manual,
  vier Ticks, 19 erwartete Ablehnungen. Fehlende Position, verletzte Grenzen,
  unbekannte Buttons und falsche Übergänge dürfen vorgemerkte oder angewandte
  Zustände nicht verändern; neue gültige Arbeit bleibt ausführbar.

4.22/4.23 erlauben vor den Aktionsticks jeweils höchstens 3600 Engine-Startup-Ticks/
60 Sekunden und 120 ausdrücklich sichtbare Spielbereitschaftsticks/zehn Sekunden.
Jeder Fall öffnet ein Fenster, verwendet ausschließlich virtuelle Eingaben und
benötigt seine eigene frische Grafikfreigabe.

Die Text-/Screenshot-Ergänzungen sind geschrieben, aber grafisch noch offen:
- [`text/unicode_input.py`](text/unicode_input.py), **4.27**: Acht Aktionsticks.
  Einstellungen über bestehenden Buttontext, Notizen über vorhandenen Name
  per Pointer finden/fokussieren; keine TabIndex-Auswahl. Komplexes Unicode
  vormerken, zwei feste Edit-/Sync-Ticks und leeres Keyboard prüfen.
  **Grafisch offen.**
- [`text/input_limits.py`](text/input_limits.py), **4.28**: 18 Aktionsticks,
  4 MiB Activity; Ziele über vorhandene Buttontexte beziehungsweise Namen
  auswählen. Fehlender editierbarer Fokus, leere Eingabe, exakt 16384 gegenüber
  16385 UTF-8-Bytes und eigene 24-Zeichen-Grenze des Namensfelds unterscheiden.
  Keine Retry-Ticks oder Trunkierung. **Grafisch offen.**
- [`screenshot/before_first_tick.py`](screenshot/before_first_tick.py), **4.29**:
  Manual, null Ticks/Inputs, genau ein Capture. Erwartet
  screenshot_window_unavailable und unveränderte testeigene Sentinel-Datei,
  Zeit und Simulation, ohne versteckten Render-Bootstrap.
- [`screenshot/overwrite.py`](screenshot/overwrite.py), **4.30**: Zwei Captures,
  keine Aktionsticks/Inputs. Zwei gültige vollständig dekodierte PNGs und
  overwritten false/true; ein zwischenzeitlich testeigener Nicht-PNG-Sentinel
  verhindert Scheinerfolg durch Wiederverwenden des ersten Bilds. Benötigt Pillow.
- [`screenshot/invalid_path.py`](screenshot/invalid_path.py), **4.31**: Keine
  Aktionsticks/Inputs, zwölf geplante Ablehnungen für syntaktisch ungültige und
  über Symlinks ausbrechende Pfade. Nur eigene In-/Outside-Ziele, unveränderte
  Dateien/Links auch nach Shutdown, keine pauschale Ablehnung aller Symlinks.

Außer 4.29 verwenden diese Fälle dieselbe begrenzte Engine-/Spielvorbereitung
wie 4.22/4.23. Pro Fall ein echtes Fenster und eigene frische Grafikfreigabe.

Die Recording-/Replay-Ergänzungen:
- [`recording/file_contract.py`](recording/file_contract.py), **4.32**: Manual,
  F12/tracing mit lokalem nichtfatalem Report; maximal 61 Ticks. Zehn Einträge
  gegen Header/Footer, Annahmereihenfolge und vollständige typgerechte Outcomes,
  obwohl spätere Antworten früher eintreffen. Controls, Events/Reports und
  Request-IDs gehören nicht in die Datei. **Grafisch offen.**
- [`recording/barriers.py`](recording/barriers.py), **4.33**: Manual, zwei
  60er-Warps bei 2/s früh stoppen. Start-/Stop-Fehler, Header-/Footer-Grenzen
  gegenüber direkt folgenden Queries, Exists-/Pfad-/Symlink-Ablehnungen und
  frische Aufnahme. Gezielt verzögerte Übergangsrennen bleiben bei 4.78.
  **Grafisch offen.**
- [`recording/io_failure.py`](recording/io_failure.py), **4.34**: Ein Manual-
  Spiel ohne Ticks/angenommene Inputs und zwei externe Peers. Nur der eigene
  Server erhält Dateigrößenlimit 65536; Cargo/Spiel/Parent bleiben unverändert.
  Vier recordbare Text-Requests mit je 16384 festen ASCII-Bytes müssen weiterhin
  text_focus_unavailable liefern, während allein die Aufnahme an ihr Limit stößt.
  Serverlogs unter Limit, genau ein RecordingFailed und Recovery prüfen.
  **Neue F0-Kontrolle der festen Schreiblast bestanden** (65536 Bytes, unveränderte
  Rejections, Kindlimit restauriert); **vollständiger Spielnachweis offen**.
  Die gesunden/fehlerhaften externen Endphasen bleiben Bestandteil des Falls.
- [`replay/invalid_recording.py`](replay/invalid_recording.py), **4.35**:
  Manual, null Ticks, 33 ungültige Datei-/Clock-/Pfadvarianten. Kein Präfix wird
  eingereicht, World bleibt unverändert; gültige leere Dateien mit/ohne LF und
  neue Key-Queue prüfen Recovery. **Loader-Varianten mit externem Peer bestanden,
  grafischer World-Nachweis offen.**
- [`replay/stop.py`](replay/stop.py), **4.36**: Manual, zwei echte Präfix-Ticks,
  ShiftLeft und 120er-Warp bei 2/s früh stoppen. Beobachtungs-Recording schließt
  Tail-Ausführung aus; Stopped/Stop/Idle-No-op, erhaltene Effekte und zwei
  ausdrückliche Cleanup-Ticks, maximal 124 Ticks. **Grafisch offen.**
- [`replay/exclusivity.py`](replay/exclusivity.py), **4.37**: Manual, 120er-Warp
  bei 2/s früh stoppen. Neun abgelehnte externe Requests, nur Warp/interner Stop
  im Beobachtungs-Recording; keine fremden Input-/Dateieffekte. **Grafisch offen.**
- [`replay/blocked.py`](replay/blocked.py), **4.38**: Zwei externe Protokoll-Peers,
  keine Spiele. Neue technische Störung blockiert den Plan mit Ursache und ohne
  Tail; historisch abweichende Outcomes aller fünf Formen sind keine Sollwerte.
  **Beide Varianten fensterlos bestanden.**

Alle sechs grafischen Ergänzungen verwenden Manual-Startup, je ein echtes
Spielfenster und eigene frische Freigabe. 4.34 startet zusätzlich zwei externe
Fehler-Peers; seine Dateigrößengrenze gilt nicht für Parent, Cargo oder Spiel.

Die CLI-/Script-Ergänzungen:
- [`cli/config_errors.py`](cli/config_errors.py), **4.39**: Acht Datei-/TOML-/
  Versionsfehler vor Session-Erzeugung, leere Liste/Artefakte und rein kindlokaler
  Cargo-Tripwire. **Fensterlos bestanden.** Fachliche Config-Owner bleiben getrennt.
- [`cli/command_errors.py`](cli/command_errors.py), **4.40**: Sechzehn JSON-/
  UTF-8-/Command-Formen inklusive roher argv-Bytes. Jeweils ausdrücklich Exit 1,
  strukturierter invalid_json/invalid_command-Fehler, kein Pending oder Bindversuch
  am eigenen Loopback-Listener. **Verschärfte Fassung mit frisch gebauter CLI
  im neuen 15-Test-Lauf fensterlos bestanden.**
- [`cli/script_validation.py`](cli/script_validation.py), **4.41**: Manual,
  null Ticks, neun fehlerhafte Dokumente hinter gültigem g-Press-/Ein-Tick-Präfix.
  Vollständige Vorvalidierung mit Index, leere Activity und unveränderte World;
  leeres gültiges Script und frische g-Press/Release ohne Tick prüfen Recovery.
  **Parserfälle fensterlos geprüft; grafischer World-/Queue-Nachweis offen.**
- [`cli/script_pipeline.py`](cli/script_pipeline.py), **4.42**: Manual, ein
  lesendes Passed-Script und ein gemischtes Zehn-Command-Script: zwei Ticks bei 2/s
  vor Recording, 60er-Warp bei 2/s früh stoppen, Inspect vorher abgeschlossen,
  eine geplante invalid_key-Ablehnung, ein weiterer Tick, Recording Stop und
  normales Shutdown. Maximal 63 Ticks, keine angenommene Eingabe. Vollständige
  Outputs/Indizes, Activity-Barrieren und fünf Recording-Einträge unabhängig prüfen.
  **Schemata akzeptiert; grafische Ausführung offen.**
- [`cli/script_unknown.py`](cli/script_unknown.py), **4.43**: Fünf externe
  WebSocket-Peers mit EOF bei unbestätigtem Submit, falschem Submit-/Poll-Call,
  Gap und falschem Event-Command. Bekanntes Ergebnis bleibt, offene/unbestätigte
  Arbeit ist `unknown`, Tail `not_submitted`; IDs bleiben nur bei bestätigter
  Annahme erhalten. Offene Fehlerverbindungen decken auch versteckte Recovery-RPCs
  auf; keine neue Einreichung, kein Stop/Shutdown. **Alle fünf fensterlos bestanden.**

4.41/4.42 öffnen je ein echtes Fenster und benötigen eigene frische Grafikfreigabe.
Die externen Peers aus 4.43 sind keine Spiele und führen keinen Tick aus.

Die REPL-Ergänzungen sind geschrieben, aber grafisch nicht ausgeführt:
- [`repl/commands.py`](repl/commands.py), **4.44**: Manual, drei Ticks, keine
  Spielinputs. Echte Pipe-REPL mit help/pending und sechs kopierbaren Inspect-
  Beispielen, 14 Parsefehlern und ungültigem UTF-8 ohne neue Activity. Aufnahme
  prüft 13 vollständige/Kurzform-Übersetzungen, dynamischen Entity-Handle und
  Type Path/Pfade mit Leerzeichen. Der unbekannte Type-Pfad muss gezielt mit
  `rejected / unknown_type_path` und exakt erhaltenem Pfad enden; die Aufnahme
  muss dieselbe Ablehnung enthalten, nachfolgende Queries laufen weiter.
  Leerer Replay fügt keine Ticks hinzu.
- [`repl/terminal.py`](repl/terminal.py), **4.45**: Manual, 120er-Warp bei 2/s
  früh stoppen. Eigene 44-Spalten-PTY, Hilfe/Pending während offener Arbeit,
  Unicode-Editierung und schmale Redraws; Fremdantwort darf die angefangene
  Zeile nicht zerstören. quit und fehlende Session prüfen Terminal-Restaurierung.
  Keine Desktop-Eingabe oder Änderung der Spielfenstergeometrie.
- [`repl/disconnect.py`](repl/disconnect.py), **4.46**: Eine Manual-Session,
  je zwölf Ticks bei 2/s nach quit, Pipe-EOF und Ctrl+C in eigener PTY natürlich
  vollenden: 36 Ticks. Kein impliziter Stop/Shutdown; eigener letzter Client
  fordert normales Shutdown an. Anschließend ein externer Fehler-Peer ohne Spiel:
  EOF während offener Arbeit muss REPL-/Server-Exit 1 ergeben.

Alle drei Fälle benötigen eigene frische Grafikfreigabe, je ein echtes Fenster,
insgesamt drei serielle Spielstarts. Externe Pipe-/PTY-/Fehler-Kontrollen sind
belegt, ersetzen aber keine grafischen Spiel-/Ticknachweise.

Die Report-/Provider-Ergänzungen:
- [`reports/tracing_config.py`](reports/tracing_config.py), **4.47**: Fünf Manual-
  Starts, vier Ticks. Opt-in aus/an, weggefiltertes Error, fehlende Layer-Bestätigung
  und bestätigtes behandeltes Err ohne Report. **Grafisch offen.**
- [`reports/caught_panic.py`](reports/caught_panic.py), **4.48**: Vier Manual-Starts,
  je zwei Ticks. Abgefangene System-/Task-/Worker-Panics und unbekannter panic_any-
  Payload, getrennte Meldung/Location/Backtrace und lebender Prozess. **Grafisch offen.**
- [`reports/panic_abort.py`](reports/panic_abort.py), **4.49**: Separater
  panic=abort-Automation-Build im wiederverwendbaren Cache `target/panic-abort`,
  eigene Logs je Lauf; ein Manual-Fenster, F11/ein Tick, lokaler Report.
  Neben Panic-Zuordnung und fehlendem doppeltem ProcessExit-Failure muss das
  tatsächliche Endevent SIGABRT ausweisen; Unwind-Exit 101 genügt nicht.
  **Grafischer Abort-Nachweis offen; isolierte Signal-Assertions geprüft.**
- [`reports/process_exit.py`](reports/process_exit.py), **4.50**: Zwei Manual-
  Starts, je ein F9-Tick, unerwarteter Exit 0/17. ProcessExit erst nach Pipe-Drain,
  kein erfundener Panic. **Grafisch offen.**
- [`reports/invalid_marker.py`](reports/invalid_marker.py), **4.51**: Acht externe
  Marker-Varianten ohne Spiel: geteilter gültiger Marker, JSON/Checksum/Base64,
  doppelte/fehlende Chunks, EOF-Fragmente und unzulässiger ProcessExit-Payload.
  Ungültiges bleibt Byte-Diagnose/ObservationError; nur gültiges wird entfernt.
  **Alle acht fensterlos bestanden.**
- [`reports/before_ready.py`](reports/before_ready.py), **4.52**: Fünf externe
  Vor-Ready-Varianten. Eigene PID-/Mode-Datei belegt den tatsächlichen Peer-Start;
  invalid_ready samt defekter Zeile beziehungsweise launch samt passendem
  Exitstatus und Markertext konkret prüfen. Kein Ready, Failure, Report oder
  Command-Kontext. **Verschärfte Fassung fensterlos bestanden.**
- [`reports/slow_provider.py`](reports/slow_provider.py), **4.53**: Ein Manual-
  Fenster, F12/drei Ticks. Externes gh kontrolliert halten, während getrennte
  Clients zwei weitere Ticks und Settings lesen; dann zugeordnetes Outcome.
  Kein echtes gh/Netzwerk. **Queue-/Client-F0-Kontrolle bestanden, grafisch offen.**
- [`rust/test_local_provider.rs`](rust/test_local_provider.rs), **4.54**:
  Eine Manual-Session ohne Ticks/Inputs; sechs parallele Created-/Existing-
  Veröffentlichungen, Markdown, Konflikte und sichere Pfade/Symlink-Sentinels.
- [`rust/test_github_provider.rs`](rust/test_github_provider.rs), **4.55**:
  Eine Manual-Session ohne Ticks/Inputs; neun externe gh-Szenarien mit Pagination,
  exakten offenen/geschlossenen Treffern, Decoys und expliziten Fallback-Ursachen.
- [`rust/test_report_snapshot.rs`](rust/test_report_snapshot.rs), **4.56**:
  Eine Manual-Session ohne Ticks, ShiftLeft-Queue; eigene App-/Asset-/Git-Fixture,
  Metadatenwerte gegen Startkonfiguration prüfen; danach History/Branch/Commit
  ändern, alten Report unverändert prüfen. Optionale Toolchain-Werte dürfen null sein.
- [`rust/test_report_signature.rs`](rust/test_report_signature.rs), **4.57**:
  Eine Manual-Session ohne Ticks/Inputs; Golden Vectors, Diagnose-/Nachrichten-
  Identität, Titel/Unicode und vollständige Markdown-Nutzdaten über öffentliche
  Report-Werte; Abschnittsfolge, Kontext-JSON, jede Fence-Schließung und Dokumentende.

Die vier Report-Rust-Targets sind in `Cargo.toml` verdrahtet, mit je einem echten Test.
Sie sind kompiliert und ihre Grafikguards geprüft, **nicht grafisch ausgeführt**.
Externe API-Kontrollen bestätigen vier Golden Vectors, Snapshot-Unveränderlichkeit
und neun gh-/Fallback-Szenarien; sie ersetzen keinen vollständigen R1-Nachweis.
Die neuen [App-Proben](../test_app/README.md) wirken nur mit Diagnose-Opt-in.

Für den ganzen Block wären 17 serielle echte Appstarts mit maximal einem Fenster
und 18 expliziten Ticks nötig, darunter absichtliche Panics/Abort/Exits. Dafür
fehlt eine frische Grafikfreigabe. **Keine echten Issues:** Github-Tests dürfen
nur das ausdrücklich vorbereitete gh-Fehlerwerkzeug verwenden.

Die Shutdown-Ergänzungen:
- [`shutdown/command.py`](shutdown/command.py), **4.58**: Ein Manual-Fenster,
  120er-Warp bei 2/s früh stoppen. Direkter Shutdown bleibt bei offenen Commands
  bzw. Recording ohne ID blockiert; danach einmal terminaler MAX-ID-Abschluss,
  keine neue Annahme oder Fehlerreports. **Grafisch offen.**
- [`shutdown/session_stop.py`](shutdown/session_stop.py), **4.59**: Ein Manual-
  Fenster, aktiver Replay-Warp 120 bei 2/s mit gleichzeitigem Recording. Echter
  App-Fortschritt belegt den offenen Warp, Verwaltungs-Stopp verhindert den Tail,
  stoppt fachlich und schreibt den Footer vor Shutdown. Ein zusätzlicher Idle-Stop
  ist zulässig, nicht Voraussetzung. **Grafisch offen.**
- [`shutdown/starting.py`](shutdown/starting.py), **4.60**: Externer Prozess hält
  nachweislich vor Ready. Cancel/Ended statt Fehlerreport, PID reaped, keine
  Annahme oder spätes Ready nach Freigabe. **Vollständig fensterlos bestanden.**
- [`shutdown/server_stop.py`](shutdown/server_stop.py), **4.61**: Zwei Manual-
  Fenster, maximal 121 Ticks. Verifiziertes A per SIGSTOP halten, B mit F12/einem
  Tick und gehaltenem Fake-gh. Serverstop sperrt Annahme; B endet trotz A,
  Provider-Ergebnis vor Freigabe von A beobachtbar, SIGCONT A und Gesamt-Exit 0.
  Eigene 20s-Frist. **Externe Kontrollen bestanden, grafisch offen.**
- [`shutdown/deadline.py`](shutdown/deadline.py), **4.62**: Zwei Manual-Fenster,
  maximal 241 Ticks. Eigene Spiele/Fake-gh halten, gemeinsame 12-Sekunden-Frist,
  A etwa bei Halbzeit freigeben und Stop wiederholen. Monotone Aufruf-/Exit-
  Intervalle trennen ursprüngliche von frühesten Neustart-/seriellen Fristen;
  keine feste Zwei-Sekunden-Annahme für A. Überlappende Messhypothesen werden
  nach verifiziertem Produkt-Cleanup Unvollständig/Skip, nicht grün.
  **Geänderte Fassung grafisch offen.**
- [`shutdown/signals.py`](shutdown/signals.py), **4.63**: Zwei serielle Manual-
  Starts für erstes SIGINT/SIGTERM, maximal je 120 Ticks bei 2/s. Danach externer
  Shutdown-Hold-Peer für zweites SIGINT; SIGKILL nur auf eigenem leeren Server
  ohne Kinder. **Externe Signalphasen bestanden, grafische Erstsignale offen.**

Für diesen Block wären acht echte Appstarts, maximal zwei sichtbare Fenster und
höchstens 842 angeforderte Ticks nötig; lange Warps werden früh gestoppt. Eine
frische Grafikfreigabe fehlt. SIGSTOP/SIGCONT treffen ausschließlich per PID-
Datei und lebender Server-Abstammung verifizierte eigene App-Prozesse. Die
beabsichtigte Eskalation des Produkts wird getrennt von Harness-Eskalation
geprüft: letztere bleibt immer Fehler. Kein echtes gh oder Netzwerk.

Die zusätzlichen öffentlichen Rust-Verträge, je ein Test pro Cargo-Target:
- [`rust/test_pending_ownership.rs`](rust/test_pending_ownership.rs), **4.64**:
  Zwei versetzte Manual-Sessions, A zwei Ticks/B null. Gleich nummerierte Pending
  verschiedener konkreter Outputtypen fremd verwenden, am Eigentümer einmal
  empfangen und erneuten Empfang ablehnen. Keine CLI kann Rust-Ownership abbilden.
- [`rust/test_pending_drop.rs`](rust/test_pending_drop.rs), **4.65**:
  Ein Manual-Fenster, zwölf natürliche Ticks bei 2/s mit Recording. Wirklich
  offenes Pending nach erstem App-Zählerfortschritt droppen; keine weitere
  Empfangsarbeit oder Stop bis zum natürlichen Ende. Offene Arbeit bleibt
  shutdown-relevant; History und Recording schließen vollständig ab.
- [`rust/test_session_shutdown.rs`](rust/test_session_shutdown.rs), **4.66**:
  Zwei serielle Manual-Fenster. Erstes ohne Ticks erfolgreich schließen,
  Ended-Operationen/lesbare Capabilities und Pending-Metadaten/kein zusätzliches
  Endevent prüfen. Zweites mit Recording/offenem 120er-Warp bei 2/s droppen:
  hartes Produkt-Cleanup, verifizierte PID/Gruppe weg, keine Footer-Garantie.
- [`rust/test_history.rs`](rust/test_history.rs), **4.67**:
  Ein Manual-Fenster, 120er-Warp bei 2/s früh stoppen. 55 verschiedene gültige
  Hierarchie-Queries verdrängen ihn aus dem 50er-Fenster; er bleibt empfangbar
  und kehrt nach Completion nicht zurück. Annahmereihenfolge und Report-Fenster.
- [`rust/test_event_overflow.rs`](rust/test_event_overflow.rs), **4.68**:
  Ein Manual-Fenster, F9/ein angeforderter Tick erzeugt höchstens 300 tracing-
  Fehler. Ohne Eventabholung und ohne Server bis zum absichtlichen Produktabbruch
  warten. Danach 256 geordnete Failures, genau ein finales Overflow-Ended und
  positiver Verlustzähler; eigene Gruppe bereinigt, keine Reportveröffentlichung.
- [`rust/test_handle_encoding.rs`](rust/test_handle_encoding.rs), **4.69**:
  **Vollständig fensterlos bestanden, auch ohne CLI-Feature.** 36 u32-Grenzwert-
  Kombinationen in Handle-/Command-JSON und öffentlicher Bevy-Konversion;
  kanonische Integerfelder ohne Bitverlust, keine Session, App oder World.

4.64–4.68 sind kompiliert/Grafikguard-geprüft, **nicht grafisch ausgeführt**.
Reduzierte externe Wire-Kontrollen bestätigen Rust-Ownership, Drop/Recording,
History, Shutdown und Überlauf; sie ersetzen die echten Appstarts nicht. Für
alle fünf Fälle wären sieben Starts, maximal zwei gleichzeitige Fenster und
höchstens 255 angeforderte Ticks nötig. Frische Freigabe fehlt. Nur 4.68 hat
virtuellen Input; Produkt-Drop/Overflow beenden eigene Gruppen hart. Kein
Harness-Signal, keine OS-Eingabe, keine echten Issues oder Netzwerkzugriffe.

Die nach dem Risikoabgleich ausdrücklich freigegebenen internen Modultests
**4.71–4.78 sind geschrieben und gemeinsam fensterlos bestanden**. Je ein Test
am zuständigen Owner, keine nachgebildete Produktlogik:
- [4.71: native Eingaben/Fokusverlust](../src/session/input/tests/test_native_input.rs):
  echte Bevy-Nachrichten, Gate und virtuelle Tastatur; getrennte native Kontrolle.
- [4.72: Textzielbindung](../src/session/input/text/tests/test_target_binding.rs):
  Fokuswechsel, Despawn/Entity-Ersatz und Verlust der Editierbarkeit vor Flush.
- [4.73: Readback-Lebensdauer](../src/session/screenshot/capture/tests/test_readback_lifecycle.rs):
  Surface-Ergebnis/Frist/alte Bilder gezielt liefern; echte CPU-PNG-Kontrolle,
  keine versteckten Updates und kein GPU-Nachweis.
- [4.74: Screenshot-Pfadaustausch](../src/session/screenshot/destination/tests/test_path_replacement.rs):
  tatsächliche prepare/write-Grenze, eigene Outside-Sentinels und erlaubte In-Root-Links.
- [4.75: Recording-/Replay-Pfadaustausch](../src/session/recording/file/tests/test_path_replacement.rs):
  beide tatsächlichen Dateiöffnungen kontrollieren, gepinnte Eltern und finale
  Symlink-/Datei-/FIFO-Wechsel, kein Überschreiben oder blockierendes FIFO-Open.
- [4.76: Session-Identität](../src/server/tests/test_session_identity.rs):
  vorhandene Kandidaten-Grenze, Root-/Link-/ID-Kollision und ambiges Präfix;
  abschließender fehlender Manifestpfad statt Spielstart.
- [4.77: Request-ID-Erschöpfung](../src/session/coordinator/tests/test_request_id_exhaustion.rs):
  echter Koordinator, letzte normale ID/offene Arbeit, interner Replay-Command
  und reservierter Shutdown, zwei eigene externe Wire-Partner.
- [4.78: Übergangsrennen](../src/session/tests/test_transition_races.rs):
  echte Datei-Worker an Header/Sync/Loader-Read halten; Aufnahmebarrieren,
  konkurrierende Stops, Fehlerpriorität und kein verspäteter Plan-Präfix.
  Sechs eigene Wire-Partner; Koordinator-Acks und Pipe-Fences statt Zufallssleeps.

Keine test_app, Winit, GPU-Geräte oder OS-Eingaben. Nur instanz-/aufruflokale
`cfg(test)`-Kontrollen; keine neue öffentliche SDK-API oder fachliche
Produktkorrektur. Die externen Partner liefern literale Antworten, keine
Spielsimulation. Diese Fälle ersetzen keine offenen grafischen Nachweise.

Python-Fälle laufen einzeln über `unittest`, öffentliche Rust-Fälle über benannte
Cargo-Targets, interne über `cargo test --lib`. `acceptance.py` entdeckt jetzt
alle 94 Fälle und trennt lokalen/GitHub-Umfang sowie die Kontrolle 3.18.

Der zusätzliche [4.82-Report-Owner-Test](../src/server/tests/test_report_completion.rs)
hält den tatsächlichen Worker instanzlokal an und prüft denselben privaten
Completion-/Join-Pfad wie der Server: terminaler Lifecycle ohne done, danach
exakt drei vollständig korrelierte Outcomes und erst dann Serverabschluss.
Normale Submission, Unterbrechung und Providerpanic; keine App oder externe API.
Die öffentlichen Reporttests prüfen außerdem die endgültige Dateimenge beziehungsweise
gh-Aufrufzahl nach Serverexit. Das ist ein komponierter Nachweis, kein erfundener
öffentlicher Abschlusscursor.

Aktueller Umsetzungs-/Laufstand und Grenzen der Nachschärfung stehen ausschließlich
im [Arbeitsplan](../docs/testing-plan.md#qualitätsnachschärfung-nach-dem-drei-reviewer-stand).
Historische grafische Belege bestätigen keine später ergänzten Assertions.

## Struktur

Die Feature-Ordner der [geplanten Struktur](../docs/testing-plan.md#zielstruktur-und-erlaubte-hilfen)
sind angelegt. `tests/__init__.py` bindet sie als `tests.*` ein, damit etwa
`tests.inspect` nicht mit Pythons Standardmodul `inspect` kollidiert. `session/`,
`inspect/`, `startup/`, `tick/`, `keyboard/`, `pointer/`, `text/`, `screenshot/`, `recording/`, `replay/`, `cli/`, `repl/`, `reports/` und `shutdown/`
enthalten bereits Tests;
die übrigen Python-Ordner
enthalten vorerst ausschließlich `__init__.py` für rekursive Discovery.
`rust/` enthält zehn benannte Rust-Integrationstargets und Konfigurations-/Prozess-
Hilfen unter `rust/support/`. `faults/` enthält den externen Spielprotokoll-Peer,
[`report_peer.py`](faults/report_peer.py) für stderr-/Vor-Ready-Fehler, den
Prozess-Dateigrößen-Wrapper, [`script_server.py`](faults/script_server.py) für das
äußere WebSocket-Protokoll und [`gh.py`](faults/gh.py) für ausschließlich lokale
REST-Antwort-/Fehlerfixtures. Das gh-Werkzeug ruft niemals echtes gh auf. Diese Werkzeuge zählen nicht als zusätzliche Tests und
sind keine Spiele; sie importieren keine Produktmodule.
[`faults/owned_tree.py`](faults/owned_tree.py) erzeugt nur für die isolierte
Harness-Cleanup-Kontrolle drei eigene Python-Prozesse in separaten Gruppen.
SIGINT bleibt absichtlich unbeantwortet; alle Prozesse haben eine feste
20s-Selbstablauffrist. Das ist kein Spiel-/Renderer- oder Produkt-Abnahmetest.
[`faults/internal_peer.py`](faults/internal_peer.py) ist der externe stdin/stdout-
Partner für interne Koordinatortests: literale Antworten, begrenzte gehaltene
Requests und diagnostische Pipe-Fences, niemals eine World oder Rendering.
Die neun Owner-Modultests liegen unter `src/`, ausschließlich per `cfg(test)`
verdrahtet; `src/test_support.rs` enthält eigene Artefakte, semantisches Laden
von `app.toml` über Python 3.11+, begrenzte reine Beobachtung und Worker-Cleanup.
Starts, Sends und Eingabepässe bleiben im jeweiligen Test sichtbar. Bei einem
Fehler vor Ready sorgt ein Guard für Cancel/Join; diagnostische Pipe-Sender
werden vor dem Sessionabschluss freigegeben.
`acceptance.py` führt dieselben Einzeltests aus, keinen zweiten Spielablauf.
`_builds.py` enthält nur standardmäßige Cargo-Aufrufe und Compiler-/Testlisten-
Auswertung. Python-Discovery und deklarierte Cargo-Targets bestimmen den Umfang;
Metadaten wählen Teilumfänge und deren Buildabhängigkeiten; unbekannte Fälle
bleiben grafisch gesperrt. `runner_checks/` enthält getrennte Offline-Kontrollen
der Runner-Infrastruktur und wird nicht als Produkttest inventarisiert.
Leere Ordner, null Rust-Tests, fehlende Fälle oder Skips werden nicht grün.
[`providers/live_driver`](providers/live_driver/Cargo.toml) ist ein separates,
locked gebautes Testwerkzeug, kein öffentliches Woodpecker-Binary. Sein JSON-
Transport bildet jeweils genau einen expliziten SDK-Aufruf aus 4.81 ab.

## Ausführen

Python-Dateinamen haben kein `test_`-Präfix. Deshalb bei Discovery ausdrücklich
`-p '*.py'` oder einen konkreten Dateinamen angeben; das Standardmuster würde die
Tests nicht finden.

Vom Repository-Root, mit Rust-Toolchain und Python 3.11 oder neuer auf Unix.
Für die PNG-Dekodierung in 3.13 und 4.30 zusätzlich Pillow installieren, falls es fehlt:

```sh
python3 -m pip install Pillow
```

Beide Bildtests prüfen diese Abhängigkeit vor dem ersten Prozessstart; Discovery
und die übrigen Tests benötigen Pillow nicht.

### Buildcache und Speicherbedarf

Der gemeinsame Einstieg verwendet `target/` für normale Cargo-Artefakte.
4.49 verwendet separat `target/panic-abort` mit CARGO_PROFILE_DEV_PANIC=abort:
Build und späterer Sessionstart erhalten denselben festen Cache, ohne die
Umgebung normaler Builds zu ändern. Der erste Abort-Build braucht zusätzlich
Platz; weitere Läufe können dessen Artefakte wiederverwenden. Logs, Konfiguration
und Ergebnisse bleiben je Lauf unter `target/test-runs/` getrennt.

Ein gemeinsamer Cache ist kein Größenlimit: geänderte Toolchains, Features und
Quellen können zusätzliche Artefakte hinterlassen. Der Runner löscht weder
bestehende Buildcaches noch alte Laufbelege und führt kein cargo clean aus.

### Gemeinsamer Einstieg

1. Auswahl samt konkreten Buildbefehlen ohne Builds, Prozesse oder Testausführung
   prüfen: `python3 -m tests.acceptance --list`. Nur Planung, keine Abnahme.
   Die Auswahlflags lassen sich auch mit `--list` und `--build-only` kombinieren.
2. Vollständige lokale Builds separat prüfen:
   `python3 -m tests.acceptance --build-only`.
   Elf Schritte: Default/UI/Screenshot/UI+Screenshot/Server/Client/CLI,
   CLI-Binary, normale/Automation-App und echte Rust-Testbinaries. Die vollständige
   lokale Auswahl fordert die Feature-Matrix automatisch an. Kompilierte Listen
   müssen neun interne und zehn Integrationstests enthalten. Keine Spielfenster;
   Gesamtergebnis **Unvollständig**, Exit 2. Der externe Live-Transport gehört
   nicht zum lokalen Umfang; `--external-github --build-only` baut nur dessen
   Driver und die Automation-App, ohne Spielstart oder GitHub-Anfrage.
3. Fensterlosen Umfang ausführen:
   `python3 -m tests.acceptance --non-graphical`.
   Nur zwei Build-Schritte: CLI und Rust-Testbinary-Aufruf mit den neun internen
   Tests sowie ausschließlich dem Integrationstarget handle_encoding. Keine
   Automation-App und kein Live-Transport; danach dieselben 25 Fälle: neun Modultests,
   Handle-Kodierung und 15 Python-Fälle.
   Der historische Lauf hatte 23 bestandene/einen roten Fall; 4.40 ist behoben.
   Der aktuelle 25er-Teilumfang und die Feature-Matrix sind bestanden. Ein bestandener Teilumfang bleibt lokal Unvollständig/Exit 2.
4. Den vollständigen grafischen Umfang erst nach gesonderter Freigabe starten;
   Befehle und Voraussetzungen folgen unten. GitHub ist nie im lokalen Standard.

Bei Teilumfängen ergänzt `--feature-matrix` ausdrücklich die sieben Library-
Feature-Checks und die normale App, unabhängig von den ausgewählten Laufzeit-
Abhängigkeiten. Die CI fordert das weiterhin an; lokal ist es für Teilumfänge
nicht automatisch aktiv. Pilot und Negativkontrolle bauen nur CLI/Automation-App;
Rust-Binaries werden nur für ausgewählte Tests gebaut und aufgelistet.

Runner-Kontrollen separat ohne Cargo-, Spiel- oder GitHub-Starts ausführen:
`python3 -m unittest discover -s tests/runner_checks -t . -p '*.py'`. Sie prüfen Auswahl, echte
Runner-Verzweigungen mit künstlichen Compilerlisten, Fehler-/Freigabesemantik
und zweimalige Cache-/Umgebungswahl. Die Hardening-Kontrollen prüfen zusätzlich
atomare Gate-Publikation, PID-/Gruppenabfragen, finale Dateimengen und ehrliche
Budget-/Stop-Klassifikation mit isolierten Daten und Transportdoubles. Sie sind keine Produktlaufbelege und zählen
nicht zu den 75 Python-Produkttests beziehungsweise 95 Inventareinträgen.

`summary.json` enthält Commit/Dirty-Zustand, Versionen, Konfiguration, alle
Sollfälle samt nicht ausgeführten/fehlenden Einträgen, Status/Dauer/Artefakte pro
Fall und Feature. `build-*.log`, `rust-*.log` und `unittest.log` bewahren
Standardausgaben. Auswahl, lokale Gesamtabnahme und echtes GitHub-Ergebnis sind
getrennt. Ein Fehler/Skip stoppt nach Cleanup vor dem nächsten Fall; keine
automatischen Wiederholungen. Vorab gebaut werden die geplanten Abhängigkeiten;
`feature_matrix` im Bericht zeigt die zusätzlich angeforderten Feature-Checks.

Exit **0** bedeutet vollständigen lokalen Erfolg einschließlich 3.18 im selben
Lauf. Exit **1** bedeutet Test-/Build-/Cleanup-Fehler, **2** fehlende Voraussetzung
oder unvollständigen Nachweis. Teilumfang/Build-only/Planung sind keine lokale
Gesamtabnahme; `--list` beendet lediglich die Planung mit 0. Ein separat
bestandener GitHub-Test kann ebenfalls keine fehlenden lokalen Fälle ersetzen.
Die CI ruft denselben fensterlosen Einstieg auf und lädt auch Fehlerbelege hoch.
4.40 ist behoben. Der bestehende Clippy-int_plus_one in
`src/session/coordinator.rs:374` bleibt unverändert. Außerdem wertet der jetzige
CI-Schritt den Exit 2 einer vollständig bestandenen Teilauswahl noch nicht
separat aus; ein erfolgreicher Teilumfang ist keine lokale Gesamtabnahme.
Diese CI-Themen wurden mit dem gezielten UTF-8-Fix nicht mitgeändert.

**Nur nach frischer Freigabe für genau den jeweiligen Umfang und mit wachem Desktop.**
Die Spielfenster müssen zumindest teilweise sichtbar bleiben, nicht minimiert,
auf einem anderen Desktop oder vollständig von Terminal/Editor verdeckt.
`--start-unfocused` garantiert keine Sichtbarkeit.

Die Zwei-Session-Tests für Session-Isolation, Request-Zuordnung, Eingabe-/Datei-Isolation
und Replay bekommen anfängliche Positionen aus `WINDOW_POSITIONS` in `_support.py`: B liegt 160
physische Pixel rechts und 100 darunter. Bei anderer Desktop-Geometrie diese
Koordinaten zentral anpassen. Jeder Test erzeugt seine zwei TOML-Varianten aus
`app.toml` in seinen Artefakten; keine nachträglichen Fensterbewegungen, neuen
Ticks oder geänderten Fokus-/Größeneinstellungen. Andere Fenster können die Apps
weiterhin verdecken. **Replay, Request-Zuordnung, Eingabe- und Datei-Isolation
liefen erfolgreich mit beiden Positionsvarianten; Session-Isolation wurde seit
der Platzierungsergänzung nicht erneut ausgeführt.**

```sh
# Vollständiger lokaler Sollumfang: 93 Fälle plus Kontrolle 3.18, kein echtes gh.
# Vorher alle Fallbudgets/Fehleraktionen mit --list und dieser Anleitung freigeben:
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --rendered

# Historischer Pilot als Teilumfang: 16 Fälle/13 Appstarts, maximal zwei Fenster:
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --pilot --rendered

# Separater Kontrolllauf 3.18: ein Fenster, eine Resource-Query, kein Tick/Input.
# Absichtlich falsche Erwartung; unittest muss tatsächlich fehlschlagen:
WOODPECKER_TEST_RENDERED=1 python3 -m tests.acceptance --negative-control --rendered
```

Die Kontrolle verlangt zuerst die vollständige richtige Resource-Antwort und
prüft danach einen absichtlich falschen Spielernamen. Nur dieser spezifische
Assertion-Fehler ohne zusätzlichen Cleanup-Fehler bestätigt die Kontrolle. Die
falsche Erwartung gilt nur auf dieser Testinstanz; Seed und Normaltest bleiben
unverändert. Kein automatischer Folgelauf und kein zusätzlicher Produkttest.
Auch eine bestätigte Kontrolle ist keine bestandene Gesamtabnahme; Exit 2 bleibt.
**Der freigegebene Kontrolllauf wurde bestätigt:** genau ein beabsichtigter
Assertion-Fehler, kein zusätzlicher Cleanup-Fehler. Belege und verbleibender
Gesamtumfang stehen im [Arbeitsplan](../docs/testing-plan.md).

### Echter GitHub-Nachweis 4.81 — gesondert freigeben

[`providers/github_live.py`](providers/github_live.py) ist standardmäßig ausgeschlossen.
Vor einem Lauf ausdrücklich das konkrete **github.com-Testrepository**, einen
Manual-Appstart ohne Ticks/Inputs, zwei Provider-Submits und die Bereinigung
freigeben. Erwartet wird genau ein Issue; bei defekter Deduplizierung können zwei
eindeutig run-eigene Issues entstehen. Cleanup schließt nur diese nach Prüfung
von vollständigem Body, Run-UUID, Signatur und URL; es löscht keine Issues.
Der Report ist ein expliziter öffentlicher Failure-Wert, keine ausgelöste App-Panic.

1. Separat genehmigtes Testrepository verwenden. Seine Description muss
   `woodpecker-acceptance-test-repository` enthalten; Issues müssen aktiviert sein.
   Der Test verändert keine Repository-Einstellungen.
2. `gh` außerhalb des Tests privat authentifizieren; Zugangsdaten niemals in
   Chat, Argumente oder Laufartefakte schreiben. Keine echte Authentifizierung
   oder Veröffentlichung wurde bislang geprüft.
3. Erst nach der Freigabe technische Guards setzen und den Einzelumfang starten:

```sh
WOODPECKER_TEST_RENDERED=1 \
WOODPECKER_TEST_GITHUB_REPO=OWNER/TEST_REPOSITORY \
WOODPECKER_TEST_GITHUB_CLEANUP=close \
python3 -m tests.acceptance --external-github --rendered
```

`GH_REPO`/`GH_HOST` werden nur für die eigenen Kindprozesse auf das genehmigte
Repository/github.com gesetzt. Fake-gh-Umgebung ist unzulässig. Jeder Submit
steht sichtbar im Python-Test; der Rust-Transport startet oder veröffentlicht
nichts implizit. Ungewisse Veröffentlichung wird nicht wiederholt: Cleanup sucht
nur lesend den eindeutigen Run und prüft Ownership vor dem Schließen. Ein Fallback
zählt nicht als Remote-Erfolg. Test und Transport sind geschrieben/kompiliert,
**nicht mit echtem Spiel/GitHub ausgeführt**. Lokale fehlende Nachweise bleiben
auch bei künftig bestandenem externem Einzeltest unvollständig.

### Einzeltests

```sh
cargo build --locked --features cli --bin woodpecker

# Die fünf Pilot-Sessiontests ohne Anwendungsstart:
python3 -m unittest discover -s tests/session -t . -p '*.py' \
  -k server_starts -k unknown_session -k invalid_session -k invalid_launch -k launch_failure -v

# Neue Startup-/Tick-Konfigurationsprüfung, ebenfalls ohne Appstart:
python3 -m unittest -v tests.startup.invalid_config

# Weitere Fälle ohne Spielfenster; Protokollfälle benötigen Cargo für Metadaten:
python3 -m unittest -v tests.session.local_access tests.session.protocol_faults

# 4.38: zwei externe Protokollvarianten, keine Spielfenster:
python3 -m unittest -v tests.replay.blocked

# 4.39/4.43: Configfehler bzw. fünf äußere Wire-Fehler, keine Spiele:
python3 -m unittest -v tests.cli.config_errors tests.cli.script_unknown

# 4.40: unveränderte Regression, nach gezieltem UTF-8-Fix bestanden:
python3 -m unittest -v tests.cli.command_errors

# 4.51/4.52: acht Marker-/fünf Vor-Ready-Varianten, keine Spiele:
python3 -m unittest -v tests.reports.invalid_marker tests.reports.before_ready

# 4.60: belegter Prozessstart vor Ready, Cancel ohne Spiel oder Fenster:
python3 -m unittest -v tests.shutdown.starting

# 4.69: echter Rust-R0-Test, keine Session, kein CLI-Feature erforderlich:
cargo test --locked --test handle_encoding -- --nocapture

# 4.71–4.78 und 4.82: neun interne Modultests, ohne Spielfenster; Python 3.11+ erforderlich:
cargo test --locked --features cli,ui,screenshot --lib -- --nocapture --test-threads=1

# Nur die vier ohne UI/Screenshot/Server-Feature:
cargo test --locked --lib -- --nocapture --test-threads=1

# Einzelner interner Fall, zum Beispiel 4.72:
cargo test --locked --features ui --lib accepted_text_stays -- --nocapture --test-threads=1

# Nur kompilieren, keine Fenster (nicht mit bestandenem R1-Lauf verwechseln):
cargo test --locked --features cli \
  --test local_provider --test github_provider --test report_snapshot --test report_signature \
  --test pending_ownership --test pending_drop --test session_shutdown \
  --test history --test event_overflow --test handle_encoding --no-run

# Eine einzelne Datei auswählen:
python3 -m unittest discover -s tests/session -t . -p 'invalid_launch_config.py' -v
```

**Grafische Einzeltests nur nach passender Freigabe für diesen Lauf, mit wachem
Desktop und den oben beschriebenen sichtbaren Spielfenstern:**

```sh
cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app

# Session-Lebenszyklus: ein Fenster.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/session -t . -p 'session_lifecycle.py' -v

# Isolation: zwei Fenster.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/session -t . -p 'session_isolation.py' -v

# Resource-Wert: ein Fenster, keine Ticks oder Inputs.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/inspect -t . -p 'resource_value.py' -v

# Manual-Startup: ein Fenster, keine Ticks oder Inputs.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/startup -t . -p 'manual_startup.py' -v

# Exakte Ticks: ein Fenster, zehn explizite Ticks.
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/tick -t . -p 'exact_ticks.py' -v

# Je ein Fenster mit der oben beschriebenen Engine-/Spielvorbereitung:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/startup -t . -p 'engine_startup.py' -v
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/keyboard -t . -p 'movement.py' -v
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/screenshot -t . -p 'capture.py' -v

# Roundtrip: zwei Fenster, Vorbereitung je Session, fünf Aufnahme-/fünf Replay-Ticks:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/replay -t . -p 'recording_roundtrip.py' -v

# Nichtfataler tracing-Report: ein Fenster, F12, zwei Ticks, nur lokale Ausgabe:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/reports -t . -p 'tracing_report.py' -v

# Absichtlicher Panic: ein Fenster, F11, ein Tick, lokaler Report, App-Exit 101:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest discover -s tests/reports -t . -p 'panic_exit.py' -v

# 4.2: vier Appstarts seriell, Startup-Grenzen wie oben, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.startup.startup_failures

# 4.4: drei Appstarts seriell, drei Ticks je App, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.tick.application_clock

# 4.5: ein Fenster, Manual-Startup, 600 + 50.000 Ticks früh stoppen, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.tick.stop_warp

# 4.6: ein Fenster, Engine-Budget wie oben, maximal 124 weitere Ticks, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.tick.set_pace

# 4.7: ein Fenster, Manual-Startup, maximal 101 gültige Ticks, sechs Ablehnungen:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.tick.warp_errors

# 4.8: ein Fenster, Manual-Startup, keine Ticks/Inputs, voller ID-/Präfixzugriff:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.session.session_binding

# 4.9: zwei Fenster, Manual-Startup, A 60 Ticks früh stoppen, B drei Ticks, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.session.request_correlation

# 4.10: ein Fenster, Manual-Startup, zwölf Ticks bei 2 Ticks/s, Client-Ende:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.session.disconnect

# 4.11: zwei Fenster, Vorbereitung wie oben, je zwei Aktionsticks, Inputs nur in A:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.session.input_isolation

# 4.12: zwei Fenster, Vorbereitung je Session wie oben, A vier/B ein Aktionstick:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.session.file_isolation

# 4.13: ein Fenster, Manual, 100 Ticks früh stoppen, maximal 64 Queries/20 s, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.session.activity_gap

# 4.16/4.17/4.19: je ein echtes Fenster, Manual-/Reflection-Opt-in, keine Ticks/Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.inspect.resource_metadata
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.inspect.entity_query
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.inspect.reflection_values

# 4.18: ein Fenster, Manual-/Reflection-Opt-in, F10 press/tick/release/tick:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.inspect.hierarchy_handles

# 4.20: ein Fenster, Engine-/Spielvorbereitung wie oben, g und sieben Aktionsticks:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.keyboard.single_action

# 4.21: ein Fenster, Manual-Startup, ShiftLeft, vier Ticks und neun Ablehnungen:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.keyboard.invalid_transitions

# 4.22/4.23: je ein Fenster, Engine-/Spielvorbereitung, drei/neun Aktionsticks:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.pointer.ui_click
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.pointer.relative_motion

# 4.24/4.25: je ein Manual-Fenster, Buttons acht Ticks / Scroll sechs Ticks:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.pointer.buttons
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.pointer.scroll

# 4.26: ein Fenster, Manual-Startup, vier Ticks, 19 erwartete Ablehnungen:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.pointer.invalid_input

# 4.27/4.28: je ein Fenster, Engine-/Spielvorbereitung, acht/18 Aktionsticks:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.text.unicode_input
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.text.input_limits

# 4.29: ein Fenster, Manual-Startup, null Ticks, genau ein abgelehnter Capture:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.screenshot.before_first_tick

# 4.30: ein Fenster, Engine-/Spielvorbereitung, zwei Captures, eigener Datei-Sentinel:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.screenshot.overwrite

# 4.31: ein Fenster, Engine-/Spielvorbereitung, zwölf Ablehnungen, eigene Symlink-Ziele:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.screenshot.invalid_path

# 4.32: ein Manual-Fenster, F12/tracing-Fehler, maximal 61 Ticks, zehn Einträge:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.recording.file_contract

# 4.33: ein Manual-Fenster, zwei 60er-Warps bei 2/s früh stoppen, keine Inputs:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.recording.barriers

# 4.34: ein Manual-Fenster, null Ticks, vier abgelehnte 16-KiB-Texte, Dateilimit, zwei Fehler-Peers:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.recording.io_failure

# 4.35: ein Manual-Fenster, null Ticks, 33 ungültige Dateien/Pfade:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.replay.invalid_recording

# 4.36: ein Manual-Fenster, Replay-Präfix/Stop/keinen Tail, maximal 124 Ticks:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.replay.stop

# 4.37: ein Manual-Fenster, 120er-Warp früh stoppen, neun abgelehnte externe Requests:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.replay.exclusivity

# 4.41: ein Manual-Fenster, null Ticks, neun ungültige Scripts und Queue-Recovery:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.cli.script_validation

# 4.42: ein Manual-Fenster, maximal 63 Ticks, eine Ablehnung, normales Shutdown:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.cli.script_pipeline

# 4.44: ein Manual-Fenster, drei Ticks, lokale Fehler und alle REPL-Formen:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.repl.commands

# 4.45: ein Manual-Fenster, 120er-Warp bei 2/s früh stoppen, zwei eigene PTY-Clients:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.repl.terminal

# 4.46: ein Manual-Fenster, 36 Ticks bei 2/s, drei Client-Enden, Shutdown und externer EOF-Peer:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.repl.disconnect

# 4.47: fünf Manual-Starts, vier Ticks, ein erwarteter Layer-Startfehler:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.reports.tracing_config

# 4.48: vier Manual-Starts, je zwei Ticks und absichtlich abgefangene Panics:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.reports.caught_panic

# 4.49: eigener Abort-Build (bis 1800s Buildfrist), ein Fenster/F11/ein Tick:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.reports.panic_abort

# 4.50: zwei Fenster seriell, je ein Tick und absichtlicher Exit 0/17:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.reports.process_exit

# 4.53: ein Fenster, drei Ticks, kontrolliert langsames Fake-gh, kein Netzwerk:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.reports.slow_provider

# 4.54–4.57: JE eigener freigegebener R1-Lauf, ein Manual-Fenster/null Ticks:
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test local_provider -- --nocapture --test-threads=1
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test github_provider -- --nocapture --test-threads=1
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test report_snapshot -- --nocapture --test-threads=1
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test report_signature -- --nocapture --test-threads=1

# 4.58/4.59: je ein Manual-Fenster, je 120er-Warp bei 2/s früh stoppen:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.shutdown.command
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.shutdown.session_stop

# 4.61: zwei Manual-Fenster, maximal 121 Ticks, eigenes SIGSTOP/CONT und Fake-gh:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.shutdown.server_stop

# 4.62: zwei Manual-Fenster, maximal 241 Ticks, gemeinsame 12-s-Frist, Produkt-Eskalation:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.shutdown.deadline

# 4.63: zwei Manual-Starts, maximal 240 Ticks, erste Signale + externe Fehlerphasen:
WOODPECKER_TEST_RENDERED=1 python3 -m unittest -v tests.shutdown.signals

# 4.64: zwei Manual-Fenster, A zwei Ticks/B null:
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test pending_ownership -- --nocapture --test-threads=1

# 4.65: ein Manual-Fenster, zwölf natürliche Ticks bei 2/s, Pending-Drop:
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test pending_drop -- --nocapture --test-threads=1

# 4.66: zwei serielle Manual-Fenster, null/120 angeforderte Ticks, Produkt-Drop:
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test session_shutdown -- --nocapture --test-threads=1

# 4.67: ein Manual-Fenster, 120er-Warp früh stoppen, 55 lesende Queries:
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test history -- --nocapture --test-threads=1

# 4.68: ein Manual-Fenster, F9/ein Tick, Event-Burst und erwarteter Produktabbruch:
WOODPECKER_TEST_RENDERED=1 cargo test --locked --features cli --test event_overflow -- --nocapture --test-threads=1
```

Die Umgebungsvariable wird nur für den jeweiligen Aufruf gesetzt. Ohne sie schlagen
die grafischen Tests vor dem ersten Prozessstart fehl; sie werden nicht als bestanden übersprungen.
Keine automatische Wiederholung eines fehlgeschlagenen grafischen Laufs.
Ein Kalibrierungs-Skip in 4.6 ist auch bei unittest-Exit 0 ausdrücklich kein
Funktionsnachweis; übersprungene Tests lassen die Gesamtabnahme unvollständig.

## Die Dateien

- Feature-Ordner mit Tests: Je eine `.py`-Datei pro Test mit sichtbaren
  Aktionen und Erwartungen, einschließlich begrenzter Bereitschaftsabfragen.
- `__init__.py` und `*/__init__.py`: Binden die Suite und ihre Feature-Ordner als
  Python-Pakete ein. `-t .` setzt den Repository-Root als gemeinsame Importwurzel;
  gemeinsame Hilfen werden über `tests._support` importiert. `pointer/__init__.py`
  enthält reine öffentliche Component-/Pointer-/Widget-Abfragen, keine Aktionen.
  `layout_target(..., name=...)` verwendet vorhandene Namen, `label=...` bestehende
  Button-Kindtexte und öffentliche Hierarchie; genau ein Ziel ist erforderlich.
  UI-Mittelpunkte werden mit inverse_scale_factor in logische Pixel umgerechnet.
  Keine Sessionstarts, Eingaben, Bereitschaftsticks oder privaten Pointer-UUIDs.
  `text/__init__.py` verwendet dieselbe reine Layoutabfrage, `screenshot/__init__.py`
  eine reine Dateistruktur-/Hash-Abfrage ohne Verfolgen von Symlinks. Kein Helper
  fokussiert ein Widget, schreibt Texte oder startet eine Aufnahme.
  `recording/__init__.py` kodiert/dekodiert nur JSONL-Artefakte ohne Commands;
  `faults/__init__.py` erstellt nur externe Sentinel-/Adapterdateien und prüft
  Cleanup bereits gestarteter Peers. Starts und Fault-Auswahl stehen im Test.
  `faults/script_server.py` ist ein eigener externer Loopback-Prozess für das
  äußere Wire-Protokoll. Er beobachtet unerwartete RPCs/Reconnects bis zum Ende
  des echten CLI-Prozesses; nur der EOF-Fall schließt absichtlich die Verbindung.
  Stop-Datei/Fixture-Cleanup gehören ausschließlich zum Testprozessmanagement.
  `repl/__init__.py` beobachtet Pipe-/PTY-Transkripte, korrelierte Ausgaben und
  verwaltet ausschließlich den bereits gestarteten REPL-Client. `send(bytes)`
  schreibt genau diese Bytes, ohne automatisches Enter oder Commands. `wait_exit`
  liest PTY-Ausgabe auch beim Beenden weiter, damit deren endlicher Puffer nicht
  selbst den Client blockiert. Keine Sessionstarts, Ticks oder Wiederholungen.
- [`repl/pty_client.py`](repl/pty_client.py): Supervisor für eine ausschließlich
  testeigene controlling PTY. Startet die ausdrücklich angegebene CLI, beobachtet
  Termios vor/nach deren Lauf und beendet sich mit deren Exitcode. Er hält den
  Session-Leader bis zur Beobachtung am Leben: macOS widerruft sonst die Slave-FDs
  beim Leader-Ende und könnte fehlende Restaurierung durch einen Kernelreset verdecken.
  Keine `tcsetattr`-Reparatur; der Vergleich nimmt nur den Kernel-Status `PENDIN`
  aus, nicht Echo, kanonischen Modus, Signale oder Steuerzeichen. Rohwerte und
  beide Prozess-IDs bleiben in den Artefakten; kein Zugriff auf das Nutzerterminal.
- `shutdown/__init__.py`: Nur Zustands-/Appdatei-/Prozessabstammungs-Abfragen und
  Cleanup eines ausdrücklich vom Test angehaltenen eigenen Prozesses. Starts,
  Stop-Requests, Signalaktionen, Fristen und Freigaben stehen im jeweiligen Test.
  `require_observation_budget` prüft vor weiteren Schritten die noch belegbare
  Frist. Nur verifizierter Deadline-Exit 1 samt Diagnose und verschwundenen eigenen
  Gruppen darf wegen bereits verbrauchten Beobachtungsbudgets Unvollständig werden;
  weder bestehende Assertions noch Cleanup-Fehler werden abgefangen oder umgedeutet.
  Die opt-in App-Probe spiegelt nur den vorhandenen Last-Zähler atomar; sie fordert
  keine Ticks an und ist keine neue Woodpecker-Schnittstelle.
- `reports/__init__.py`: Nur begrenzte Activity-Abfragen und reine Failure-/Report-
  Auswahl und finale Report-Dateiabfragen nach Serverexit, keine Aktion oder Gap-Recovery. `providers/__init__.py` schreibt nur
  den explizit gewählten gh-Ersatz und dessen Kindprozess-Umgebung.
- `rust/support/mod.rs`: TOML-Konfiguration, eigene Artefakte, Git-Fixturedaten,
  reine Pending-/Event-/Dateizähler-/Prozessgruppen-Abfragen. Fensterpositionen
  werden ausschließlich aus `WINDOW_POSITIONS` in `_support.py` gelesen.
  Signale mit Wert null fragen nur Existenz ab, kein Helper beendet ein Spiel.
  Keine versteckten Sessionstarts, Ticks, Inputs oder Wiederholungen von Aktionen.
  Git-Schreiboperationen sind auf eigene Lauf-Fixtures begrenzt, ohne fremde Hooks
  oder Commit-Signierung. Nur der jeweilige einzelne Testprozess setzt benötigte
  Umgebungswerte vor SDK-Threads und restauriert sie nach deren Join; der aufrufende
  Prozess bleibt unverändert. Nach erfolgreichem Shutdown bestätigt auch der
  regulär geschlossene Event-Stream das Ende, ohne ein unerwartetes Endevent zu erfinden.
- `_support.py`: Konfigurationsdateien, Serverprozess, CLI-Transport, Fristen, Logs
  und Cleanup über `unittest.addCleanup`. `load_config(path=...)` liest eine frische
  Kopie der TOML-Daten, standardmäßig aus `tests/app.toml`. `write_config(path, data)`
  schreibt erst nach erfolgreichem Parse-/Werte-Roundtrip. Die dependency-freie
  Ausgabe verwendet Inline-Tabellen für Strings, Zahlen, Booleans, Listen und
  verschachtelte Tabellen; kein allgemeiner TOML-Formatierer. Absichtlich ungültige
  SDK-Werte einschließlich NaN/Inf bleiben für Ablehnungstests erhalten; andere
  Datentypen werden vor dem Schreiben abgelehnt. `run_cli` liefert Exitstatus und Ausgabe für erwartete
  Fehler; `cli` verlangt Erfolg und liefert JSON. `result` beobachtet einen
  angenommenen Command per Cursor-Polling mit Frist, optional ab ausdrücklich
  angegebenem Startcursor, und liefert dessen rohes
  `completed`/`rejected`/`failed`-Ergebnis. Activity-Lücken, falsche Zuordnung und
  fehlender Abschluss schlagen fehl. `call` sendet genau den angegebenen Command
  einmal und liefert dessen erfolgreichen Output; erwartete Fehler prüfen Tests
  weiterhin mit `run_cli` bzw. `result`. `read_resource` sendet genau eine lesende
  Resource-Query und liefert den lesbaren Wert. Keine erneute Einreichung,
  Spielaktionen, Ticks oder versteckte Session-Erzeugung. Launch-, Startup-, Handshake-
  und Panic-Fehlerfälle deklarieren `expected_exit_code=1`: Eine gespeicherte `Failed`-Session ergibt beim Serverende
  `shutdown_incomplete`. Exitstatus und Diagnose werden im Test geprüft; erzwungenes
  Cleanup bleibt auch dort ein Fehler. `start_server(env=...)` setzt ausdrücklich
  angegebene Overrides nur im Server-Kindprozess; bestehende Umgebungswerte des
  Testprozesses werden nicht verändert. `activity_bytes=...` setzt nur die
  ausdrücklich gewählte serverseitige Aufbewahrungsgröße. `command_prefix=...`
  setzt nur den ausdrücklich angegebenen externen Prozess-Launcher vor die CLI;
  4.34 nutzt dies für seinen serverlokalen Dateigrößenfehler. `shutdown_seconds=...`
  setzt die ausdrücklich gewählte Serverfrist; Default bleibt zwei Sekunden.
  `owned_process` prüft lebende App-Abstammung und separate Gruppe; `wait_process_gone`
  verlangt PID- und Gruppenabwesenheit ohne Cleanup-Signale. Registrierte Spiele
  werden auch nach dem Server-Cleanup auf Restprozesse geprüft. Bei Timeout
  verwendet `force_owned_tree` eine zuvor erfasste und vor jedem Kill neu geprüfte
  lebende Hierarchie; Blattgruppen zuerst, Server zuletzt, danach PID-/Gruppen-
  abwesenheit. Build-/Live-Transport-Cleanup nutzen dieselbe Funktion. Fremde,
  wiederverwendete oder umgehängte Identitäten werden nie signalisiert;
  erzwungenes Harness-Cleanup bleibt immer Fehler.
  `read_tick_probe(root)` liest ausschließlich den schon vom Opt-in-Spiel atomar
  geschriebenen Last-Zähler und klammert den Lesezeitpunkt mit monotonen Zeiten.
  Fehlende Daten, Warte-/Messbudgets und sämtliche Aktionen bleiben im Test;
  der Helper verbindet keinen Client und wartet oder tickt nicht selbst.
  Keine impliziten Limits, Fixture-Starts oder globalen Änderungen normaler Aufrufe.
- `app.toml`: Gemeinsame echte Launchkonfiguration; Engine-Varianten liegen nur
  in den jeweiligen Laufartefakten. `startup.kind = "manual"` verhindert
  automatische Engine-Startup-Ticks. Pfade gelten vom Repository-Root; der
  Settings-Seed wird vom App-Projektverzeichnis aus aufgelöst. Der SDK-Start nutzt
  `cargo run`; deshalb die Anwendung vorher mit denselben Features bauen.
- `settings.json`: Fester Ausgangsstand mit dem Spielernamen `Woodpecker-Abnahme`,
  der in den isolierten Session-Workspace übernommen wird. Der abweichende Name
  unterscheidet den Testwert vom App-Default. Persönliche Spieleinstellungen
  werden nicht als Seed verwendet.

Jeder Server bindet einen freien Loopback-Port. CLI-Aufrufe, Serverausgabe und
Cleanup-Ergebnis bleiben im ausgegebenen Ordner unter `target/test-runs/` erhalten.
Bei Fehlern wird der vorhandene Server einmal per Signal beendet; keine Wiederholung
von CLI-Aktionen. Erzwungenes oder fehlerhaftes Cleanup erscheint zusätzlich zum
ursprünglichen Fehler. Diese Tests implementieren keinen Headless-Spielmodus.

### Konfigurationsvarianten

Nach dem sichtbaren Serverstart stehen die gewünschten Änderungen im Test:

```python
configuration = load_config()
configuration["startup"] = {"kind": "engine", "timeout_seconds": 60, "max_ticks": 3600}
config = server.artifacts / "engine.toml"
write_config(config, configuration)
# Erst der Test selbst ruft danach session create auf.
```

Jeder Aufruf liest neue Daten; Varianten beeinflussen weder die Basisdatei noch
andere Varianten. Kommentare/Formatierung der Basis sind unerheblich und werden
nicht in die generierte Datei übernommen. Starts, Inputs und Ticks führt die
Konfigurationshilfe nicht aus.

## Umfang

**Historische B-Test-Härtung vor dem full-stop-Grafiklauf:** Alle 17 vereinbarten Maßnahmen sind umgesetzt oder,
bei der unverändert festen Screenshot-Szene, ausdrücklich als Fixture-Vertrag
entschieden. CLI frisch gebaut; 15 fensterlose Regressionen, eine zusätzliche
reale CLI-/F0-Kontrolle des Recording-Dateilimits und 15 Gruppen isolierter
Assertion-/Abfragekontrollen bestanden. Alle 15 geänderten Grafikfälle sperren
vor Prozessstart. 35 Server endeten mit erwarteten Exits ohne Harness-Eskalation.
Belege: `target/test-runs/b-tier-quality-6uijdoy3/`. Kein App-/Grafiklauf; die
strengeren grafischen Fassungen haben ausdrücklich keinen neuen Erfolgshaken.
Die folgenden älteren Belege gelten nur für ihre damaligen Fassungen.

**Historische Belege:** Die folgenden früheren Laufnotizen (auch rote 4.40-
Ergebnisse) bleiben historisch. Während der späteren UTF-8-Korrektur war target/
plötzlich verschwunden, Ursache unklar; alte Artefaktpfade sind nicht mehr
verfügbar. Neu belegt sind der unveränderte 4.40-Test und 15 fensterlose Python-
Regressionen mit frisch gebauter CLI. Aktueller Fix-Beleg:
`target/test-runs/cli-utf8-recheck-2mmCAR/`. Kein neuer grafischer Gesamtlauf.

Die Konfigurationsumstellung wurde ohne grafische Läufe geprüft: 55 Varianten aus
22 Tests sind nach TOML-Parsing identisch zum vorherigen Stand. Eine umformatierte
Basis, verschachtelte Werte, Escaping, NaN/Inf und Fehler vor dem Schreiben sind
geprüft; alle 28 Grafikfreigabesperren funktionieren vor Prozessstart. Acht
fensterlose Regressionen bestanden, 17 Server und zehn Fehlerprozesse sauber
beendet. Belege: `target/test-runs/config-maintenance-3zhqM8/`.
Die folgenden grafischen Erfolgsbelege stammen aus Läufen vor dieser Umstellung;
sie sind kein neuer grafischer Gesamtlauf des refaktorierten Stands.

Die sechzehn regulären Pilotfälle haben erfolgreiche Einzelbelege aus mehreren
Läufen; zusätzlich ist der absichtlich fehlschlagende Kontrolllauf bestätigt.
Ein neuer gemeinsamer 16-Test-Lauf wurde nicht ausgeführt. Die neuen Fälle
4.2–4.7 haben ebenfalls erfolgreiche Einzelbelege aus mehreren Läufen.
4.2 und 4.4 bestanden gemeinsam in 35,404 Sekunden ohne Skip oder Retry:
vier Startup-Fehlerfälle mit erwartetem Server-Exit 1 und drei Clock-Varianten
mit Exit 0, kein erzwungenes Cleanup oder verbliebene Produktprozesse.
Belege: `target/test-runs/startup-clock-live-Bde4hO/`.
Der freigegebene Lauf 4.5–4.7 bestand in 23,740 Sekunden ohne Skip oder Retry;
alle drei Server endeten mit Exit 0 ohne erzwungenes Cleanup, keine Produktprozesse
blieben übrig. Logs und Zusammenfassung: `target/test-runs/tick-controls-live-egpEBo/`.
Der freigegebene Gesamtblock **4.8–4.15 bestand in 40,557 Sekunden** ohne Skip
oder Wiederholung. Neun Spielstarts, 17 Server mit erwarteten Exits, zehn externe
Protokoll-Fehlerprozesse und ein Snapshot-Beobachter wurden ohne erzwungenes
Cleanup beendet; persönliche Settings und alle Testeingaben blieben unverändert,
keine Produktprozesse übrig. Belege: `target/test-runs/session-range-live-btNrhG/`.
Frühere fensterlose Regressionen und die Sensitivitätskontrolle für 4.14 bleiben
unter `target/test-runs/session-access-checks-5hBD3L/` und
`target/test-runs/session-boundaries-checks-M8AYwy/` erhalten.
Der freigegebene Lauf **4.16–4.20 bestand in 19,211 Sekunden**, ohne Skip oder
Wiederholung. Fünf serielle Spielstarts, alle fünf Server mit Exit 0 ohne
erzwungenes Cleanup, keine Produktprozesse übrig. Testeingaben und persönliche
Settings blieben unverändert. Belege: `target/test-runs/inspect-keyboard-live-9O1aUj/`.
Danach wurde 4.16 um unabhängige vollständige Fixture-Sollmengen verschärft.
Diese neuen Mengen-Assertions bestehen mit gespeicherten echten CLI-Antworten
und erkennen 15 gezielte Auslassungen sowie eine unerwartete Fixture-Resource;
zusätzliche Nicht-Fixture-Resources bleiben erlaubt. Kein neuer grafischer Lauf;
aktuelle Belege: `target/test-runs/resource-metadata-quality-fbaWTV/`.
Vorbereitungsprüfungen stehen unter `target/test-runs/inspect-checks-TXEV8o/`
und `target/test-runs/keyboard-checks-dDN54c/`: Builds/Format, Syntax, Discovery,
Konfigurationsvarianten, Freigabesperren und fensterlose Regressionen.
**4.21 bleibt geschrieben, aber grafisch nicht ausgeführt**; er war nicht Teil
der freigegebenen Auswahl 4.16–4.20.
4.22–4.26 sind geschrieben, aber grafisch nicht ausgeführt. CLI-/Automation-Builds,
41-Test-Discovery, fünf Freigabesperren, zwei Engine-Konfigurationen und acht
fensterlose Regressionen sind geprüft; Belege: `target/test-runs/pointer-checks-MHPQ4b/`.
4.27–4.31 sind geschrieben, aber grafisch nicht ausgeführt. CLI-/Automation-Builds,
46-Test-Discovery, fünf Freigabesperren, vier Engine-Konfigurationen, reine UTF-8-/
Snapshot-/PNG-Lesekontrollen und acht fensterlose Regressionen sind geprüft.
Belege: `target/test-runs/text-screenshot-checks-iKJKZz/`; daraus folgt kein neuer
Fokus-, Bytegrenz-, Capture- oder Pfadsicherheitsnachweis am Spiel.
4.32–4.38 sind geschrieben. **4.38 bestand mit beiden externen Varianten** im
Neun-Test-Regressionslauf: 19 Server und zwölf Peers ohne erzwungenes Cleanup beendet.
Die sechs grafischen Fälle 4.32–4.37 wurden nicht ausgeführt. Für 4.34 sind externe
Start-/Active-I/O-/Recovery- und beide Endphasen geprüft; für 4.35 alle 33 Loader-
Varianten und zwei leere positive Kontrollen, ohne einen Präfix an den Peer zu senden.
Diese F0-Kontrollen ersetzen weder den realen Spiel-World- noch den Renderer-Nachweis.
53-Test-Discovery, sechs Freigabesperren, Diagnose-TOML und Builds sind geprüft;
Belege: `target/test-runs/recording-replay-checks-OXsI1F/`.
4.39–4.43 sind geschrieben. **4.39/4.43 bestanden fensterlos**, erneut im
Elf-Test-Regressionslauf: 20 echte Server, zwölf Spielprotokoll-Peers und fünf
äußere Wire-Peers ohne erzwungenes Cleanup beendet. **4.40 schlägt normal fehl**:
rohe ungültige UTF-8-argv werden nicht als `invalid_json` ausgegeben; die anderen
15 Varianten bestehen. Der abschließende neue Teilumfang ist daher fehlgeschlagen,
nicht grün. Für 4.41 sind die neun exakten Parserfehler/Indizes geprüft, für 4.42
beide Script-Schemata akzeptiert, jeweils gegen fehlende Session vor Spielstart.
Die vollständigen grafischen Fälle 4.41/4.42 sind nicht ausgeführt.
58-Test-Discovery, zwei Freigabesperren und CLI-/Automation-Builds sind geprüft;
Belege: `target/test-runs/cli-script-checks-EcVvyW/`. Die Gesamtabnahme ist wegen
des gefundenen Fehlers nicht bestanden; zusätzliche Nachweise bleiben offen.
4.44–4.46 sind geschrieben, alle drei grafischen Fälle bleiben offen. Fensterlose
Kontrollen mit externen Protokoll-Peers bestanden: lokale Parse-/UTF-8-Fehler ohne
Einreichung, Recording/leerem Replay, echte PTY mit Unicode-Editierung trotz
asynchroner Antwort, Restaurierung nach quit/Fehler und erhaltenes Pending nach
quit/EOF/Ctrl+C. Die exakte externe technische-Endphase aus 4.46 bestand mit
geplanten REPL-/Server-Exits 1. Dabei wurden keine Spielticks simuliert.
Elf unabhängige Regressionen bestanden erneut; 20 Server, zwölf Spielprotokoll-
Peers und fünf äußere Wire-Peers sauber beendet. Die bekannte UTF-8-Abweichung
4.40 wurde separat erneut bestätigt und bleibt rot. CLI-/Automation-Builds,
61-Test-Discovery, drei Freigabesperren und reine Transkript-/Termios-Kontrollen
sind geprüft; Belege: `target/test-runs/repl-checks-WLbkHV/`.
4.47–4.57 sind geschrieben: sieben Python-Fälle und vier verdrahtete Rust-
Integrationstests. **4.51/4.52 bestanden vollständig fensterlos**; zusammen mit den
elf bisherigen Regressionen bestanden 13 Tests, 33 Server/25 Protokoll-Peers/fünf
äußere Wire-Peers ohne erzwungenes Cleanup beendet. Öffentliche Rust-Report-API
mit externer Session sowie gehaltenes Fake-gh sind zusätzlich geprüft; kein
Spiel-/Renderer-Nachweis daraus. 68-Python-Discovery, vier echte Rust-Targets,
fünf Python-/vier Rust-Freigabesperren, normale/Automation-/CLI-Builds, Format und
kopierte Snapshot-App sind geprüft. **Neun grafische/R1-Fälle sowie der echte
Abort-Build bleiben offen.** Der bekannte Fehler 4.40 bleibt separat bestätigt rot.
Belege: `target/test-runs/report-provider-checks-HW8W6m/`.
4.58–4.63 sind geschrieben; **4.60 bestand fensterlos**. Der 14-Test-Regressionslauf
bestand mit 34 Servern/26 Protokoll-Peers/fünf äußeren Wire-Peers ohne Harness-
Eskalation. Zusätzliche externe Kontrollen belegen direkte Vorbedingungen,
Replay-/Recording-Verwaltungsstopp, unabhängigen Abschluss, eine gemeinsame
6s-Frist (gemessen 6,080s) trotz wiederholtem Stop, gehaltenen Provider und beide
externen Signalphasen. Daraus folgt kein Spiel-/Counter-Nachweis. Alle fünf
neuen grafischen Fälle bleiben offen; CLI-/App-Builds, 74-Python-Discovery,
fünf Freigabesperren und Dokumentation sind geprüft.
Die Kontrolle zeigte keine internen Replay-Warp-Pendings in öffentlicher
Server-Activity. **Bei 4.79 wurden 4.36/4.37 korrigiert:** atomarer unabhängiger
App-Zähler statt unerreichbarer Pending-Einträge, danach Recording-/Counter-
Orakel. Kein zusätzlicher Tick oder Recovery; geänderte Fassungen grafisch offen. Auch 4.40 bleibt separat bestätigt rot.
Belege: `target/test-runs/shutdown-checks-pGSJoN/`.
4.64–4.69 sind als sechs weitere öffentliche Rust-Integrationstargets geschrieben.
**4.69 bestand vollständig ohne Fenster, mit und ohne CLI-Feature.** Alle zehn
Rust-Targets kompiliert; neun Grafikguards ohne Appstarts oder Testartefakte
geprüft. 14 fensterlose Regressionen bestanden, 34 Server/26 Protokoll-Peers/fünf
Wire-Peers ohne erzwungenes Harness-Cleanup beendet. Reduzierte direkte Rust-
Kontrollen mit sechs externen Peers bestätigen Ownership, Drop/Recording,
Shutdown, Retention und Überlauf: 256 Failures, genau ein Ended, 44 gezählte
Verluste. Keine Spiel-/World-Behauptung daraus. Neue 300er-App-Diagnose und
normale/Automation-/CLI-Builds geprüft; alle fünf R1/R2-Läufe bleiben offen.
Belege: `target/test-runs/rust-contract-checks-aoxY5f/`.
Nach ausdrücklicher Bestätigung sind **4.70 abgeglichen und 4.71–4.78 als acht
interne Modultests bestanden**. Gemeinsamer Lauf mit cli/ui/screenshot: acht
Tests ohne Skip, Default-Lauf vier; 4.69 und 14 fensterlose Python-Regressionen
bestanden. 34 Server/26 Protokoll-Peers/fünf äußere Wire-Peers sauber beendet;
final zusätzlich acht interne Wire-Partner. Neun Grafikguards, zehn öffentliche
Rust-Targets und Default-/CLI-/App-Builds geprüft. Keine fachlichen Produktfixes.
Ein früher 4.78-Entwicklungslauf hing am testseitig gehaltenen Pipe-Sender und
lief in den Harness-Timeout. Fehlbeleg erhalten; Sender-Lebensdauer und
Cancel-/Join-Schutz korrigiert. Final kein Harness-Zwangsende oder Prozessrest.
In diesem historischen Block keine neuen Spielfenster; damalige
4.40-/4.36-/4.37-Befunde blieben unverändert. Spätere Testkorrektur siehe 4.79.
Belege: `target/test-runs/internal-contract-checks-XZyRHn/`.
4.79 integriert alle lokalen Tests und die standardmäßige Buildmatrix.
Der echte gemeinsame fensterlose Lauf bestand für 23/24 Fälle; 4.40 scheitert
unverändert, korrekt Exit 1. Alle zwölf Builds sowie Workspace-Alltargets
bestanden; gezielte Preflight-/Build-/Cleanup-/Ergebniskontrollen geprüft.
4.81 ist geschrieben, Transport kompiliert und beide Guards geprüft; kein echtes
gh/Issue und keine Spielfenster. 4.80 bleibt offen. CI-Clippy ist zusätzlich am
bestehenden int_plus_one rot. Belege: `target/test-runs/acceptance-integration-checks-gVvC9e/`
und `target/test-runs/acceptance-74jubrj4/`.
Noch **nicht** nachgewiesen sind die übrigen Features, Grenzfälle, öffentlichen Zugänge,
Rust-Verträge und die vollständige lokale/externe Gesamtabnahme. Der
[abgestimmte Plan](../docs/testing-plan.md) führt diesen verbleibenden Umfang in
Schritt 4 auf.
