# Überarbeitung der Woodpecker-Testsuite

Dieser abgestimmte Plan ersetzt die bisherige Testplanung. Er beschreibt das Ziel
und die verbleibende Arbeit, nicht eine bereits implementierte oder bestandene Suite.
Schritte 1 und 2 sind abgeschlossen. Auf ausdrücklichen Nutzerauftrag wurden vor
Schritt 3 alle bisherigen Python-/Rust-Tests, Testprogramme und Smoke-Tests entfernt,
auch die Tests des Modell-Zoos und historische Diagnoseprogramme unter `tests/`.
Der Neuaufbau beginnt ohne bestehende Tests. Aktuell gibt es weder eine ausführbare
Testsuite noch einen automatisierten Verhaltensnachweis; Build-/Lint-Erfolg ist keine
Gesamtabnahme. Die alte Suite bleibt ausschließlich über die Git-Historie nachvollziehbar.

## Zweck

Die Signalstation in `test_app/` ist Referenzanwendung und echte Testumgebung für
Woodpecker. Die Feature-Tests sollen zugleich nachweisen, dass Woodpecker im
Nutzereinsatz funktioniert, und als verständliche Vorlagen für eigene Bevy-Spiele
dienen. Spielaufgaben sind ausschließlich Spielinhalt: Sie bieten beispielsweise
einem AI-Agenten eine komplexe Sequenz, werden aber kein Woodpecker-Aufgabenmodell.

Einzelne Feature-Tests und die Gesamtabnahme verwenden dieselben Nachweise.
Es entsteht kein zusätzlicher großer Spieltest mit denselben Assertions.

## Festgelegte Grundlagen

- **Python mit `unittest`, primär über die echte CLI mit Command-JSON.** Aktionen
  und Erwartungen stehen in derselben Testfunktion. Keine eigene Testsprache.
- **Zugänge nach ihrem zusätzlichen Risiko prüfen.** REPL und CLI-Scripts erhalten
  gezielte Tests ihrer Besonderheiten, keine Kopie sämtlicher Feature-Abläufe.
  Öffentliche Rust-Verträge wie Pending-Ownership werden bei Bedarf als gezielte
  neue Rust-Nachweise ergänzt, nicht pauschal aus der alten Suite übernommen.
- **Ein Verhalten hat einen verantwortlichen Test.** Andere Tests dürfen Inputs
  oder Inspect verwenden, ohne deren vollständige Semantik erneut zu prüfen.
  Weitere Kombinationen benötigen einen eigenen relevanten Fehlerfall.
- **Fortschritt bleibt sichtbar.** Keine `click()`-, `tap()`-, Reise- oder
  Reset-Helfer mit zusätzlichen Ticks und versteckten Spielprüfungen. Ein bewusst
  gestarteter Warp oder Replay darf natürlich die angeforderte Arbeit ausführen.
- **Vorbereitung ist ausgewiesen.** Startup-Policy, Bereitschaft und Tickbudget
  gehören zum Ablauf. Engine-Startup führt gezählte Ticks aus; `EngineReady` ist
  nicht automatisch die Spielbereitschaft. Eine nötige Tick-/Inspect-Schleife bis
  `LevelLoad::Ready` bleibt sichtbar, begrenzt und protokolliert.
- **Tests sind unabhängig und standardmäßig seriell.** Isolierte Sessions statt
  eines Abhängigkeitsgraphen zwischen Tests. Mehrere Sessions innerhalb eines
  Tests sind möglich, etwa für Replay oder Isolation.
- **Aktuell nur gerenderte Referenz-App.** Es gibt keine angebundene
  Headless-Variante; auch die alten Smoke-/Offscreen-Startoptionen sind entfernt.
  Kein Headless-Neubau nebenbei.
  Grafische Läufe benötigen eine ausdrückliche Freigabe für den jeweiligen Lauf.

Jeder Test zeigt: Start und Vorbereitung → Ausgangszustand → öffentliche Aktionen
→ ausdrücklich angeforderter Fortschritt → relevante Rückgabe und Wirkung →
zuverlässiges Aufräumen, auch bei einem Fehler.

## Feature-Nachweise und Zuständigkeit

Alle Einträge sind geplante Nachweise, keine Aussage über aktuelle Testergebnisse.
Die Dateinamen beschreiben die Zielstruktur.

| Zieldatei | Verantwortlicher Nachweis |
| --- | --- |
| `test_sessions.py` | Anlegen, Auflisten, Inspizieren und Request-Zuordnung; Arbeit nach Disconnect; getrennte Eingaben, Zustände und Dateien zweier Sessions; Activity-Verlust als unbekanntes Ergebnis; Launch-/Verbindungsfehler. |
| `test_startup.py` | Manual ohne Warm-up; Engine mit gezählten Startup-Ticks und anschließendem Stillstand; getrennte Spielbereitschaft; Render-Bootstrap; begrenzte Fristen/Tickbudgets und fehlende Voraussetzungen. |
| `test_ticks.py` | Exakte Ticks und Stillstand; Fixed-Clock unabhängig von Pace und Denkpausen; Application-Clock beibehalten; laufenden Warp beschleunigen/stoppen; Null-Ticks und ungültige Konfiguration ablehnen. |
| `test_keyboard.py` | Press/Hold/Release mit tatsächlicher Bewegung erst beim Tick; einmalige Aktion trotz Halten; kontrollierte Eingabe ohne nativen Fokus; doppelte Press, unbegründete Release und unbekannte Taste ablehnen. |
| `test_pointer.py` | Absolute/relative Position, Begrenzung, Buttons und Scroll mit echter UI-/Spielwirkung; FPV-Blickrichtung; kein Bewegen des OS-Cursors; ungültige Positionen und Zustandswechsel. |
| `test_text.py` | Echtes Feld fokussieren; Unicode erst beim Tick; richtige Zielbindung; leere Eingabe und UTF-8-Bytegrenze; Text unabhängig von Tastaturereignissen. |
| `test_inspect.py` | Resource-Metadaten/Werte; Entity-Filter, Component-Namen/Werte und Handles; tatsächliche Kinder und Tiefengrenzen einer Hierarchie; unbekannt/fehlend/veraltet unterscheiden; keine Simulationsfortschreibung. |
| `test_screenshot.py` | Echtes PNG mit Dimensionen und aussagekräftigen Szenenregionen; unveränderte Simulation; Wiederholung/Overwrite und Pfadsicherheit; ausdrückliche Surface-Ablehnung statt scheinbarem Bild. Keine Capture-Retries. |
| `test_recording.py` | Header, Reihenfolge, Outcomes einschließlich Ablehnungen, ausgeschlossene Controls und Footer; Barrieren bei laufender Arbeit; Ende und Schreibfehler. |
| `test_replay.py` | Aufnahme in frischer Session mit gleicher Ausgangslage/Clock wiedergeben; relevante Zwischen-/Endzustände vergleichen; Handles neu auflösen; Stop, Exklusivität und Fehler; ungültigen Footer vor jeglicher Präfixausführung ablehnen. |
| `test_reports.py` | Nichtfataler tracing-Fehler samt weiterhin nutzbarer Session; Opt-in/Filter; Panic/Exit ohne doppelten Failure; aufgelöste Commands und persistenter Report; echter `panic=abort`-Build; Provider/Fallback und Snapshots gezielt ergänzen. |
| `test_shutdown.py` | Terminale Completion und genau ein Ende; offene Arbeit auflösen; Serverstop mit mehreren Sessions und Reports; gemeinsame Shutdownfrist, erzwungene Bereinigung und keine Prozessreste. |
| `test_cli.py` | JSON-/Command-Form vor dem Senden prüfen; Script vollständig vorvalidieren, ohne gültigen Präfix auszuführen; Pipeline/Barrieren, Command-Indizes, Exitstatus und unbekannte Outcomes. |
| `test_repl.py` | Echte REPL-Eingabe und eigene Ausgabe; Parsefehler ohne Senden; Disconnect ohne Arbeitsabbruch; Request-Zuordnung, PTY-Eingabe während Activity und Terminalwiederherstellung. |

Normale Feature-Nachweise verwenden die Referenz-App. Absichtlich defekte
Protokollpartner sind nur für nicht regulär erzeugbare Fehler sinnvoll und werden
bei tatsächlichem Bedarf neu gebaut. Zusätzliche interne Tests benötigen einen
konkreten eigenen Fehlerfall, etwa GPU-Readback-Rennen, Fokus/Despawn,
Dateizugriffsrennen oder öffentliche Rust-Verträge. Die bisherigen Tests sind auf
Nutzerauftrag vollständig entfernt. Reine Missions-, Parcours-, Trefferwertungs-
oder Asset-Konventionsprüfungen gehören nicht zu den neuen Woodpecker-Pflichtnachweisen.

Für Reports fehlt derzeit eine gezielt auslösbare Signalstation-Fehleraktion.
Geplant ist eine opt-in Diagnoseaktion ausschließlich in der App, ausgelöst durch
normale Eingabe und Tick, kein neuer Woodpecker-Command. Die bisherigen
Fehlerprogramme sind entfernt; notwendige Fehlerauslöser werden gezielt neu gebaut.

## Vier Beispiele zum Erproben der Testform

1. **Tastatur:** Position lesen; `input.keyboard.press` mit `arrow_right` senden;
   Position vor dem Tick unverändert. Einen Tick ausführen und Bewegung prüfen,
   drei weitere Ticks ausführen und fortgesetzte Bewegung prüfen. Taste über
   `input.keyboard.release` loslassen; nach Verarbeitung und weiteren Ticks bleibt
   die Position gleich. Kein versteckter R-Reset.
2. **Screenshot:** Nach ausgewiesener Bereitschaft Zeit und Spielerzustand lesen;
   `screenshot.capture` senden. Antwort, PNG-Dimensionen und bekannte HUD-/Startpad-
   Regionen prüfen; Zeit und Spielerzustand bleiben unverändert. Die bisherigen
   Bildregionen sind Ausgangspunkt und müssen im neuen Lauf verifiziert werden.
3. **Recording/Replay:** In A `recording.start`, eine kurze Eingabe-/Tickfolge,
   relevante Beobachtungen und `recording.stop` senden. Datei ausdrücklich nach B
   kopieren; B mit gleicher Clock, Settings und Ausgangslage vorbereiten.
   `replay.start` muss abschließen und die relevanten Beobachtungen wiederherstellen.
   Tickdifferenzen relativ zum jeweiligen Start vergleichen, keine pauschalen
   Weltsnapshots oder Determinismusgarantie für beliebige Spiele.
4. **Fehler und Prozessende:** App-Diagnose aktivieren; Fehler durch Eingabe und
   ausdrücklich angeforderten Tick auslösen. Dessen Request-ID erhalten, terminales
   Fehlerergebnis statt Erfolg prüfen; Failure, Report und Ende zuordnen. Report
   bleibt lesbar, keine doppelte Exit-Meldung, App-Prozess weg und Server bereinigt.

Diese Beschreibungen sind noch keine ausführbaren Tests. Insbesondere die
Report-Diagnoseaktion ist nicht implementiert.

## Zielstruktur und erlaubte Hilfen

```text
tests/
  README.md          # kurzer Einstieg und Anbindung eines eigenen Spiels
  app.toml           # echte Woodpecker-Launchkonfiguration
  settings.json      # fester Seed statt persönlicher Einstellungen
  _support.py        # nur technische Lebensdauer, Transport und reine Queries
  test_<feature>.py  # konkrete Dateinamen und Zuständigkeiten in der Tabelle
  faults/            # notwendige Absturz-/Protokollprogramme
```

`unittest` übernimmt Discovery, Auswahl und Testausführung. Der Gesamteinstieg
organisiert Standardwerkzeuge und Ergebniszusammenfassung, keine neue
Runner-Architektur, Szenariokataloge, Capability-Matrix oder Schrittgraphen.
Bildregionen und Toleranzen stehen beim Bildtest; Pillow liest PNGs statt eines
selbstgeschriebenen Codecs. Diese Bildtest-Abhängigkeit wird ausdrücklich genannt.
Kein zusätzliches `app.json` als Metakatalog und keine parallele Beispielsuite.

Erlaubte Hilfen: Prozesse starten/beenden, Artefaktpfade, einmaliges Senden,
zugeordnetes Warten mit Frist und reine Inspect-Abfragen. Die geplanten Kürzel
`call`, `submit`, `result` und `read_player` sind **Testhilfen, kein vorhandenes
Python-SDK**. `call` sendet genau einmal und wartet auf ein erfolgreiches Ergebnis;
`submit`/`result` trennen Annahme und terminales Ergebnis für laufende Arbeit oder
erwartete Fehler. Warten erzeugt keine zusätzlichen Spiel-Commands oder Ticks.
Activity-Verlust bzw. unbestätigte Übermittlung führt nicht zu automatischem Retry.

## Gesamtabnahmelauf

Ein gemeinsamer Einstieg führt dieselben Feature-Tests, gezielte Zugangsprüfungen
und erforderliche zusätzliche Rust-/Fehlernachweise aus. Einzelne Tests bleiben
separat ausführbar. Ein fehlender Nachweis darf nicht durch bloße Discovery oder
einen registrierten Command als abgedeckt erscheinen.

1. Vor dem ersten Prozess Grafikfreigabe, Desktop und Abhängigkeiten prüfen.
2. Commit, Dirty-Zustand, Versionen und Konfiguration festhalten; Targets einmal
   bauen oder verwendete Artefakte ausdrücklich validieren.
3. Standardmäßig seriell mit isolierten Sessions ausführen. Nach einem Fehler nur
   unabhängige Tests weiterführen, wenn die Prozessbereinigung sicher war.
4. Pro Feature Status, verantwortliche Tests und Artefaktpfad melden; anschließend
   das Gesamturteil. Alte Ergebnisse sind kein Nachweis für diesen Lauf.

| Gesamturteil | Bedingung |
| --- | --- |
| **Bestanden** | Alle verpflichtenden Nachweise des benannten Umfangs erfolgreich; keine unbekannten Outcomes oder Prozessreste. |
| **Fehlgeschlagen** | Verhalten, Build oder Cleanup entspricht nicht der Erwartung. |
| **Unvollständig** | Voraussetzung oder Nachweis fehlt, ein Test wurde übersprungen oder der Lauf abgebrochen. Kein erfolgreicher Gesamtexit. |

Der lokale Umfang enthält alle lokalen Features und öffentlichen Zugänge der
aktuell gerenderten Integration. Echter GitHub-Provider-Erfolg braucht eine
zusätzliche ausdrücklich freigegebene Abnahme gegen ein Testrepository. Ohne sie
bleibt der externe Nachweis offen; die lokale Abnahme ist keine uneingeschränkte
„alle Features“-Abnahme. Der Standardlauf erzeugt keine echten GitHub-Issues.

Pro Test bleiben Command-Journal und Prozesslogs erhalten; PNG, Recording und
Report nur bei tatsächlicher Nutzung. Session, Request-ID, Aktion, Erwartung und
Ergebnis müssen zuordenbar sein. Cleanup-Fehler verdecken nicht den ursprünglichen
Fehler. Komplexe AI-Aufträge bleiben getrennte Erprobungen mit Ziel, Budget und
Erfolgskriterium, keine Verdopplung der Feature-Tests.

## Checkliste und Arbeitsstand

### 1. Bestand prüfen — abgeschlossen

- [x] Tests ihrem tatsächlichen Woodpecker-Verhalten zuordnen.
- [x] Doppelungen, Implementierungsbindungen und unnötige Infrastruktur identifizieren.
- [x] Vorschläge zum Behalten, Ersetzen, Zusammenlegen und Entfernen gruppieren.

Quelltextanalyse auf `84857b4`, ohne Testlauf: Der bisherige Keyboard-Filter startet
den ganzen Signalstation-Durchlauf; Helfer enthalten zusätzliche Ticks und Aktionen;
Dokumentation und Konfiguration widersprechen sich. Konkrete Doppelungen betreffen
unter anderem doppelte Press-Ablehnung und Inspect-Fälle. Wertvolle öffentliche
Wirkungsnachweise und zusätzliche interne Fehlerfälle wurden identifiziert.
Die Analyse ist historischer Ausgangspunkt; der alte Testcode ist jetzt entfernt.

### 2. Zielstruktur festlegen — abgeschlossen und abgestimmt

- [x] Zweck, Sprache, öffentliche Zugänge und gemeinsames Testmuster festlegen.
- [x] Feature-Nachweise und vier unterschiedliche Beispielabläufe beschreiben.
- [x] Hilfscode und Dateistruktur begrenzen.
- [x] Gemeinsamen Gesamtabnahmelauf mit ehrlichem Umfang und Ergebnis festlegen.

### 3. Testform praktisch erproben — offen

- [ ] Benötigte technische Hilfen und repräsentative Beispiele umsetzen.
- [ ] Fehlende App-Diagnose für den Report-Fall ausdrücklich berücksichtigen.
- [ ] Erfolgsfall und absichtlich falsche Erwartung prüfen, einschließlich Cleanup.
- [ ] Einzel- und Gesamtergebnisdarstellung erproben; noch fehlende Features als unvollständig ausweisen.
- [ ] Grafische Läufe nur nach frischer Freigabe ausführen.

### 4. Feature-Nachweise neu aufbauen — offen

- [ ] Alle Feature-Nachweise nach dem erprobten Muster neu umsetzen.
- [ ] Zugangs- und interne Tests nur für zusätzlichen Nutzen ergänzen.
- [x] Bisherige Tests und reine Testinfrastruktur vorab auf Nutzerauftrag entfernen.
- [ ] Vollständigen lokalen Gesamtabnahmelauf auf dem finalen Stand ausführen.
- [ ] Externe Provider-Abnahme durchführen oder ausdrücklich als offen ausweisen.

### 5. Struktur und Dokumentation bereinigen — offen

- [ ] Überflüssige Runner-Abstraktionen, Konfigurationen, Ordner und Doppelungen entfernen.
- [ ] Veraltete Beschreibungen und Beispiele in der bestehenden Dokumentation ersetzen.
- [ ] Einstieg kurz halten und auf ausführbare Tests statt doppelte Erklärungen verweisen.
- [ ] Prüfen, ob ein Nutzer einen Feature-Test finden, verstehen und auf sein Spiel übertragen kann.

Der alte Runner und seine Dokumentation unter `tests/` sind entfernt. Frühere
Prüfstände in `docs/api/` sind historisch und gelten nicht für den testfreien
Neustart. Ein grüner Pilot in Schritt 3 ist noch keine vollständige Gesamtabnahme.
