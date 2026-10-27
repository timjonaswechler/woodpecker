# Überarbeitung der Woodpecker-Testsuite

Dieser Plan ist die Arbeitsliste des Neuaufbaus, kein Gesamtabnahmebericht.
Ausgangscommit nach Entfernung des Altbestands: `f9a39ed`.

**Stand:** Alle 94 Produkttests sind erstellt. Für alle 93 lokalen Fälle gibt es
Erfolge aus den Abnahmeetappen beziehungsweise gezielten Nachprüfungen; das ist
kein erfolgreicher gemeinsamer Lauf auf dem finalen Stand. Der letzte Gesamtlauf
scheiterte bei 3.12 an fehlender Fensteroberfläche. 4.80 bleibt offen, 4.81 ohne
benanntes externes Testrepository ebenfalls. Keine automatischen Wiederholungen
oder aus Teilbelegen konstruierte Gesamtabnahme.

## Full-stop-Abnahme und offene Checkpoints

Der Nutzer hat die benötigten Testmethoden einschließlich echter grafischer
Läufe freigegeben. Ausgeführt wurden der gemeinsame Einstieg, gezielte
Fehlernachweise und die lokalen CI-Befehle; kein Headless-Ersatz,
keine Produktmutationen und keine echten GitHub-Issues.

**Gefunden und korrigiert:**

- Snapshot-Fixture: fehlende TOML-Tabellen mit Table::insert statt panischem
  Indexzugriff anlegen; echter Snapshot-Fall danach bestanden.
- Runner: die bestätigte Negativkontrolle nicht als failfast-Benutzerabbruch
  werten. Regression zuerst rot, danach grün; falscher Fehler, Cleanup-Fehler
  und echter Stop halten den Lauf weiterhin an.
- Script-Pipeline: Annahme von Inspect/Stop vor dem Warp-Outcome statt
  unzulässiger Completion-Reihenfolge im selben Serverdurchlauf prüfen.
  Vollständige Outputs, Grenzen und Recording bleiben streng; echter Fall grün.
- Pace: Mindestbeobachtung an die gemessene Basiskapazität binden, keine
  unerreichbare Ziel-Durchsatzgarantie verlangen. Obergrenze und Ausschluss
  alter Pace unverändert. Aufgezeichneter Messfall erst rot/grün, danach alle
  drei echten Tick-Fälle bestanden.
- CI: nur erwarteten Exit 2 **und vollständig bestandenen ausgewählten Umfang**
  akzeptieren; Skip-/Fehlerausgaben bleiben rot. Clippy-int_plus_one durch die
  äquivalente Integer-Bedingung beheben, ohne Friständerung.
- Review S1: Timeout-Eskalation erfasst vor dem Kill die tatsächlich lebende
  eigene Prozesshierarchie, prüft Identität/Abstammung jeder Gruppe erneut und
  beendet Blattgruppen vor ihren Eltern. Build-, Server- und Live-Transport-
  Cleanup teilen dieselbe Grenze. Fremde, wiederverwendete oder umgehängte PIDs
  werden nicht signalisiert; erzwungenes Cleanup bleibt rot. Isolierte Regression
  erst rot/grün, danach drei reale eigene Prozessgruppen verschwunden geprüft.

**Belegt:** Alle elf lokalen Buildschritte, Workspace-Alltargets-Check, Clippy
mit -D warnings, Format, 28 isolierte Harness-Kontrollen und der tatsächliche
fensterlose CI-Shellschritt bestanden. Alle lokalen Produkttests besitzen
Einzelbelege; die beabsichtigte Kontrolle 3.18 wurde typisiert bestätigt.
Screenshot-Fixture als echtes PNG zusätzlich betrachtet. Vollständige Zuordnung:
`target/test-runs/full-stop/checkpoint-audit.json`; Rohbelege und Fehlversuche
bleiben unter `target/test-runs/full-stop/`.

**Unabhängiges Review:** Zwei Runden mit GPT-6.1 Sol/xhigh; S1 am Cleanup
behoben und mit realer Drei-Gruppen-Fixture unabhängig bestätigt. Aktueller Code
APPROVE, aber kein Ready-to-merge: S2 (gemeinsamer Finalstand) und S3 (Live-GitHub)
bleiben ausdrücklich offene Abnahme-Gates.

**Weiter offen:**

- **3.12 / 4.80:** Im letzten Gesamtlauf window surface=false nach drei
  Vorbereitungsticks, obwohl GPU-Daten und Kamera bereit waren. Die folgende
  Desktop-Abfrage zeigt ein bildschirmfüllendes MonitorControl-Overlay und
  ein großes T3-Fenster; die konkrete Verursachung ist nicht bewiesen.
  Keine weiteren Grafikläufe, keine Timeout-Erhöhung, kein Umschalten auf
  Offscreen und keine Änderung persönlicher Anzeigeeinstellungen.
- **4.81:** gh ist angemeldet, aber ein ausdrücklich benanntes Testrepository
  mit dem geforderten Description-Marker fehlt. Auth allein ist kein Providerbeleg.
- **Schritt 5:** abschließende Struktur-/Dokumentationsbereinigung bleibt
  eigenständige Arbeit. Beispiele und Mutationskampagne aus 6/7 nicht begonnen.

## Qualitätsnachschärfung nach dem Drei-Reviewer-Stand

Die [Bewertung](testing-quality.md) bleibt ein historischer Snapshot mit unveränderten
Noten. Die sechs vorgeschlagenen Blöcke sind eingearbeitet:

- **3.12, 4.53, 4.72:** Bewegung auf jedem Hold-Tick, atomare gh-Gate-Datei,
  positives Textziel auch nach dem zweiten Editpass unverändert.
- **4.19, 4.56, 4.57:** Zwei verschiedene reservierte Handles gegen unabhängig
  exportierte native ID-Bits; Report-Metadaten gegen Startkonfiguration;
  vollständige Markdown-Payloads, Abschnittsfolge und geschlossene Fences.
- **3.2, 3.3, 3.17, 4.58, 4.59:** Lebende App-Abstammung und separate Gruppe vor
  Stop erfassen, PID und Gruppe unabhängig danach und beim Server-Cleanup prüfen.
- **3.16, 3.17, 4.47–4.51, 4.53:** Dauerhafte lokale Report-Dateimenge beziehungsweise
  endgültige gh-Aufrufzahl erst nach Serverexit prüfen. Die öffentliche Activity
  besitzt keine Providerabschlussbarriere; vollständiger Queue-/Outcome-Abschluss
  wird deshalb zusätzlich am tatsächlichen internen Owner mit **4.82** belegt.
  Kein neuer öffentlicher API-Vertrag und kein Sleep als Ersatzbarriere.
- **4.5, 4.6, 4.10, 4.13, 4.61–4.63, 4.66:** Natürlichen Abschluss vor Stop,
  nicht trennbare Pace-/Disconnect-Messungen und verbrauchte Beobachtungsbudgets
  nicht als Erfolg behandeln. Vollständig belegte ungeeignete Messlagen werden
  Unvollständig; falsche Outcomes, unerledigte Commands oder Restprozesse bleiben
  Fehler. Pace-Obergrenzen verwenden Tickphasen statt Prozenttoleranzen.
  Signalvergleich gegen die ursprüngliche Frist; kein erfundenes 5s-Drop-SLA.
- **4.81:** Echte externe Abhängigkeiten bleiben ausdrücklich bestehen. Kein
  Ersatz durch Testdoubles, keine neue Veröffentlichung und keine Notenaufwertung.

Aktuell **94 Produkttests**: 75 Python, zehn öffentliche und neun interne
Rust-Tests; 93 lokal, GitHub separat. Einschließlich Kontrolle 3.18 sind es
95 Inventareinträge. Keine Produktmutationen aus Schritt 7 ausgeführt.

**Vor dem full-stop-Grafiklauf verifiziert:** 25 fensterlose Produkttests bestanden (15 Python, neun interne
Rust-Tests, 4.69), 22 isolierte Harness-Kontrollen bestanden; Syntax, Inventar,
Format, CLI-Build, Automation-App-Check und Kompilierung der geänderten öffentlichen
Rust-Targets geprüft. Default-Library: vier Tests bestanden.
Belege: `target/test-runs/quality-hardening/` und
`target/test-runs/acceptance-xhwhupwk/summary.json`.
Der vorherige Runner-Versuch stoppte vor Testausführung an der alten festen
Acht-Test-Prüfung (`target/test-runs/acceptance-fnrawg0c/summary.json`); die korrigierte
Prüfung vergleicht jetzt die tatsächlichen Funktionsnamen mit dem Inventar.
Der finale Teilumfang ist Bestanden; Gesamtabnahme bleibt Unvollständig/Exit 2.

**Damals nicht ausgeführt:** geänderte grafische/R1-Fassungen, echte GitHub-Wirkung,
Mutationstests und vollständiger Gesamtlauf. Den neueren Ausführungsstand beschreibt
jetzt der full-stop-Abschnitt oben; diese frühere Härtung ist kein Gesamterfolg.

## Zweck

Zuerst entsteht die Abnahmesuite unter `tests/` für Woodpeckers Features,
Fehlerfälle und öffentliche Zugänge. Danach folgen getrennte Anwender-Beispiele
unter `examples/`. Die Signalstation in `test_app/` ist Referenzanwendung;
ihre Missionen und Asset-Konventionen sind keine Woodpecker-Features.

## Festgelegte Grundlagen

- **Öffentliche Grenzen:** Python mit `unittest`, primär echte CLI und Command-JSON.
  Sollwerte und Zugänge vor der Implementierung prüfen; Vertragswidersprüche klären,
  nicht das Istverhalten abschreiben. Zusätzliche Fälle zuerst hier einordnen.
- **Keine Doppelungen:** Ein Verhalten hat einen verantwortlichen Test. REPL und
  Scripts prüfen ihre Besonderheiten; Rust-Nachweise brauchen einen eigenen
  öffentlichen Vertrag oder ein öffentlich nicht zuverlässig erzeugbares Risiko.
  Interne Test-Seams werden erst nach dem Abgleich 4.70 bestätigt.
- **Sichtbarer Ablauf:** Vorbereitung, Aktionen, Erwartungen, Tickbudgets und Cleanup
  stehen im Test. `EngineReady` ersetzt keine Spielbereitschaft. Keine versteckten
  Sessionstarts, Inputs, Ticks, Resets oder automatischen Retries, auch nach Activity-Verlust.
- **Unabhängig und seriell:** Keine Abhängigkeiten zwischen Tests; mehrere isolierte
  Sessions innerhalb eines Falls sind erlaubt. Varianten desselben Risikos können
  `subTest` verwenden. Spielwirkung und Cleanup gehören zum Nachweis.
- **Nur testeigene Eingriffe:** Fehler nur an eigenen Prozessen, Artefakten und
  temporären Testrepositories erzeugen; keine persönlichen Einstellungen,
  fremden Dateien oder Änderungen am Projekt-Checkout als Fehlereingriff.
- **Grafikfreigabe:** Vor jedem grafischen Lauf Tests, Appstarts/Fenster, Varianten,
  Eingriffe und Tickbudgets nennen und frische Freigabe abwarten – auch für
  Wiederholungen und negative Kontrollen. `WOODPECKER_TEST_RENDERED=1` ist nur die
  technische Sperre, keine Erlaubnis.
- **Render-Voraussetzungen:** Wacher Desktop, zumindest teilweise sichtbare Fenster
  und während der Session unveränderte Größe, Skalierung und Bildschirmzuordnung.
  Keine angebundene Headless-Spielvariante oder Offscreen-Ausweichlösung.

## Arbeitsliste lesen

- **Test:** außen Code geschrieben, innen erfolgreich ausgeführt.
  Syntax, Discovery, Build oder blockierter Start ersetzen keinen Lauf.
- **Umsetzung/Arbeit:** Vorbereitung, Runner, Dokumentation oder Entscheidung;
  kein zusätzlicher Produkttest. Umsetzung und Prüfung bleiben getrennt.
- **Grafik:** `0` ohne Spielfenster, `1/2` eine/zwei echte Sessions, `+` mehrere
  Varianten/Starts. `R0/R1/R2` bezeichnet Rust-Nachweise ohne/mit einer/zwei Sessions.
- **Fehler/Extern:** `F` benötigt ein Fehlerwerkzeug, `F0` keinen Spielprozess.
  `X` bezeichnet gesondert freizugebende echte GitHub-Wirkung.
  Fensterlose Rust-/Fehlernachweise sind kein Headless-Spielmodus.

## Zielstruktur und erlaubte Hilfen

```text
tests/
  __init__.py  README.md  app.toml  settings.json  _support.py  acceptance.py
  session/  startup/  tick/  inspect/  keyboard/  pointer/  text/
  screenshot/  recording/  replay/  reports/  shutdown/  cli/  repl/
  rust/  faults/  providers/
```

Eine Datei je Test; Python-Dateinamen ohne `test_`-Präfix. `tests/app.toml` und
`tests/settings.json` bleiben die Basis; Varianten entstehen in Laufartefakten.
Hilfen verwalten Konfiguration, Prozesse, Artefakte, Transport, Fristen und reine Abfragen;
keine versteckten Spielaktionen. `unittest` und Standardwerkzeuge übernehmen
Discovery und Ausführung, keine neue Runner-Architektur, Testsprache oder
Szenariokataloge. Leere Ordner, Dummytests und Rust-Targets mit null Tests sind
kein Nachweis; Rust-Dateien brauchen tatsächliche Target-/Modulverdrahtung.
Importe, Helper-APIs, Discovery-Befehle, Pillow und Fensterpositionierung stehen
in der [Testanleitung](../tests/README.md).

## Gesamtabnahmelauf

Der gemeinsame Einstieg führt dieselben Einzeltests und erforderlichen
Rust-/Fehlernachweise aus, keinen zweiten monolithischen Spielablauf.

1. Sollumfang, Freigaben, Render-Voraussetzungen und Abhängigkeiten vor Prozessstart prüfen.
2. Commit, Dirty-Zustand, Versionen und Konfiguration festhalten; Targets einmal bauen
   oder verwendete Artefakte ausdrücklich validieren.
3. Seriell und isoliert ausführen. Nach Fehlern nur unabhängige Tests fortsetzen,
   wenn ihre Vorgänger sicher bereinigt sind.
4. Pro Feature Tests, Ergebnis und Artefaktpfade melden, dann das Gesamturteil.
   Journale/Logs müssen Session, Request, Aktion, Erwartung, Outcome und Cleanup zuordnen.

| Gesamturteil | Bedingung |
| --- | --- |
| **Bestanden** | Alle verpflichtenden Nachweise erfolgreich; keine ungeklärten Outcomes oder Prozessreste. |
| **Fehlgeschlagen** | Unerwarteter Verhaltens-, Build- oder Cleanup-Fehler. |
| **Unvollständig** | Voraussetzung/Nachweis fehlt, Test übersprungen oder Lauf abgebrochen; kein erfolgreicher Gesamtexit. |

Erwartetes `unknown` oder Produktende gilt nur als ausdrücklich geplanter Fehlerfall.
Erzwungenes Harness-Cleanup ist nie Erfolg; Cleanup-Fehler verdecken den Erstfehler nicht.
Echter GitHub-Erfolg braucht einen separat freigegebenen Testrepository-Lauf.
Standardläufe erzeugen keine echten Issues; lokales Grün ist kein externer Nachweis.

## Checkliste und Arbeitsstand

### 1. Bestand prüfen — abgeschlossen

- [x] Tests ihrem tatsächlichen Woodpecker-Verhalten zuordnen.
- [x] Doppelungen, Implementierungsbindungen und unnötige Infrastruktur identifizieren.
- [x] Behalten, Ersetzen, Zusammenlegen und Entfernen gruppieren.

Quelltextanalyse auf `84857b4`, ohne Testlauf: Der alte Keyboard-Filter startete
den ganzen Signalstation-Durchlauf; Helfer enthielten zusätzliche Ticks/Aktionen,
Dokumentation und Konfiguration widersprachen sich. Dieser Bestand ist inzwischen
auf Nutzerauftrag entfernt; die Analyse ist nur historischer Ausgangspunkt.

### 2. Zielstruktur festlegen — abgeschlossen und abgestimmt

- [x] Zweck, Sprache, öffentliche Zugänge und Abnahmetestmuster festlegen.
- [x] Zuerst unsere Abnahme, danach getrennte Anwender-Beispiele.
- [x] Feature-Verhalten und vier repräsentative Abläufe festlegen.
- [x] Hilfscode und Struktur begrenzen.
- [x] Einen gemeinsamen Lauf mit ehrlichem Gesamturteil festlegen.
- [x] Alte Tests und reine Testinfrastruktur auf Nutzerauftrag entfernen.

### 3. Testform praktisch erproben — geordnete Einzelfälle

- [x] **3.1 · Server-Lebenszyklus · Code** — `tests/session/server_lifecycle.py` · Grafik **0**. Server starten, leere Session-Liste lesen und Server stoppen → Ready, leere Liste und normaler Prozessabschluss.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.2 · Session-Lebenszyklus · Code** — `tests/session/session_lifecycle.py` · Grafik **1**. Signalstation anlegen, auflisten, inspizieren und stoppen → SessionReady, danach Ended; keine Eingaben oder Tick-Aufträge.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.3 · Unabhängige Session-Lebensdauer · Code** — `tests/session/session_isolation.py` · Grafik **2**. A und B starten, nur A stoppen → B bleibt SessionReady und beantwortet danach einen frischen Inspect-Spielcommand mit dem unabhängigen App-Zähler null; B anschließend separat stoppen. Keine Inputs oder Tick-Aufträge.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.4 · Unbekannte Session · Code** — `tests/session/unknown_session.py` · Grafik **0**. Gültige, nicht vorhandene ID abfragen → session_not_found und Exit 1; Server bleibt nutzbar.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.5 · Ungültige Session-ID · Code** — `tests/session/invalid_session_id.py` · Grafik **0**. Kurze, lange und nicht-hexadezimale IDs abfragen → invalid_session_id; Server bleibt nutzbar.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.6 · Ungültige Startkonfiguration · Code** — `tests/session/invalid_launch_config.py` · Grafik **0**. Leeren Package-Namen übergeben → invalid_config vor Session-Erzeugung; Liste bleibt leer.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.7 · Fehlgeschlagener Start · Code** — `tests/session/launch_failure.py` · Grafik **0**. Fehlendes Manifest angeben → Start zunächst angenommen, danach Failed mit abfragbarer Diagnose; Server bleibt verfügbar.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.8 · Erster Spiel-Command und Resource-Wert · Code** — `tests/inspect/resource_value.py` · Grafik **1**. Eine bekannte registrierte Resource aus dem festen Seed lesen → vollständiger erwarteter Wert; Annahme und terminales Ergebnis derselben Request-ID zugeordnet. Einreichung und vollständige Erwartung stehen im Test; `Server.result` ergänzt nur zugeordnetes CLI-Polling ohne erneute Einreichung. Der feste Seed enthält einen vom App-Default abweichenden Spielernamen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.9 · Manual-Startup ohne Fortschritt · Code** — `tests/startup/manual_startup.py` · Grafik **1**. Nach SessionReady Anwendungsticks und Bevy-`Time<()>` lesen, 0,25 reale Sekunden warten und erneut lesen → anfänglich null Ticks/Zeit und danach unveränderte Werte. Die Referenz-App zählt ihre Last-Schedule-Durchläufe als reflektierte `SimulationTicks`; keine privaten SDK-Zähler, Inputs oder angeforderten Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.10 · Exakte Ticks und Fixed-Clock · Code** — `tests/tick/exact_ticks.py` · Grafik **1**. Anwendungsticks und Bevy-Zeit lesen, ausdrücklich drei maximal schnelle und sieben auf 20 Ticks/s begrenzte Ticks anfordern → passende terminale Completion, unabhängiger App-Zähler steigt exakt um drei bzw. sieben, feste Zeitdifferenzen 0,050000001 bzw. 0,116666669 Sekunden. Nach jedem Warp 0,25 reale Sekunden warten → Zähler und Zeit unverändert. Keine versteckten Ticks, Inputs oder Warp-Stop-Commands.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.11 · Engine-Startup mit gezählten Ticks · Code** — `tests/startup/engine_startup.py` · Grafik **1**. Engine-Konfiguration ausdrücklich aus der Basis in die Laufartefakte schreiben (60 Sekunden, höchstens 3600 Startup-Ticks) → SessionReady vor EngineReady, zugehöriges EngineReady-Event mit passendem unabhängigem App-Zähler und anschließender Stillstand. LevelLoad getrennt prüfen; nötigenfalls höchstens 120 weitere explizite Ticks in zehn Sekunden anfordern und zählen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.12 · Tastatur mit tatsächlicher Bewegung · Code** — `tests/keyboard/movement.py` · Grafik **1**. Explizite Engine-/Spielvorbereitung wie 3.11; arrow_right drücken, vier einzelne Ticks mit jeweils beobachteter Bewegung, loslassen und einen plus drei Ticks stehen bleiben. Vorabwirkung ausgeschlossen; vollständige Warp-Outcomes und unabhängige Zählerstände +1/+4/+5/+8 prüfen. Genau acht Aktionsticks; kein Reset-Helfer oder OS-Input.
  - [ ] **Erfolgreich ausgeführt** — Letzter Gesamtlauf scheiterte vor den Bewegungsticks: startup_timeout nach drei Vorbereitungsticks, GPU-Daten/Kamera bereit, window surface=false; Server wegen Failed-Session Exit 1 statt des normalen Soll-Exits 0, keine Harness-Eskalation. Beleg `target/test-runs/keyboard-movement-2x47wge6/` und `target/test-runs/acceptance-ig4cf4ib/summary.json`. Frühere verschärfte Hold-Tick-Fassung im Etappenlauf bestanden (`target/test-runs/acceptance-yq5as6b4/summary.json`), aber kein letzter Gesamterfolg. Desktop-Oberflächenvoraussetzung ungeklärt; keine Wiederholung ohne geklärte Sichtbarkeit.

- [x] **3.13 · Screenshot ohne Simulationsfortschritt · Code** — `tests/screenshot/capture.py` · Grafik **1**. Explizite Engine-/Spielvorbereitung wie 3.11; genau ein Capture ohne weitere Inputs/Ticks. Bewusster Bild-Fixture-Vertrag: feste Signalstation-Startszene, 1440×900, Skalierung 1 und unveränderte HUD-/Eingangsboden-/Startpad-Grenzen. Konfiguration, Fixture-Version, Rechtecke und RGB-Bänder protokollieren; PNG dekodieren und Zähler/Zeit/Player unverändert prüfen. Visuelle Fixture-Änderungen erfordern eigene Bewertung, keine still gelockerten Toleranzen oder Capture-Retries.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.14 · Recording und Replay in frischer Session · Code** — `tests/replay/recording_roundtrip.py` · Grafik **2**. A und frisches B mit gleicher Engine-Konfiguration, Settings und Spielerstartposition, aber versetzten anfänglichen Fensterpositionen ausdrücklich vorbereiten. In A fünf Ticks samt Eingaben und drei Player-Checkpoints aufnehmen, Datei sichtbar nach B kopieren und dort wiedergeben → reale Zwischen-/Endpositionen stimmen. Eine zweite Recording beobachtet die tatsächlich ausgeführten Replay-Queries, keine externen Inspects während Replay. Zähler und Zeit relativ zu den jeweiligen Starts vergleichen; Resource-Selektoren verwenden keine sessionlokalen Entity-Handles.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.15 · Vorbereitung: App-Diagnoseaktion · Umsetzung** — `test_app/src/diagnostics.rs` · Grafik **0** für die Vorbereitung. Nur mit automation, kontrollierter Session und App-Argument --woodpecker-diagnostics einen normalen Eingabeauslöser installieren: F12 erzeugt beim nächsten expliziten Tick einen tracing-Fehler, F11 eine beabsichtigte Panic. Ohne Opt-in keine Diagnose-Systeme; kein neuer Woodpecker-Command. Modulverdrahtung in lib.rs und woodpecker.rs ist umgesetzt.
  - [x] **Vorbereitung geprüft** — Modulverdrahtung, Automation-Build, normaler App-Build-Check und Format geprüft. Kein eigenständiger 3.15-Test; die Funktionsnachweise für F12/F11 sind durch 3.16/3.17 ebenfalls erbracht.

- [x] **3.16 · Nichtfataler Fehler mit lokalem Report · Code** — `tests/reports/tracing_report.py` · Grafik **1**. App-Diagnose/tracing und kindlokal RUST_LOG=error aktivieren; F12 mit zwei expliziten Ticks drücken/loslassen → zugeordneter Failure und lokaler Report mit Dateireferenz/Signatur. Inspect danach nutzbar, dann Session bei weiter laufendem Server ausdrücklich stoppen und Activity bis Ended begrenzt auf einen Failure/Report prüfen; Ended ist keine Providerbarriere. Nach Serverexit endgültige Report-Dateimenge und Inhalt prüfen, Queue-/Outcome-Abschluss zusätzlich in 4.82; keine Engine-Warm-up-Ticks oder Veröffentlichung.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.17 · Panic, Report und Prozessende · Code** — `tests/reports/panic_exit.py` · Grafik **1**. Panic per App-Eingabe und explizitem Tick auslösen → betroffener Request endet terminal statt mit Erfolg, genau zuordenbarer Panic-Failure ohne zusätzlichen Exit-Failure, Report bleibt lesbar; App-Prozess und Server werden nach ihrem erwarteten Ausgang bereinigt.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **3.18 · Kontrolliert falsche Erwartung im Spielpfad · Umsetzung** — `tests/inspect/resource_value.py` · Grafik **1**. Eine Erwartung für einen separat freigegebenen Kontrolllauf absichtlich falsch setzen → Test schlägt tatsächlich fehl, ursprünglicher Fehler und Cleanup bleiben sichtbar; danach Erwartung wiederherstellen. Kein zusätzlicher dauerhafter Produkttest.
  - [x] **Geprüft** — Im freigegebenen Kontrolllauf bestätigt: vollständige richtige Resource-Antwort, danach genau eine `ExpectationControlFailure` durch den falschen Spielernamen, kein zusätzlicher Fehler. Seed und normale Erwartung blieben unverändert; Server-Exit 0 ohne erzwungenes Cleanup. `unittest.log` bewahrt den echten Testfehlschlag, der Einstieg bestätigt die Kontrolle und meldet die lokale Gesamtabnahme weiterhin Unvollständig mit Exit 2. Beleg `target/test-runs/acceptance-fcdei6uy/`.

- [x] **3.19 · Gemeinsamer Einstieg und ehrliche Zusammenfassung · Umsetzung** — `tests/acceptance.py` · Grafik **0**. unittest und erforderliche Standardwerkzeuge zusammenführen, Sollumfang explizit berücksichtigen → vorhandene Pilotfälle ausführbar; fehlende Feature-Nachweise ergeben Unvollständig statt grüner Gesamtabnahme. Erfolg, Fehler und fehlende Voraussetzungen gezielt erproben.
  - [x] **Geprüft** — Fünf echte fensterlose Tests gemeinsam bestanden und ohne erzwungenes Cleanup bereinigt; Gesamturteil bleibt Unvollständig mit Exit 2. Fehlende Grafikvoraussetzung startet keine Tests; ein absichtlich fehlschlagender externer Cargo-Prozess ergibt Fehlgeschlagen mit Exit 1. unittest-Ergebniszuordnung einschließlich Skip, Subtest- und Cleanup-Fehler separat geprüft. Metadaten, Build-/Testlogs und Pilot-Ergebnisse je Feature stehen in den Laufartefakten. Der nutzergestartete grafische Pilot meldet den echten Fehler in 3.11 als Fehlgeschlagen und die fünf nicht ausgeführten Folgetests separat. Der spätere Spielpfad-Kontrolllauf bestätigt den erwarteten Assertion-Fehler und liefert korrekt Unvollständig/Exit 2. Alle Pilotfälle sind einzeln belegt, aber kein neuer gemeinsamer 16-Test-Lauf wurde ausgeführt; Rust-/Feature-Nachweise aus Schritt 4 sind noch nicht eingebunden.

- [x] **3.20 · Pilotabschluss · Umsetzung** — `docs/testing-plan.md` · Grafik **2+**. Basisfälle und die vier repräsentativen Abläufe 3.12, 3.13, 3.14 und 3.17 einzeln belegen; 3.19 meldet den verbleibenden Umfang korrekt → Testform brauchbar, vollständige Feature-Abnahme weiterhin offen.
  - [x] **Geprüft** — Basisfälle und alle vier repräsentativen Abläufe haben Einzelbelege; der Restblock mit fünf regulären Tests bestand, die negative Kontrolle erkannte ihren echten Fehler und alle Server wurden zum deklarierten Exit eingesammelt. Testform und Pilot sind damit erprobt. Belege stammen aus mehreren Läufen, nicht aus einem neuen gemeinsamen Gesamtpilot; die vollständige lokale Abnahme aus Schritt 4 bleibt offen.

#### Bisherige Belege, keine neue Gesamtabnahme

**Artefaktverfügbarkeit:** Während der späteren 4.40-Korrektur war `target/`
plötzlich nicht mehr vorhanden; Ursache nicht ermittelt. Die unten genannten
älteren Artefaktpfade sind dadurch nicht mehr zugänglich. Historische Ergebnis-
notizen bleiben erhalten, ersetzen aber keine erneut prüfbaren Belege oder 4.80.
Neue CLI-/15-Python-Nachweise: `target/test-runs/cli-utf8-recheck-2mmCAR/`.

3.1–3.10 sind durch den nutzergestarteten Pilotlauf
`target/test-runs/acceptance-4ho077i6/` belegt. Dessen Builds bestanden;
3.11 schlug fehl, die fünf Folgetests wurden wegen Fail-fast nicht ausgeführt.
Die Cleanup-Dateien der zehn bestandenen Tests bestätigen die deklarierten
Exitcodes ohne erzwungene Bereinigung. Der separat freigegebene Einzellauf
`target/test-runs/engine-startup-vo7s38k6/` reproduzierte den Timeout nach drei
Ticks mit `window surface=false` und unerwartetem Server-Exit 1, ohne erzwungenes
Harness-Cleanup. Nach Freiräumen der Fenstermitte bestand ein weiterer separat
freigegebener Einzellauf unter `target/test-runs/engine-startup-nt50_b2v/`.
Die erzeugten Engine-Konfigurationen beider Einzelläufe sind bytegleich;
Test-/Produktcode und Fokusoption wurden nicht geändert. Anschließend bestand
der separat freigegebene Restblock 3.12–3.14, 3.16 und 3.17; Originalausgabe und
Exit 0 stehen in `target/test-runs/pilot-remaining-u81ewO/`. Der Kontrolllauf 3.18
ist in `target/test-runs/acceptance-fcdei6uy/` bestätigt. Genau sieben zusätzliche
Sessions wurden gestartet, alle sechs beteiligten Server sind zum deklarierten
Exit ohne erzwungenes Cleanup eingesammelt. Damit bestehen sechzehn erfolgreiche
Pilot-Einzelbelege und die bestätigte negative Kontrolle, aber weiterhin keine
vollständige Gesamtabnahme. Die früheren Belege bleiben unten erhalten.

- Session-Lebenszyklus: `target/test-runs/session-lifecycle-q3qnpxcr/`;
  zugehöriger Serverlauf: `target/test-runs/server-lifecycle-pv8nmb2c/`.
  Keine Inputs/Ticks und keine Änderung persönlicher Einstellungen.
- Unbekannte ID: `target/test-runs/unknown-session-ab_uglk7/`.
- Weitere Fehlerfälle: `target/test-runs/invalid-session-id-wdk9ecnu/`,
  `target/test-runs/invalid-launch-config-2g4wilj8/`,
  `target/test-runs/launch-failure-gvkiflik/`.
  Die fünf Tests ohne Spielstart bestanden gemeinsam; falsche Erwartungen wurden
  separat erkannt, kein Server benötigte erzwungenes Cleanup.
- Gemeinsamer Einstieg, nur die fünf Tests ohne Spielstart:
  `target/test-runs/acceptance-cwzse1df/`. Auswahl bestanden, Server ohne
  erzwungenes Cleanup eingesammelt; lokale Gesamtabnahme ausdrücklich
  Unvollständig, Exit 2.
  `target/test-runs/acceptance-f_vngdkm/` belegt einen absichtlich fehlschlagenden
  externen Cargo-Prozess mit Exit 1, keinen Produktfehler.
  `target/test-runs/acceptance-5a1mlqn1/` belegt nur den vor Prozessstart
  gesperrten Kontrolllauf, nicht die Spielwirkung oder Durchführung von 3.18.
- Aktuelle ausführbare Aufrufe stehen ausschließlich in
  [tests/README.md](../tests/README.md). Die früher einmal erteilte grafische
  Freigabe gilt nicht für einen weiteren Lauf.

### 4. Fehlende Feature-Nachweise — in dieser Reihenfolge

Bereits in Schritt 3 verantwortetes Verhalten wird hier **nicht** noch einmal als
neuer Normalfall angelegt. Jeder folgende Test braucht Code- und Laufnachweis.
Beim gezielten Risikoabgleich können interne Kandidaten nur durch ausdrücklich
belegte öffentliche Abdeckung entfallen, nicht stillschweigend.

#### Startup und Tick — 4.1 bis 4.7

- [x] **4.1 · Vorbereitung: fehlende Referenz-App-Fähigkeiten · Umsetzung** — `test_app/src/diagnostics.rs; test_app/src/diagnostics/` · Grafik **0** für die Vorbereitung. Vorhandene Ladezustände, Startup-Asset-Adapter, Tick-/Zeit-Resources und F11/F12 weiterverwenden. Getrennte opt-in-Modi für Startup-Voraussetzungen, App-eigene 50-ms-Clock samt Pause/Geschwindigkeit und Reflection-Beispieldaten sind ergänzt; [Argumente und Daten](../test_app/README.md#vorbereitung-weiterer-abnahmefälle-41). Keine neue Woodpecker-API, Headless-Variante oder zusätzliche Dependencies; bestehende normale und Automation-Buildvariante genügen. Weitere spezielle Fehlerauslöser/Abort-Builds folgen erst bei ihren Nachweisen.
  - [x] **Vorbereitung geprüft** — Automation-Build, normaler App-Check ohne Automation und Rustfmt bestanden; Logs unter `target/test-runs/diagnostics-preparation-QJ0FJw/`. Kein eigenständiger Abnahmetest: Startup-/Clock-Modi sind inzwischen durch die freigegebenen Tests 4.2/4.4 live belegt; Reflection-Funktionsnachweise gehören zu 4.16–4.19 und bleiben offen. Für 4.18 ist die nur beim Reflection-Opt-in aktive F10-Hierarchieaktion ergänzt; ein expliziter Tick entfernt/ersetzt vier echte Bevy-Entities, ohne SDK-API oder automatische Ticks. Aktuelle Build-/Format- und fensterlose Regressionsbelege: `target/test-runs/inspect-checks-TXEV8o/`.

- [x] **4.2 · Startup-Grenzen und fehlende Voraussetzungen · Code** — `tests/startup/startup_failures.py` · Grafik **1+**. Vier Diagnosevarianten seriell mit je eigenem Server/App: missing-scope, pending, missing-asset und no-camera → passender StartupFailed-Event und Failed-Diagnose, kein EngineReady. Je höchstens 3600 Startup-Ticks/zehn Startup-Sekunden; pending hat maximal zwei Ticks, missing-scope erwartet null. Keine Eingaben oder extern angeforderten Ticks. Jeder Server wird vor dem nächsten Start mit erwartetem Exit 1 wegen Failed-Session eingesammelt; erzwungenes Harness-Cleanup bleibt Fehler.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.3 · Ungültige Startup-/Tick-Konfiguration · Code** — `tests/startup/invalid_config.py` · Grafik **0**. 15 gültige TOML-Dateien mit ungültigen Startup-Budgets, Pace- und Fixed-Clock-Werten prüfen, einschließlich nichtendlicher und nicht darstellbarer Dauern → CLI-Exit 1, keine Session und kein Launch. Eine externe Cargo-Startbarriere nur im Server-Kindprozess verhindert auch bei einer Regression ein Spielfenster; ihr Aufrufmarker muss fehlen. Keine Wiederholung des leeren Package-Namens aus 3.6.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.4 · Application-Clock beibehalten · Code** — `tests/tick/application_clock.py` · Grafik **1+**. Drei frische Appstarts seriell mit Manual-Startup und SDK-Clock application: App-eigene 50-ms-Clock in steady/double/paused. Je drei explizite Ticks → unabhängiger App-Zähler +3, virtuelle Zeit +0,15/+0,30/+0 Sekunden und Delta 0,05/0,10/0 Sekunden; Pause und relative Geschwindigkeit bleiben erhalten. Nach 0,25 realen Sekunden ohne Warp unveränderte Ticks/Zeit/Policy. Keine Inputs oder Bereitschaftsticks; jeder Server endet vor dem nächsten Start normal.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.5 · Laufenden Warp stoppen · Code** — `tests/tick/stop_warp.py` · Grafik **1**. Eine Manual-Session, keine Inputs: 600 Ticks bei 10 Ticks/s und danach 50.000 maximal schnelle Ticks anfordern, beide nach beobachtetem App-Fortschritt ausdrücklich stoppen → Start endet stopped mit echter Teilzahl passend zum unabhängigen App-Zähler, Stop bestätigt laufende Arbeit. Je 0,25 reale Sekunden danach unveränderte Ticks/Zeit. Natürlicher Abschluss vor Stop zählt nicht als Unterbrechungsnachweis; kein automatischer Wiederholungs- oder Vergrößerungsversuch. Maximal 50.600 angeforderte Ticks, keine Startup-Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/full-stop/tick-tail-fix.log`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.6 · Pace ändern, nicht Simulationszeit · Code** — `tests/tick/set_pace.py` · Grafik **1**. Engine-Variante mit 60 Sekunden/3600 Startup-Ticks; höchstens 124 explizite Ticks ohne Inputs. Vorhandene atomare App-Dateiprobe ausdrücklich aktivieren. 16 Ticks bei 8/s kalibrieren, unter 6 beobachteten Ticks/s oder zu wenigen Beobachtungen Unvollständig/Skip. Default 0,5 setzen, 100er-Warp mit Override 2/s starten und aktiv auf 8/s ändern; zwei 2,5-Sekunden-Fenster nur über Dateizähler und monotone Leseklammern messen, keine CLI-Latenz im Messfenster. Ausdrücklich stoppen, acht Ticks mit geerbtem Default ebenso beobachten; Fixed-Zeit 0,016666667 pro unabhängig gezähltem Tick. Keine Leistungsgarantie, exakten Millisekunden oder Retries. Obergrenze mit einer Tickphase statt Prozentband prüfen; zu langsame oder von altem Default/Override nicht unterscheidbare Messung nach verifiziertem Abschluss Unvollständig statt Grün.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/full-stop/tick-tail-fix.log`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.7 · Warp-Zustands- und Wertfehler · Code** — `tests/tick/warp_errors.py` · Grafik **1**. Eine Manual-Session ohne Inputs: Null-Ticks und Request-Pace 0/−1 im Idle ablehnen, ohne Tick-/Zeitänderung. Danach 100er-Warp bei 2 Ticks/s starten; zweiten gültigen Start sowie SetPace 0/−1 passend ablehnen und frischen Fortschritt des ursprünglichen Warps beobachten. Keine undokumentierte Fehlerpriorität bei gleichzeitig ungültigem Wert und Zustand voraussetzen. Ausdrücklich stoppen, Idle-Stop als No-op ohne Tick-/Zeitänderung prüfen und einen frischen Tick ohne Pace-Override abschließen. Höchstens 101 gültig angeforderte Ticks, keine Startup-Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/full-stop/tick-tail-fix.log`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Session, Verbindung und Zugriff — 4.8 bis 4.15

- [x] **4.8 · Session-Auswahl und Bindung · Code** — `tests/session/session_binding.py` · Grafik **1**. Eine Manual-Session ohne Ticks/Inputs: volle ID und eindeutiges Acht-Zeichen-Präfix über Verwaltungsabfrage sowie WebSocket-Poll/Inspect-Command verwenden. Activity enthält stets die volle ID; danach ausdrücklich per Präfix stoppen und Ended mit unveränderter ID, Erstellungszeit und Artefaktpfad über beide Selektoren erneut lesen. Liste enthält weiterhin genau diesen einen Eintrag, keine neue Session oder Umwahl. Ambige Präfixe bleiben beim Identitäts-Risikofall 4.76.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.9 · Request-Zuordnung über Clients und Sessions · Code** — `tests/session/request_correlation.py` · Grafik **2**. Zwei versetzte Manual-Sessions, keine Inputs; jede Einreichung durch einen frischen CLI-Client. A fordert 60 Ticks bei 2 Ticks/s an und wird früh gestoppt, B schließt drei maximal schnelle Ticks ab. Erste Request-IDs beider Sessions müssen zahlenmäßig gleich sein; auch die jeweils zweite ID ist gleich, aber ihre Inspect-Ergebnisse sind Settings in A und App-Zähler in B. A-Inspect und B-Abschluss müssen vor A-Warp-Abschluss beobachtbar sein. Pending-Reihenfolge je Session monoton, je Request genau eine passende Completion und ausschließlich eigene Cursor. Höchstens 63 angeforderte Ticks, keine Startup-Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.10 · Angenommene Arbeit nach Disconnect · Code** — `tests/session/disconnect.py` · Grafik **1**. Eine Manual-Session mit ausdrücklich aktivierter vorhandener App-Dateiprobe, keine Inputs. Genau einmal zwölf Ticks bei 2/s einreichen; Einreicher bereits beendet, bevor neue Beobachter verbinden. Nach mindestens zwei unabhängig gezählten Ticks 0,75 Sekunden ohne CLI-/REPL-Clients warten und weiteren Fortschritt rein per atomarer Datei vor jeder Wiederverbindung belegen. Danach ursprüngliche natürliche Completion mit zwölf echten Ticks, genau eine Annahme/Completion; keine Resubmission oder Warp-Stop.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.11 · Getrennte virtuelle Eingaben und Spielzustände · Code** — `tests/session/input_isolation.py` · Grafik **2**. Zwei versetzte Sessions mit je höchstens 3600 Engine-Startup-Ticks/60 Sekunden und 120 sichtbaren Spielbereitschaftsticks/zehn Sekunden; danach zwei Aktionsticks je Session. Nur A erhält arrow_right, MoveTo [720,450] und mittlere Press/Release. Jeweils B zuerst ticken, während A-Eingaben noch vorgemerkt sind → nur A bewegt sich; B-Spieler, Keyboard-/Mouse-ButtonInput und PointerLocation/PointerPressState bleiben unverändert. Pointer ausschließlich über öffentliche Inspect-Components entdecken, keine Namen/UUIDs aus SDK-Interna und keine Handles zwischen Worlds übernehmen. Keine Wiederholung der Lebensdauerprüfung aus 3.3 oder neue App-Fixture.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.12 · Getrennte Settings und Dateien · Code** — `tests/session/file_isolation.py` · Grafik **2**. Zwei versetzte Sessions, je höchstens 3600 Startup-Ticks/60 Sekunden und 120 Spielbereitschaftsticks/zehn Sekunden. Vor jeder Änderung eigene Workspace-Settings aus dem festen Seed und getrennte Roots/Dateien bestätigen. Nur A erhält Tab-Press/Release und Texteingabe „!“; vier A-Aktionsticks synchronisieren echte UI, Settings und automatische Speicherung. Ein B-Tick bestätigt unabhängigen Zustand. Marker genau einmal im A-Namen, übrige Werte und B-Datei unverändert; beide Dateien bleiben nach Shutdown erhalten. Persönliche Settings ausschließlich per Hash prüfen, niemals sichern, überschreiben oder wiederherstellen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.13 · Activity-Lücke und Wiederaufnahme · Code** — `tests/session/activity_gap.py` · Grafik **1**. Eine Manual-Session ohne Inputs, 4096 Byte Activity; 100er-Warp bei 2/s früh stoppen. Höchstens 64 ausdrücklich protokollierte Metadata-Queries in 20 Sekunden, bis alter Cursor Gap/unknown mit CLI-Exit 1 liefert und Warp-Pending nachweislich verdrängt ist. Eigener laufender Cursor beobachtet jede Query; keine erneute Einreichung oder automatische Gap-Reparatur. Frischer Plain-Pipe-REPL-Snapshot muss dieselbe Pending-ID/Command zeigen, ohne Abhängigkeit von Überschriften oder ganzen Ausgabesätzen. Explizit gewählter neuer Cursor beobachtet Stop und ursprüngliche Completion.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.14 · Nur lokaler Zugriff · Code** — `tests/session/local_access.py` · Grafik **0**. Wildcard-Bindung mit invalid_config vor Artefaktanlegung ablehnen; ein exklusiv reservierter Loopback-Port verhindert auch bei Validierungsregression einen exponierten Listener. Origin https://example.invalid, null und leer auf GET /sessions und POST /stop müssen jeweils 403/unsupported_origin liefern. Danach bleibt echte lokale CLI-Verwaltung nutzbar. Direkte HTTP-Anfragen ausschließlich für diesen Transportvertrag; keine Sessions/Spiele gestartet.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.15 · Handshake und beschädigte Spielantworten · Code** — `tests/session/protocol_faults.py; tests/faults/protocol_peer.py` · Grafik **F0**. Zehn serielle Varianten über echte CLI und externen JSONL-Fehlerprozess: falsche Version, typfalsche Capabilities, fehlende Fixed-Clock-/Engine-Capability; nach Ready beschädigte bekannte ID, falscher Command-Name, falscher Output, unlesbare Zeile ohne ID, unbekannte ID und doppelte Antwort. Korrelierte Fehler müssen genau den Auftrag mit invalid_response schließen; unkorrelierte Fehler lassen ihn bis zum ausdrücklichen Stop offen, Duplikate erzeugen keine zweite Completion. Nur externer CARGO-Aufruf wird für ein testeigenes Sentinel-Paket adaptiert; echtes Cargo liefert dessen Metadaten. Auch ohne Adapter kann dieses Paket nur mit Exit 86 enden, niemals ein Spiel starten. Kein Bevy-/Headless-Spiel, keine neuen Produkt-APIs oder Dependencies.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Inspect — 4.16 bis 4.19

- [x] **4.16 · Resource-Metadaten und fehlende Typen · Code** — `tests/inspect/resource_metadata.py` · Grafik **1**. Eine Manual-Session mit Reflection-Opt-in, keine Ticks/Inputs; Activity ausdrücklich auf 16 MiB für ungekürzte All-Value-Ausgaben gesetzt. All/Type und Metadata/Value prüfen: sortierte eindeutige Type Paths, Metadaten ohne Werte/Lesbarkeitsstatus auch bei NaN/opaken Resources, AbsentResource als leere Metadata bzw. Missing-Value, unbekannter Type Path als Ablehnung. Nicht reflektierbare Resource ist per Type-Metadata sichtbar, nicht in Resource-All. 3.8 bleibt Eigentümer des einfachen vollständigen Settings-Wertes.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.17 · Entity-Filter und Component-Projektionen · Code** — `tests/inspect/entity_query.py` · Grafik **1**. Eine Manual-Session mit Reflection-Opt-in, keine Ticks/Inputs: Summary, ComponentNames, All/Listed und positive/negative with/without-Kombinationen mit bekannter Vier-Component-Entity prüfen. All vollständig und sortiert; Listed in gewünschter Reihenfolge einschließlich Missing, NotReflectable und entdecktem NotRegistered. Unbekannte Filter-/Listed-Pfade lehnen die ganze Query ab. Vorhandene Resource-Typen ergeben als Entity-Filter keine Treffer, komplette World-Summary bleibt unverändert.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.18 · Hierarchie und veraltete Handles · Code** — `tests/inspect/hierarchy_handles.py` · Grafik **1**. Eine Manual-Session mit Reflection-Opt-in: vierteilige echte Bevy-Hierarchie durch flache Query entdecken, unabhängige Sollbäume für Tiefen 0/1/2/255 prüfen. Children bewusst Z vor A und gegen die Handle-Sortierung, Blatt ohne Namen. F10-Press/ein Tick ersetzt die Hierarchie, Release/ein Tick beendet die Flanke; genau zwei Ticks, keine Panic. Neue Handles bei gleicher Struktur, alle vier alten und ein fehlender Handle müssen entity_not_found liefern statt stiller Umleitung. Keine erzwungene Indexvergabe oder behauptete Index-Recycling-Garantie.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.19 · Reflection-Werte und Unavailable-Diagnosen · Code** — `tests/inspect/reflection_values.py` · Grafik **1**. Eine Manual-Session mit Reflection-Opt-in, keine Ticks/Inputs: String-/Integer-Key-Maps, sortiertes Set, Unit/Newtype/Struct-Enums und Image-Handles gegen bekannte Beispiele prüfen. Pfad und ausdrückliche Fixture-UUID sind literal vorgegeben; zwei verschiedene flüchtige Handles als exakt 16 kleine Hex-Zeichen gegen unabhängig exportierte native Indexbits prüfen; jeweilige Typed/Untyped-Referenz identisch und nur innerhalb derselben Session stabil. NaN, ungeeignete Map-Keys und opaker Wert ohne Serializer ergeben NotSerializable; nicht reflektierbare Resource NotReflectable. Serialisierbares opakes Null bleibt Readable(null), Tick-/Zeitwerte unverändert. Component-NotRegistered bleibt bei 4.17.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Keyboard — 4.20 bis 4.21

- [x] **4.20 · Einmalige Aktion trotz Hold · Code** — `tests/keyboard/single_action.py` · Grafik **1**. Eine Session mit höchstens 3600 Engine-Startup-Ticks/60 Sekunden und 120 Spielbereitschaftsticks/zehn Sekunden, danach sieben Aktionsticks. Vorhandene Gesamtkarte per g einschalten; drei gehaltene Ticks einzeln beobachten, damit gerade Anzahlen unerwünschter Toggles nicht im Endbild verschwinden. Modus bleibt Map und KeyG tatsächlich gehalten, Release schaltet nicht um; erst erneute Press wechselt nach Overview. Keine Bewegungseingaben, keine neue App-Fixture oder SDK-API.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.21 · Ungültige Tastenwechsel · Code** — `tests/keyboard/invalid_transitions.py` · Grafik **1**. Eine Manual-Session, genau vier Ticks; nur gültige ShiftLeft-Press/Release. Doppelte ShiftLeft-Press, unbegründete ControlLeft-Release und unbekannte Token für Press/Release sowohl bei vorgemerkter als auch angewandter Press prüfen. Acht Ablehnungen dürfen weder die vorgemerkte Press verlieren noch gehaltenen ButtonInput verändern/neue Flanken erzeugen. Nach gültiger vorgemerkter Release muss eine zweite Release als neunte Ablehnung key_not_pressed liefern, obwohl Bevy bis zum nächsten Tick noch held zeigt. Danach korrekte Release-Flanke und leeres Idle, kein Tick durch Fehler-Commands. Kein Bewegungs-Normalfall und keine behauptete Prüfung aller logischen Modifier-Aliase.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Pointer — 4.22 bis 4.26

- [x] **4.22 · Absolute Position und echtes UI-Picking · Code** — `tests/pointer/ui_click.py` · Grafik **1**. Engine-/Spielvorbereitung mit höchstens 3600 Startup-Ticks/60 Sekunden und 120 Bereitschaftsticks/zehn Sekunden, danach drei Aktionsticks. Gesamtkarte-Button über vorhandenen Text Gesamtkarte [G] und echte öffentliche Hierarchie-/Text-/Layout-Components finden, nicht TabIndex. Mittelpunkt mit inverse_scale_factor in logische Pixel umrechnen. MoveTo/Press/Release ausdrücklich ticken; Vorabwirkung ausschließen, echte Pressed-Component und Map-Aktion ohne Release-Toggle prüfen. Keine neuen App-Marker oder SDK-APIs.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.23 · Relative Bewegung und FPV · Code** — `tests/pointer/relative_motion.py` · Grafik **1**. Engine-/Spielvorbereitung wie 4.22, danach neun Aktionsticks. Mittelpunkt-Start, summierte/Null-Deltas und positive/negative Randüberschreitungen; vollständige MouseMotion und unabhängige FPV-Yaw/Pitch-Sollwerte. Positive Randposition liegt weniger als 0,001 logische Pixel unter der exklusiven Fenstergrenze, ohne einen bestimmten f32-Nachbarwert vorzuschreiben. MoveTo erzeugt keine relative Motion, Idle wiederholt nichts. Keine neue Fixture oder OS-Cursor-Aktion.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.24 · Button-Zustände · Code** — `tests/pointer/buttons.py` · Grafik **1**. Eine Manual-Session, genau acht Ticks: MoveTo vormerken, Left/Right/Middle jeweils einzeln drücken und ticken; danach Hold, drei einzelne Releases, Idle. Nach jeder Press und Release Picking- und tatsächlichen Bevy-ButtonInput samt Flanken vergleichen, damit vertauschte Einzelzuordnungen nicht in einer Gesamtmenge verschwinden. Keine Wirkung vor Tick, konstante Position, keine neue Spielmechanik.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.25 · Scroll-Deltas · Code** — `tests/pointer/scroll.py` · Grafik **1**. Eine Manual-Session, genau sechs Ticks. Horizontal positiv, vertikal negativ, gemischte Vorzeichen, Null und zwei gemeinsam vorgemerkte Scrolls gegen vorgegebene Summen prüfen; sechster Tick ohne Input löscht das Delta. Tatsächliche Bevy-Resource AccumulatedMouseScroll muss Line und korrekte Deltas zeigen; Position/Buttons unverändert, vor Tick keine Wirkung. Bevy-Wirkung statt zusätzlicher UI-Scroll-Fixture.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.26 · Ungültige Pointer-Eingaben · Code** — `tests/pointer/invalid_input.py` · Grafik **1**. Eine Manual-Session, genau vier Ticks und 19 geplante Ablehnungen. Press/Scroll ohne Position sowie vier verletzte Fenstergrenzen, unbekannte Press-/Release-Buttons, doppelte Middle-Press und unbegründete Left-Release bei vorgemerktem bzw. angewandtem Zustand prüfen. Fehler dürfen gültige vorgemerkte Position/Press und angewandte Pointer-/Buttonzustände nicht beschädigen oder Ticks erzeugen. Gültige vorgemerkte Release macht eine zweite Release bereits unzulässig; danach frische gültige Position, korrekte Release-Flanke und leerer Idle.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Text — 4.27 bis 4.28

- [x] **4.27 · Unicode in echtem Eingabefeld · Code** — `tests/text/unicode_input.py` · Grafik **1**. Engine-/Spielvorbereitung wie bisher, danach acht Aktionsticks. Einstellungen über vorhandenen Buttontext Einstellungen [M], mehrzeiliges Notizfeld über vorhandenen Name Notizen und öffentliches Layout per Pointer finden/fokussieren; keine TabIndex-Auswahl. Unicode mit Umlaut, Emoji, Zeilenumbruch, kombinierendem Zeichen und japanischem Text vormerken; vor Tick unverändert, zwei feste Edit-/Sync-Ticks liefern exakte Notes und leere physische Tastatur. Keine neue App-Fixture.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.28 · Fokus und UTF-8-Bytegrenze · Code** — `tests/text/input_limits.py` · Grafik **1**. Engine-/Spielvorbereitung wie 4.27, danach 18 Aktionsticks, 4 MiB Activity. Widgets über vorhandene Buttontexte Einstellungen [M]/Zurück zum Spiel und Namen Notizen/Spielername auswählen; keine festen Fokusnummern. Leer-/Normaltext ohne editierbaren Fokus ablehnen; fokussiert leer akzeptieren, 8192 é/16384 Bytes tatsächlich übernehmen, 8193 Zeichen/16385 Bytes ohne Fortschritt ablehnen. Name-Widget lehnt 25 ASCII-Zeichen trotz SDK-Annahme vollständig ab; keine Tick-Retries oder erfundene Trunkierung.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Screenshot — 4.29 bis 4.31

- [x] **4.29 · Capture vor erstem Tick · Code** — `tests/screenshot/before_first_tick.py` · Grafik **1**. Eine Manual-Session, null Ticks/Inputs und genau ein Capture. Testeigenen vorhandenen .png-Sentinel anlegen; screenshot_window_unavailable erwarten, gesamte Screenshot-Dateistruktur und Bytes auch nach Shutdown unverändert. App-Zähler ausdrücklich null, Zeit und Spieler unverändert, Session weiter SessionReady; kein Bootstrap-/Reparaturtick und kein Pillow nötig.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.30 · Wiederholung und Overwrite · Code** — `tests/screenshot/overwrite.py` · Grafik **1**. Eine Engine-/Spielvorbereitung wie 4.27, danach keine Ticks/Inputs und genau zwei Captures desselben Pfads. Beide PNGs mit Pillow vollständig prüfen/dekodieren, Dimensionen 1440×900 und overwritten false/true erwarten. Zwischen beiden Captures nur die erste testeigene Datei durch Nicht-PNG-Sentinel ersetzen, damit ein altes Bild plus falsches Flag nicht genügt. Zähler, Zeit und Spieler bleiben unverändert; keine neuen Szenenfarbprüfungen oder Capture-Retries.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.31 · Screenshot-Pfadsicherheit · Code** — `tests/screenshot/invalid_path.py` · Grafik **1**. Eine Engine-/Spielvorbereitung wie 4.27, danach keine Ticks/Inputs und zwölf erwartete invalid_screenshot_path-Ablehnungen: leer, absolut, Traversal, eingebettetes .., Punkt-/Leerkomponente, Backslash, falsche/Großbuchstaben-Endung, NUL und aus dem Session-Root führende Verzeichnis-/Dateisymlinks. Alle echten Ausweichziele/Sentinels sind testeigen; In-/Outside-Snapshots ohne Symlink-Verfolgung, PNG-Inventar und Marker schützen bestehende Bytes/Links. Zeit und Ticks unverändert, Schutzdateien auch nach Shutdown gleich; kein pauschales Verbot legitimer In-Root-Symlinks behauptet.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Recording — 4.32 bis 4.34

- [x] **4.32 · Dateivertrag und Reihenfolge · Code** — `tests/recording/file_contract.py` · Grafik **1**. Verschiedene aufnehmbare Commands und eine fachliche Ablehnung aufzeichnen → genau ein Header/Footer, Annahmereihenfolge, vollständige typgerechte Outcomes und korrekte Anzahl; keine Request-IDs, Controls, Shutdown, Events oder Reports. Eine Manual-Session, F12-Diagnose mit lokalem nichtfatalem Fehler-Report; ein Tick plus 60er-Warp bei 2/s früh gestoppt, maximal 61 Ticks, ShiftLeft-Press/Release. Zehn Einträge gegen exakten Fixed-v2-Header/Footer und typgerechte ungekürzte Outcomes prüfen; späterer Inspect schließt vor früherem Warp ab.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.33 · Recording-Zustände und Barrieren · Code** — `tests/recording/barriers.py` · Grafik **1**. Start/Stop, doppelte/fehlende Aufnahme und offene frühere Arbeit gezielt kombinieren → richtige Ablehnung bzw. Barriere; erst nach erfolgreichem Header gehören Commands hinein, nach abgeschlossenem Footer nicht mehr. Eine Manual-Session, zwei 60er-Warps bei 2/s früh stoppen, maximal 120 Ticks. Header/Footer bei Control-Abschluss und Completion-Reihenfolge der direkt folgenden Queries prüfen; nur drei innere Commands gehören zur Datei. Acht zusätzliche Exists-/Pfad-/Symlink-Fehler und frisches leeres Recording. Kontrolliert verzögerte Starting-/Stopping-Rennen und recording_transition_in_progress bleiben ausdrücklich bei 4.78.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.34 · Recording-I/O und unerwartetes Ende · Code** — `tests/recording/io_failure.py` · Grafik **1+F**. Echtes Manual-Spiel ohne Ticks/angenommene Inputs, zwei zusätzliche externe Peers. RLIMIT_FSIZE soft 65536/SIGXFSZ ignoriert nur im eigenen Server; Cargo/Spiel und Parent behalten ursprüngliche Limits/Signale. Zu langer Dateiname erzeugt Start-Io. Vier recordbare Text-Requests mit je 16384 festen ASCII-Bytes werden am Spiel mit text_focus_unavailable abgelehnt; ihre bekannten Argumentgrößen füllen die Aufnahme unabhängig vom Resource-Inventar. Nur RecordingFailed, unveränderte Rejections, Dateigröße nahe Limit und nicht volle Serverlogs prüfen; Recovery erhalten. Externe Endvarianten prüfen session_ended/unanswered bzw. RecordingFailed vor Ended mit 64 Bytes Rest. Keine fremden Dateien oder neuen Produkt-APIs.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Replay — 4.35 bis 4.38

- [x] **4.35 · Ungültige Datei vollständig vorvalidieren · Code** — `tests/replay/invalid_recording.py` · Grafik **1**. Version, Header/Footer, Zähler, Zeilen, Command-/Outcome-Form, Pfade und Symlinks als Varianten prüfen, darunter gültiger wirkender Präfix mit defektem Ende → passende Ladefehler vor jeglicher Präfixausführung; World unverändert. Eine Manual-Session, null Ticks; 33 Varianten, unter anderem gültiger g-Press-/Ein-Tick-Präfix mit defektem Ende, Fixed-v2/v1 und Clock-Abweichung. Keine Präfix-Pendings; Settings/Time/View/Keys/Zähler identisch. Zwei gültige leere Dateien mit/ohne LF und g-Press/Release ohne Tick prüfen Recovery und angenommene Key-Queue.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.36 · Replay stoppen · Code** — `tests/replay/stop.py` · Grafik **1**. Replay mit langem Warp starten und stoppen → keine weiteren Plan-Commands, laufender Warp kontrolliert beendet, Start Stopped und Stops passend abgeschlossen; bereits ausgeführte Wirkungen bleiben. Idle-Stop ist No-op. Eine Manual-Session, zwei echte Präfix-Ticks und ShiftLeft-Press, danach 120er-Warp bei 2/s früh stoppen. Die Wartebedingung verwendet jetzt den unabhängigen atomaren App-Zähler (mindestens drei Ticks) statt nicht öffentlicher interner Warp-Pendings; zweite Aufnahme enthält genau vier interne Commands einschließlich Stop, keinen Release-/Ein-Tick-Tail. App-Zähler stimmt mit Ausführungen überein; zwei ausdrückliche Nachbereitungsticks wenden erhaltene Press an und geben sie frei, maximal 124 Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.37 · Replay-Exklusivität · Code** — `tests/replay/exclusivity.py` · Grafik **1**. Während aktivem Replay weitere externe Spielarbeit und zweiten Replay-Start einreichen → fachlich abgelehnt, nur Replay-Stop zugelassen; kein fremder Command schleicht in den Plan. Eine Manual-Session mit 120er-Warp bei 2/s, früh gestoppt. Acht externe Spiel-/Recording-Requests mit replay_in_progress und zweiten Start mit replay_already_running ablehnen. Zweite Aufnahme enthält ausschließlich Warp und internen Stop; Zähler stimmt mit executed_ticks überein, Settings/Keyboard/Pointer und verbotene Dateien unverändert. Anschließend gültige g-Press/Release ohne Tick gegen vergiftete Queue.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.38 · Technische Blockierung des Plans · Code** — `tests/replay/blocked.py` · Grafik **F0**. Mit gezielt defektem Protokollpartner technische Fehler eines Plan-Commands erzeugen → Replay Blocked mit Ursache, keine späteren Plan-Commands und kein fälschliches Completed. Frühere aufgezeichnete Outcomes sind keine Sollwerte des neuen Spiels. Zwei externe Varianten: defekter neuer Warp-Output blockiert mit command_protocol_failed/korrelierter invalid_response, spätere vier Commands erreichen den Peer nicht. Gesunder Peer verarbeitet denselben Plan trotz aller fünf historischen Outcome-Formen vollständig; Original-session_ended-Datei unverändert, neue Aufnahme belegt neue Outcomes. Journal vor Serverstop trennt Plan-Abbruch von späteren Cleanup-Stop-/Shutdown-Nachrichten.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### CLI und Scripts — 4.39 bis 4.43

- [x] **4.39 · TOML-/Dateifehler · Code** — `tests/cli/config_errors.py` · Grafik **0**. Nicht lesbare Datei, unbekannte Felder und nicht unterstützte Config-Version prüfen → verständlicher Fehler vor Session-Erzeugung. Fachliche Owner-Validierung bleibt bei 3.6 bzw. Startup/Tick-Konfiguration. Acht Varianten: fehlende Datei, Verzeichnis, defektes TOML/UTF-8, unbekannte Top-/Launch-Felder, Version 2 und falscher Versionstyp. Echte CLI, leere Session-Liste/-Artefakte; rein kindlokaler Cargo-Tripwire schützt vor versehentlichem Spielstart.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.40 · Command-JSON vor dem Senden · Code** — `tests/cli/command_errors.py` · Grafik **0**. Ungültiges JSON/UTF-8 und strukturell ungültige Commands über echte CLI → jeweils ausdrücklich Exit 1 und invalid_json/invalid_command, kein Senden oder Request-ID. Sechzehn Varianten mit echten argv-Bytes; eigener Loopback-Listener erkennt jeden Bindversuch. Gültiger Shutdown-Präfix plus defektes Tail darf nicht teilweise eingereicht werden.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.41 · Script vollständig vorvalidieren · Code** — `tests/cli/script_validation.py` · Grafik **1**. Fehler am Dateiende, falsche Version und Shutdown an falscher Position hinter gültigen Commands prüfen → keine Einreichung des Präfixes und keine Spielmutation. Eine Manual-Session, null Ticks; neun ungültige Dokumente hinter g-Press-/Ein-Tick-Präfix, passender invalid_script-Index, keine neue Activity, unveränderte Settings/Time/View/Keys/Zähler. Leeres gültiges Script und abschließende g-Press/Release ohne Tick prüfen Recovery und Key-Queue.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.42 · Script-Pipeline, Barrieren und Indizes · Code** — `tests/cli/script_pipeline.py` · Grafik **1**. Kurzes Script mit Warp, Inspect, Recording-Grenzen und fachlicher Ablehnung ausführen → Pipeline ohne allgemeine Ergebnisbarriere; Recording/Shutdown-Barrieren korrekt, spätere Commands trotz fachlicher Ablehnung eingereicht, vollständige Outputs nach ursprünglichem Index und richtiger Exitstatus. Eine Manual-Session: lesendes Passed-Script, danach gemischtes Zehn-Command-Script; zwei Ticks bei 2/s vor Recording, 60er-Warp bei 2/s früh stoppen, Inspect und Stop vor Publikation des Warp-Outcomes annehmen, erwartetes invalid_key, ein weiterer Tick, Recording Stop und normales Shutdown. Höchstens 63 Ticks, keine angenommene Eingabe. Activity belegt Barrieren; Datei enthält genau fünf Commands; vollständige Passed-/Failed-Outputs, sortierte Indizes/IDs und unabhängiger App-Zähler.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.43 · Ungewisse Script-Ergebnisse · Code** — `tests/cli/script_unknown.py` · Grafik **F0**. Verbindung/Korrelation nach teilweiser Einreichung beschädigen bzw. Activity verlieren → bekannte Ergebnisse erhalten, offene unknown, spätere not_submitted; weitere Einreichung beendet, unbestätigter Command nicht wiederholt. Fünf explizite externe WebSocket-Peers: EOF nach unbestätigtem Submit, falscher Submit-/Poll-Call, Gap und falscher Event-Command. Ein vollständiges bekanntes Ergebnis bleibt; bestätigte offene ID 2 ist unknown, unbestätigtes Submit hat keine vertrauenswürdige ID, Tail bleibt not_submitted. Gap-/Korrelationsvarianten lassen die Verbindung offen, um auch versteckte Recovery-RPCs zu erkennen; keine Wiederholung, kein Stop/Shutdown.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### REPL — 4.44 bis 4.46

- [x] **4.44 · Eingabeformen und Parsefehler · Code** — `tests/repl/commands.py` · Grafik **1**. Echte REPL mit help, pending, vollständigen Commands und Inspect-/Tick-/Recording-/Replay-Kurzformen bedienen → richtige Übersetzung und eigene Ausgabe; Parsefehler senden nichts und beenden die REPL nicht. Keine Wiederholung aller Feature-Assertions. Eine Manual-Session, drei ausdrückliche Ticks, keine Spielinputs. Help samt sechs Inspect-Beispielen, 14 Parsefehler und ungültiges UTF-8; Activity belegt lokale Nicht-Einreichung. Aufnahme enthält die 13 erwarteten Command-/Arguments-Übersetzungen, einschließlich dynamischem Entity-Handle und Type Path mit Leerzeichen. Der unbekannte Type-Pfad muss mit rejected/unknown_type_path und exakt erhaltenem Pfad enden; dieselbe Ablehnung muss in der Aufnahme stehen, spätere Queries laufen weiter. Pfade mit Leerzeichen und leerer Replay ohne zusätzliche Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.45 · PTY-Reaktionsfähigkeit und Terminal · Code** — `tests/repl/terminal.py` · Grafik **1**. Echte REPL im PTY während offener Arbeit bedienen → Eingabe und korrelierte Pending-/Ergebnisausgabe bleiben nutzbar; nach normalem Ende und Fehler Terminalzustand wiederhergestellt. Eine Manual-Session, 120er-Warp bei 2/s früh stoppen; keine Spielinputs. Eigene 44-Spalten-PTY, Unicode/Links/Delete/Home/Backspace/End während asynchroner Fremdantwort, vollständige bearbeitete Eingabe und begrenzte Redraw-Spalten. Der unbekannte Unicode-Type-Pfad muss mit unknown_type_path abgelehnt werden; seine Fehlermeldung muss dem bearbeiteten Pfad exakt entsprechen. Normaler quit und zweiter Client mit fehlender Session prüfen Termios-Restaurierung. Supervisor beobachtet vor eigener Session-Leader-Beendigung, setzt keine Attribute zurück; nur der Kernel-Status PENDIN ist vom Konfigurationsvergleich ausgenommen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.46 · REPL beenden ist kein Session-Shutdown · Code** — `tests/repl/disconnect.py` · Grafik **1 + F0**. quit, EOF und Ctrl+C als Varianten bei angenommener Arbeit prüfen → nur Client getrennt, Session/Arbeit bleiben; expliziter shutdown dagegen fordert Session-Ende an. Technisches Session-Ende ergibt REPL-Fehlerexit. Eine Manual-Session, je zwölf Ticks bei 2/s nach quit/Pipe-EOF/PTY-Ctrl+C natürlich vollenden, insgesamt 36 Ticks. Client muss vor Warp-Ende zurückkehren; Activity enthält keinen versteckten Stop/Shutdown und App-Zähler stimmt. Vierter Client führt ausdrückliches Shutdown aus. Danach ein eigener externer recording-end-Peer: offener Warp und Inspect lösen EOF aus, REPL/Server erwarteter Exit 1, kein Cleanup verdeckt den Fehler.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Reports und Provider — 4.47 bis 4.57

- [x] **4.47 · Tracing-Opt-in, Filter und Layer-Bestätigung · Code** — `tests/reports/tracing_config.py` · Grafik **1+**. Beobachtung aus/an, herausgefiltertes Error-Event und fehlenden Layer prüfen → nur tatsächlich dispatchte aktivierte Fehler erzeugen Reports; fehlende Bestätigung verhindert Ready; behandeltes Err ohne Fehlerauslöser erzeugt keinen Report. Fünf serielle Manual-Starts, vier F12-/F9-Ticks und ein erwarteter Launchfehler ohne Layer. Handled-Err-Probe bestätigt ihre Ausführung über ReportProbe, ohne Failure/Report.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.48 · Beobachtete nichtfatale Panics · Code** — `tests/reports/caught_panic.py` · Grafik **1+**. Gezielte abgefangene System-/Task-/Worker-Panics einschließlich panic_any ohne String erzeugen → Panic mit getrennter Location/Backtrace-Diagnose beobachtet, keine erfundene Meldung oder Pflicht zu stabilen Stackframes; fortlebender Prozess nicht allein wegen Marker beendet. Vier Manual-Varianten, F9 und je zwei Ticks (acht total); ReportProbe belegt Catch und weiteren Fortschritt. Location/Erfassungsstatus/Backtrace getrennt, panic_any-Meldung bleibt null.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.49 · Echter panic=abort-Build · Code** — `tests/reports/panic_abort.py` · Grafik **1**. Separaten echten panic=abort-Build mit wiederverwendbarem CARGO_TARGET_DIR=target/panic-abort und CARGO_PROFILE_DEV_PANIC=abort verwenden; ein Manual-Fenster, F11 und ein Tick. Synchroner Marker, genau ein Panic-Failure/Report und tatsächliches ProcessExit-Endevent mit Unix-Signal SIGABRT prüfen; Unwind-Exit 101 genügt nicht. Build-only-Server Exit 0, Testserver erwartet Exit 1; keine zusätzlichen ProcessExit-Failures oder Reparaturticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.50 · Unerwarteter Exit ohne Panic · Code** — `tests/reports/process_exit.py` · Grafik **1+**. App ohne Panic unerwartet mit Erfolg und Fehler beenden → nach Pipe-Drain jeweils ProcessExit mit Status; Exit 0 ist nicht automatisch geordneter Shutdown. Kein zusätzlicher Panic-Failure. Zwei Manual-Starts, F9/je ein Tick, Exit 0 bzw. 17. Genau ein ProcessExit/Report, Failure vor Ended und vollständiger vorausgehender stderr-Sentinel.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.51 · Beschädigte Report-Marker · Code** — `tests/reports/invalid_marker.py` · Grafik **F0**. Defekte, unvollständige und fragmentierte stderr-Marker senden → ObservationError statt erfundenem Panic, gültige Marker entfernt und übrige stderr-Bytes erhalten; EOF korrekt abgeschlossen. Acht externe Varianten: gültiger geteilter/chunkierter Panic-Marker, JSON/Checksum/Base64, doppelter/fehlender Chunk, EOF-Fragmente und unerlaubter ProcessExit-Payload. Nicht-UTF-8-Restbytes bleiben erhalten; beschädigte Marker unterdrücken keinen echten ProcessExit.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.52 · Fehler vor Ready · Code** — `tests/reports/before_ready.py` · Grafik **F0**. Fünf externe Vor-Ready-Varianten: Panic-/Tracing-Marker, defekter Handshake/Marker und Exit 0/17. Tatsächlichen Peer-Start über eigene PID-/Mode-Datei belegen. invalid erhält invalid_ready samt defekter Ready-Zeile; übrige Varianten launch mit Vor-Ready-/konkretem Exitstatus-Bezug, Markerfälle zusätzlich ihren Nachrichtentext. Keine Ready-, Report-, Failure- oder Command-Annahme.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.53 · Langsamer Provider blockiert Spiel und Clients nicht · Code** — `tests/reports/slow_provider.py` · Grafik **1+F**. Provider am externen gh-Prozess kontrolliert aufhalten und Report auslösen → Spiel-Commands, Activity und andere Clients machen weiter Fortschritt; Provider-Ergebnis später zugeordnet. Nur begrenzter Lastfall, keine unbegrenzte Lastgarantie. Ein Manual-Fenster, F12 und insgesamt drei Ticks. Nur testeigenes gh blockiert; zwei weitere Ticks, App-Zähler und separate Settings-Abfrage vor Freigabe. Danach genau zugeordnetes Issue-Outcome des lokalen Fehlerwerkzeugs, kein Netzwerk.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.54 · Lokaler Provider und Konflikte · Code** — `tests/rust/test_local_provider.rs` · Grafik **R1**. Öffentliches report::submit gezielt verwenden → Created/Existing/Conflict, vollständiges Markdown, relative Referenz, sichere Pfade und kein Überschreiben fremden Inhalts; konkurrierende gleiche Veröffentlichung sauber behandeln. Explizit verdrahtetes Rust-Integrationstarget local_provider, eine echte Manual-Session ohne Ticks/Inputs. Sechs parallele Veröffentlichungen: einmal Created/fünfmal Existing, vollständiges Markdown, Konflikt-/Marker-/Symlink-Sentinels und intrinsisch ungültige Ausgabepfade.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.55 · GitHub-Vertrag und lokaler Fallback ohne Netzwerk · Code** — `tests/rust/test_github_provider.rs` · Grafik **R1+F**. Nur externes gh durch kontrolliertes Werkzeug ersetzen → CWD, stdin/Argumente, Pagination, exakter Marker und offene/geschlossene Duplikate stimmen; Search-/Publish-/Antwortfehler ergeben expliziten Fallback, bei lokalem Fehler beide Ursachen. Keine echten Issues. Rust-Target github_provider, eine echte Manual-Session ohne Ticks/Inputs. Neun kontrollierte gh-Szenarien mit zwei Seiten, PR-/Fence-/Marker-Decoys, offenen/geschlossenen Treffern, Search/Publish/Antwortfehlern und doppelter Fallback-Ursache. Ausschließlich externer gh-Ersatz; Prozessumgebung vor SDK-Threads gesetzt und nach Join restauriert.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.56 · Unveränderlicher Report-Snapshot · Code** — `tests/rust/test_report_snapshot.rs` · Grafik **R1**. Report über öffentliche API erzeugen, danach Commands und Zustand eines temporären Testrepositorys ändern → ursprünglicher Kontext und History bleiben unverändert; erlaubte Metadaten vorhanden, ausgeschlossene vertrauliche Kontextfelder fehlen. Projekt-Checkout nicht ändern. Rust-Target report_snapshot; eine echte Manual-Session ohne Ticks, ShiftLeft-Press/Release nur in Queue. Eigene App-/Asset-Kopie und Git-Fixture mit vorab aufgelöstem Lock; späterer Branch/Commit/History verändern alten Report nicht. Exakte erlaubte Kontextfelder und tatsächliche Werte für Target, Features, Argumente, Tickpolitik, Capabilities und SDK-Version gegen die Start-Fixture prüfen. Vorhandene optionale Toolchain-Werte mit unabhängigen Tool-Aufrufen vergleichen; erlaubtes null bleibt unavailable.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.57 · Titel, Markdown und Fehlersignatur · Code** — `tests/rust/test_report_signature.rs` · Grafik **R1**. Öffentliche Failure-/Report-Werte mit kontrollierten Nachrichten und Diagnosevarianten erzeugen → Titelbereinigung, Markdown-Escaping und versionierte Signatur vertragsgemäß; relevante Nachrichtenänderung unterscheidet Signatur, reine Diagnoseänderung nicht. Rust-Target report_signature; eine echte Manual-Session ohne Ticks/Inputs. Vier öffentliche Golden Vectors, Diagnoseänderung versus relevante Meldungsänderung, Projekt-/Adress-/Zeitnormalisierung, Titel/Unicode-Kürzung und sichere Markdown-Fences. Gesamtes Markdown in Abschnittsreihenfolge konsumieren: vollständige Nachricht/Backtrace, Origin sowie Application/Environment/Commands als JSON-Werte; jedes Fence geschlossen, kein Rest oder doppelter Abschnitt.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Shutdown — 4.58 bis 4.63

- [x] **4.58 · Direkter Shutdown und Vorbedingungen · Code** — `tests/shutdown/command.py` · Grafik **1**. shutdown bei offenem Command und aktiver Aufnahme versuchen, dann ausdrücklich auflösen/stoppen und erneut senden → Blockierung ohne Annahme bzw. danach terminale Completion und normaler Exit; keine neue Arbeit danach. Erwartetes Stoppen erzeugt keinen Spielfehler-Report. Eine Manual-Session, 120er-Warp bei 2/s früh stoppen, keine Inputs. Bei offenen Commands hat shutdown_commands_pending Vorrang vor aktiver Aufnahme; beide Rejections ohne ID/Activity. Erst nach ausdrücklichem Warp-/Recording-Stop einmal MAX-ID, normaler Abschluss, weitere Arbeit ohne Annahme abgelehnt.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.59 · Verwaltungs-Stopp löst aktive Arbeit auf · Code** — `tests/shutdown/session_stop.py` · Grafik **1**. Session mit Warp/Replay bzw. Recording verwaltungsseitig stoppen → Stop-Operationen und Footer abgeschlossen, relevante Outcomes erhalten, genau diese Session beendet; nicht bloß gewaltsam alle Prozesse töten. Eine Manual-Session, Replay-Warp 120 bei 2/s und verbotener g-/Ein-Tick-Tail, gleichzeitig Recording. App-eigener Counter-Spiegel belegt aktiven Warp ohne Annahme interner Pendings in Server-Activity. Replay Stopped, wirkungsvoller Warp-Stop, optional zusätzlicher Idle-Stop, dauerhafter Footer vor Shutdown, Server weiter nutzbar.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.60 · Stopp während Starting · Code** — `tests/shutdown/starting.py` · Grafik **F0**. Startvorgang kontrolliert vor Ready aufhalten und Session stoppen → Starting wird abgebrochen und Ressourcen eingesammelt; kein späteres Ready und kein Spielfehler-Report. Externer Prozess hält nach PID-/Gate-Datei tatsächlich vor Ready. Annahme vor Ready abgelehnt; gezielter Sessionstop, normales Ended, PID eingesammelt, nachträgliche Gate-Freigabe erzeugt kein Ready, erneuter Stop idempotent.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.61 · Serverende mit mehreren Sessions und Reports · Code** — `tests/shutdown/server_stop.py` · Grafik **2+F**. Server bei verschiedenen aktiven Arbeiten und noch ausstehendem Provider-Ergebnis stoppen → keine neue Annahme, Sessions unabhängig abgewickelt, Reports berücksichtigt und eindeutiges Gesamtergebnis; langsame Session hält andere nicht auf. Zwei Manual-Apps, A mit frühem Stop eines 120er-Warps/Recording, B F12/ein Tick/Recording und gehaltenem Fake-gh; maximal 121 Ticks. Nur verifizierte eigene A per SIGSTOP halten, 20s Serverfrist: B beendet unabhängig, neue Session/Commands abgelehnt, Report vor A-Freigabe zugeordnet, dann SIGCONT A und Gesamt-Exit 0.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.62 · Gemeinsame Shutdownfrist und erzwungenes Ende · Code** — `tests/shutdown/deadline.py` · Grafik **2+F**. Zwei Manual-Apps mit je 120er-Warp bei 2/s plus B F12/ein Tick, maximal 241 Ticks. Verifizierte eigene Spiele/Fake-gh halten; eine gemeinsame 12-Sekunden-Frist. Stop-Annahme durch monotone CLI-Aufrufklammer eingrenzen, beide Stopping bestätigen, A etwa bei Halbzeit freigeben und bis zur ursprünglichen Frist Ended beobachten, Stop erneut senden. Exit durch Poll-Intervall eingrenzen; Trennpunkt zwischen ursprünglicher spätester Frist und frühester Neustart-/serieller Folgefrist ableiten. Überlappende Messhypothesen nach geprüftem Cleanup Unvollständig/Skip statt Grün. B/Provider durch Produkt bereinigt, Exit 1, keine erfolgreiche Veröffentlichung oder sauberer B-Footer.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.63 · Signale · Code** — `tests/shutdown/signals.py` · Grafik **1+F**. Erstes Ctrl+C bzw. SIGTERM während aktiver Arbeit und zweites Ctrl+C während Shutdown prüfen → zuerst geordneter Ablauf, danach sofortige erzwungene Bereinigung mit passendem Fehlerstatus. SIGKILL nicht als geordneten Abschluss ausgeben. Zwei serielle Manual-Starts für erstes SIGINT/SIGTERM, jeweils 120er-Warp bei 2/s früh stoppen (maximal 240 Ticks). Danach externe Shutdown-Hold-Variante: zweites SIGINT erzwingt vor der 20s-Frist Exit 1 ohne erfundene Completion. SIGKILL nur auf eigenem leerem Server ohne Kinder: negativer Signalexit, kein geordneter Abschluss.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-yq5as6b4/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Zusätzliche öffentliche Rust-Verträge — 4.64 bis 4.69

Sechs explizite Cargo-Targets, je ein öffentlicher Integrationstest; kein Zugriff
auf private Session-Zustände. Nach dieser Gruppe waren 84 Fälle geschrieben: 74 Python und zehn öffentliche Rust-Tests.
Belege: `target/test-runs/rust-contract-checks-aoxY5f/summary.json`.
4.69 ist vollständig fensterlos geprüft. Die fünf R1/R2-Fälle sind kompiliert
und gesperrt, aber nicht grafisch ausgeführt; frische Freigabe fehlt. Reduzierte
Kontrollen mit externen Wire-Peers bestätigen Ownership, Drop/Recording, History,
Shutdown und Event-Überlauf, nicht den echten App-/Renderer-Ablauf. Alle sechs
Peers des finalen Kontrolllaufs beendet, keine Harness-Signale. 14 fensterlose
Regressionen bestanden; neun Rust-Grafikguards einschließlich 4.54–4.57 geprüft.

- [x] **4.64 · Pending-Ownership und einmaliger Empfang · Code** — `tests/rust/test_pending_ownership.rs` · Grafik **R2**. Zwei direkte Manual-Sessions mit den gemeinsamen versetzten Fensterpositionen, A zwei Ticks/B null, keine Inputs. Gleich nummerierte Pending verschiedener konkreter Outputtypen fremd verwenden → InvalidPending ohne Verlust am Eigentümer; Completion beziehungsweise Stopped korrekt typisiert genau einmal, zweite Entnahme InvalidPending. CLI kann diesen Rust-Ownership-Vertrag nicht nachweisen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.65 · Pending-Drop ist kein Abbruch · Code** — `tests/rust/test_pending_drop.rs` · Grafik **R1**. Eine Manual-Session, zwölf Ticks bei 2/s und Recording, keine Inputs. Nach unabhängig beobachtetem ersten Last-Lauf wirklich offenes Pending droppen; kein Empfang, Stop oder erneutes Einreichen während der restlichen Arbeit. App-Dateizähler erreicht zwölf, Shutdown bleibt während offener Arbeit ohne Annahme blockiert, History und einziger Recording-Warp erhalten den vollständigen natürlichen Abschluss samt Footer. Diese Blockierung belegt die Drop-Wirkung, nicht erneut allgemeine Shutdown-Vorbedingungen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.66 · Direkte Rust-Session terminal abschließen · Code** — `tests/rust/test_session_shutdown.rs` · Grafik **R1+**. Zwei serielle Manual-Sessions. Erste ohne Ticks erfolgreich herunterfahren → neue Operationen Ended, Capabilities/verbrauchte Pending-Metadaten weiter lesbar, kein zusätzliches Ended-Event, Prozessgruppe weg. Zweite mit Recording und offenem 120er-Warp bei 2/s nach beobachtetem Fortschritt droppen → hartes Produkt-Cleanup statt Warten auf natürliche Completion, eigene PID/Gruppe eingesammelt. Kein Harness-Signal und keine Footer-Garantie erfunden. Allgemeine Shutdown-Vorbedingungen bleiben 4.58; hier kein zweiter negativer Shutdown-Test.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.67 · History begrenzen, aktive Arbeit behalten · Code** — `tests/rust/test_history.rs` · Grafik **R1**. Eine Manual-Session, 120er-Warp bei 2/s offen halten; 55 eindeutig unterscheidbare gültige Hierarchie-Queries mit Tiefe 0–54, keine Inputs. History nach jeder Query höchstens 50 und zuletzt exakt Queries 5–54 in Annahmereihenfolge. Verdrängter Warp bleibt offen und empfangbar; ausdrücklicher Stop liefert Stopped, spätere Completion darf ihn nicht zurück in die History setzen. Report-Kontext übernimmt genau das begrenzte Fenster.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.68 · Event-Überlauf und reserviertes Endevent · Code** — `tests/rust/test_event_overflow.rs` · Grafik **R1**. Eine direkte Manual-Session, F9/ein angeforderter Tick, neue begrenzte Diagnose event-overflow: 300 tracing-Fehler. Vor bestätigtem Produkt-Cleanup keine Eventabholung, kein Server. Danach keine neue Annahme, 256 geordnete Tracing-Failures plus genau ein reserviertes finales Ended mit Kapazität 256 und positivem Verlustzähler; eigene Prozessgruppe eingesammelt. Kein exakter Gesamtverlust behauptet, da das Produkt den Produzenten abbrechen kann; Tick nur bekannt abgeschlossen oder Ended, keine erfundene Completion. Keine automatische Reportveröffentlichung.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.69 · Verlustfreie Handle-Kodierung · Code** — `tests/rust/test_handle_encoding.rs` · Grafik **R0**. 36 Kombinationen kleiner und großer u32-Index-/Generationwerte durch Handle-JSON und gemeinsame Command-Kodierung; kanonische zwei Integerfelder, exakter Roundtrip, öffentliche Bevy-Entity-Konversion und ungültiger Index-Sentinel ohne World. Überläufe, negative Werte und unbekannte Felder abgelehnt. Keine normale Entity-Suche, Session oder App.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Gezielte Grenzfälle statt pauschaler interner Tests — 4.70 bis 4.78

- [x] **4.70 · Abgleich zusätzlicher Risiken · Umsetzung** — `docs/testing-plan.md` · Grafik **0**. Vor den folgenden Kandidaten prüfen, welchen konkreten Fehler der öffentliche Test nicht zuverlässig erzeugen kann. Nur dann den benannten kleinen Rust-Nachweis am zuständigen Owner ergänzen. Bei ausreichender öffentlicher Abdeckung stattdessen verantwortliche ID und Beleg eintragen und Kandidat ausdrücklich streichen; kein Dummytest/Skip als Erfolg. Offene Entscheidung blockiert uneingeschränkte lokale Abnahme.
  - [x] **Geprüft** — Quellenabgleich dokumentiert und acht interne Modultest-Grenzen anschließend vom Nutzer ausdrücklich bestätigt. Alle acht Fälle einzeln umgesetzt, danach gemeinsam fensterlos bestanden. Nach diesem Block 92 Fälle: 74 Python, zehn öffentliche Rust-Integrationstests, acht interne Rust-Modultests. Ursprünglicher Abgleich: `target/test-runs/risk-seam-review-VFphe0/summary.json`; Umsetzung/Endbelege: `target/test-runs/internal-contract-checks-XZyRHn/summary.json`.

**Ergebnis des Quellenabgleichs:** Keinen der acht Kandidaten allein wegen der
bisherigen öffentlichen Tests streichen. Die Begründung ist jeweils fehlende
zuverlässige Steuerbarkeit des konkreten Fehlers, nicht deren noch offene
grafische Ausführung. Auch die falschen Warteassertionen in 4.36/4.37 begründen
keinen zusätzlichen internen Test: Bei 4.79 auf unabhängigen App-Fortschritt
korrigiert, aber weiterhin ohne neuen grafischen Ausführungsbeleg.
Maßgeblich ist das [API-Ziel](api/target.md), nicht ein nachgebautes Istverhalten.

| Kandidat | Öffentlicher Owner und zusätzliches, dort nicht zuverlässig erzeugbares Risiko | Bestätigte interne Testgrenze |
| --- | --- | --- |
| **4.71** | 3.12 und 4.20–4.26 prüfen virtuelle Inputs, nicht das Eindringen nativer Bevy-Nachrichten. Echte OS-Eingaben sind ausdrücklich verboten; sie könnten auch keinen reproduzierbaren Fokusverlust zwischen Gate und Verarbeitung garantieren. | [`input.rs`](../src/session/input.rs) und [`input/keyboard.rs`](../src/session/input/keyboard.rs): native Bevy-Nachrichten vor dem echten Gate einspeisen, virtuelle Commands über ihren echten Flush ausführen und denselben Tick-Eingabepfad prüfen. Native Keyboard/Maus/Touch/IME werden ausgeschlossen, virtuelle Holds bleiben bei Fokusverlust erhalten; unveränderte native Verarbeitung ohne Installation als Kontrolle. Keine OS-/Winit- oder Renderer-Abnahme daraus. |
| **4.72** | 4.27/4.28 prüfen Texteingabe und Fokusvoraussetzungen. Zwischen bestätigter Annahme und Flush gibt es keinen öffentlichen Fokus-/Despawn-Command. Ein erst im Tick verarbeitetes UI-Klicken erzwingt diesen Zwischenzustand nicht. | [`input/text.rs`](../src/session/input/text.rs): nach echtem `State::input` Fokus beziehungsweise Ziel im isolierten Bevy World ändern, dann echtes `flush` und Bevy-Editverarbeitung. Angenommener Text bleibt beim alten lebenden Ziel; Despawn oder Verlust von EditableText leitet ihn nicht um. Ersetzt nicht den echten UI-Nachweis 4.27/4.28. |
| **4.73** | 4.29 prüft vor dem ersten Tick, 3.13/4.30 echte PNGs. Sie halten weder GPU-Readback gezielt über seine Frist noch liefern sie einen alten Readback nach einem Requestwechsel. Verdeckung ist kein reproduzierbarer oder zugesicherter Fehlerhebel. | [`screenshot/capture.rs`](../src/session/screenshot/capture.rs): tatsächliches `poll` mit kontrollierten Surface-Ergebnissen, Readback-Nachrichten und abgelaufener Aktivierungszeit. Fehlende Surface, Bild bereits im Kanal bei Fristablauf und späte Lieferung dürfen weder schreiben noch einen Folgeauftrag abschließen. Kein echter GPU-/Surface-Nachweis und keine neue öffentliche Uhr-/Capture-API. |
| **4.74** | 4.31 legt gefährliche Links vor dem Command an. Der öffentliche Pending-Zeitpunkt bestätigt keine bereits abgeschlossene Zielprüfung; ein gleichzeitiger Dateitausch würde die entscheidende Lücke nur zufällig treffen. | [`screenshot/destination.rs`](../src/session/screenshot/destination.rs): tatsächliche Prüfung und tatsächliches Schreiben mit deterministischem Austausch eigener Pfadkomponenten dazwischen. Reale Directory-Capabilities und Sentinels, keine nachgebildete Pfadprüfung; kein Schreiben oder Ersetzen im eigenen Outside-Sentinel. Legitime In-Root-Links nicht pauschal verbieten. |
| **4.75** | 4.33 und 4.35 prüfen statische Pfade, Symlinks und Dateiformen. Sie steuern keinen Austausch während des komponentenweisen Öffnens. Writer und Loader besitzen zwei getrennte tatsächliche Dateizugänge. | [`recording/file.rs`](../src/session/recording/file.rs): echte `Recording::start`-/`open_read`-Zugänge an den Öffnungsgrenzen mit eigenem Pfadtausch kontrollieren. Kein Folgen von Links oder Überschreiben; Loader akzeptiert nur reguläre Dateien und hängt nicht an einer ausgetauschten FIFO. Beide Zugänge prüfen, nicht nur gemeinsame lexikalische Validierung. |
| **4.76** | 4.8 prüft eindeutige Präfixe. 128-Bit-ID-Kollisionen und passende Präfixkollisionen sind nicht verlässlich zufällig herstellbar; Massenerzeugung von Sessions ist kein angemessener Ersatz. | [`server/mod.rs`](../src/server/mod.rs): bestehende Grenze `Inner::create_using` mit kontrollierten ID-Kandidaten, eigenen vorhandenen Roots/Links und Einträgen; tatsächliches `select` muss mehrdeutige Präfixe ablehnen. Fremde Bytes bleiben erhalten. Kein öffentlicher RNG-/ID-Schalter und keine echten Spielstarts. |
| **4.77** | 4.9/4.64 prüfen Korrelation und Ownership normaler IDs; 4.58 den MAX-ID-Shutdown. Keiner erreicht den letzten normalen Wert durch eine begrenzte öffentliche Folge. | [`session/coordinator.rs`](../src/session/coordinator.rs): ID-Startwert nur testlokal an die Grenze setzen und denselben Annahme-/Dispatchpfad einschließlich interner Replay-Vergabe prüfen. Erschöpfung erzeugt keine Annahme, History oder Wiederverwendung, verliert keine offene Arbeit und lässt den reservierten Shutdown nach deren Abschluss zu. Kein isolierter Ersatz-Allocator als alleiniger Nachweis. |
| **4.78** | 4.33 verantwortet normale Barrieren, 4.34 I/O-Fehler, 4.36/4.59 Stop aktiver Pläne; deren Ausführungsstand bleibt jeweils separat ausgewiesen. Kurze Starting-/Stopping- und Replay-Preparing-Phasen sowie deren konkurrierende Fehler lassen sich darüber nicht gezielt festhalten. | [`recording.rs`](../src/session/recording.rs), [`recording/worker.rs`](../src/session/recording/worker.rs), [`replay.rs`](../src/session/replay.rs) und [`replay/loader.rs`](../src/session/replay/loader.rs), integriert mit dem Koordinator: echte Dateiarbeit per testlokaler Handshake anhalten/freigeben. Annahme gegenüber Dispatch, beide Recording-Barrieren, Stop während Laden, gleichzeitige Stops und Reihenfolge von Fehler/Stop prüfen; keine spätere Freigabe abgebrochener Plan-Commands. Bereits öffentlich verantwortete Normalabläufe nicht erneut testen. |

**Vom Nutzer bestätigter Freigabeumfang:** Acht gezielte Rust-Testdateien an den unten
bereits benannten Ownern, je ein Test mit nötigen Varianten. Nur isolierte
Bevy-World-/Datei-/Koordinatornachweise; falls für Transportbeobachtung nötig,
ausschließlich eigene externe Protokollpartner ohne Spiel. Keine test_app,
Renderer, OS-Eingaben, Netzwerkveröffentlichung oder grafische Ausweichabnahme.
Vorhandene interne Grenzen zuerst verwenden; notwendige zusätzliche Gates nur
testlokal, instanzgebunden und mit begrenzten Handshakes statt Scheduler-Sleeps
oder globalen Fehlerflags. Keine neue öffentliche SDK-API oder fachliche
Produktkorrektur. Reale Produktabweichungen bleiben normale rote Tests.
Die Fälle wurden einzeln am jeweiligen Owner ergänzt. 4.71–4.73 verwenden
Software-Worlds beziehungsweise vorhandene Input-/Readback-Grenzen; 4.74/4.76
vorhandene Prüf-/Schreib- beziehungsweise ID-Kandidaten-Grenzen. 4.75 ergänzt
zwei aufruflokale cfg(test)-Checkpoints vor tatsächlichen Dateiöffnungen.
4.77/4.78 verwenden instanzgebundene Koordinator-Kontrollen, echte I/O-Worker und
einen eigenen externen Wire-Partner. Ihre Klasse ist daher **F0**, kein R1-
Spielnachweis. Der Partner liefert nur literale Antworten und diagnostische
Pipe-Fences, keine simulierte World.

Gemeinsamer Lauf mit cli,ui,screenshot: acht Tests ohne Skip bestanden;
Default separat vier Tests. 14 fensterlose Python-Regressionen und 4.69 bestanden;
neun Rust-Grafiksperren und zehn öffentliche Targets geprüft. Keine neue SDK-API
oder fachliche Produktkorrektur. Grafiknachweise bleiben offen; die spätere
Einbindung ist unter 4.79 dokumentiert.
Frühere Entwicklungsfehler sind erhalten: falsche Bevy-/Output-Form-Annahmen und
ein 4.78-Harness-Timeout durch einen zu lange gehaltenen diagnostischen Pipe-Sender.
Lebensdauer/Join korrigiert; final keine Harness-Eskalation.

- [x] **4.71 · Native Eingaben und Fokusverlust · Code** — `src/session/input/tests/test_native_input.rs` · Grafik **R0**. Bevy-seitig native Tastatur/Maus/Touch/IME und Fokusverlust kontrolliert einspeisen → kontrollierte Simulation erhält nur virtuelle Geräte, gehaltene virtuelle Tasten bleiben erhalten; ohne Session-Plugin native Bedienung unverändert. Keine echten Nutzer-Eingaben automatisieren.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.72 · Textziel bei Fokuswechsel oder Despawn · Code** — `src/session/input/text/tests/test_target_binding.rs` · Grafik **R0**. Zwischen Annahme und Tick Fokus/Entity gezielt ändern → Text bleibt an angenommenes Ziel gebunden, nie Umleitung ins neue Feld; verschwundenes/nicht mehr editierbares Ziel bekommt keinen Edit.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.73 · Surface, Timeout und verspäteter GPU-Readback · Code** — `src/session/screenshot/capture/tests/test_readback_lifecycle.rs` · Grafik **R0**. Fehlende Capture-Surface und Readback nach Frist/Requestwechsel kontrolliert erzeugen → spezifische Ablehnung ohne Schreiben bzw. screenshot_failed; späte Antwort schreibt nichts und beendet keinen anderen Request, keine versteckten Ticks.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.74 · Pfadaustausch beim Screenshot-Schreiben · Code** — `src/session/screenshot/destination/tests/test_path_replacement.rs` · Grafik **R0**. Testeigene Pfadkomponente zwischen Prüfung und Schreiben austauschen → kein Ausbruch aus Root und kein Ersetzen eines fremden Ziels; begründeter Zusatz zu statischen Pfaden.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.75 · Pfadaustausch bei Recording und Replay · Code** — `src/session/recording/file/tests/test_path_replacement.rs` · Grafik **R0**. Testeigene Komponenten zwischen Prüfung und Anlegen/Öffnen austauschen → kein Folgen von Symlinks, Recording überschreibt nichts, Loader akzeptiert nur reguläre Datei unter Root. Beide tatsächlichen Dateizugänge berücksichtigen.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.76 · ID-Kollision und ambiges Präfix · Code** — `src/server/tests/test_session_identity.rs` · Grafik **R0**. Kontrollierte ID-Kandidaten und bestehende Roots vorgeben → keine Übernahme/Überschreibung einer fremden Session, Kollision sicher aufgelöst; mehrdeutiges Präfix abgelehnt. Zufallsbasierter Live-Test wäre kein zuverlässiger Nachweis.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.77 · Request-ID-Erschöpfung · Code** — `src/session/coordinator/tests/test_request_id_exhaustion.rs` · Grafik **F0**. Normalen ID-Raum kontrolliert an seine Grenze bringen → keine Wiederverwendung oder fiktive Annahme, laufende Arbeit bleibt; reservierter Shutdown weiterhin möglich. Nicht Milliarden Live-Commands senden.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

- [x] **4.78 · Recording-/Replay-Übergangsrennen · Code** — `src/session/tests/test_transition_races.rs` · Grafik **F0**. Dateiarbeit in Starting/Stopping bzw. Replay-Preparing kontrolliert anhalten → Barrieren, Stop während Laden, gleichzeitige Stops und Fehlerpriorität vertragsgemäß; kein späteres Freigeben abgebrochener Plan-Commands. Nur nicht bereits öffentlich belegte Übergänge.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

#### Integration, externer Nachweis und Reportabschluss — 4.79 bis 4.82

- [x] **4.79 · Relevante Feature-Builds · Umsetzung** — `Cargo.toml; test_app/Cargo.toml; tests/acceptance.py; tests/_builds.py; .github/workflows/contracts.yml` · Grafik **0**. Default-/CLI-/UI-/Screenshot-Kombinationen und Referenz-App mit ihren ausdrücklich verwendeten Varianten mit Standardwerkzeugen bauen → öffentliche Rust-Testtargets tatsächlich eingebunden und erforderliche Prüfungen im gemeinsamen Einstieg ausgewählt, keine stillen Feature-Abhängigkeiten; Buildfehler separat sichtbar. Teilumfänge bauen nur ausgewählte Abhängigkeiten; vollständige lokale Auswahl verlangt die Feature-Matrix automatisch, CI fordert sie bei --non-graphical ausdrücklich mit --feature-matrix an. Der externe Live-Transport bleibt seiner Auswahl vorbehalten. Keine Behauptung einer grafischen Abnahme durch cargo check.
  - [x] **Historischer Build-/Integrationsbeleg** — Gemeinsamer Einstieg integriert: 75 per unittest entdeckte Python-Fälle, zehn explizite Cargo-Integrationstargets und acht echte interne Tests; separater typisierter Kontrolllauf 3.18 bleibt dieselbe Resource-Testimplementierung. Lokaler Sollumfang 92 Fälle plus Kontrolle, GitHub separat. Zwölf Build-Schritte bestanden: sieben Feature-Kombinationen, CLI, normale/Automation-App, reale Rust-Binaries samt Testlisten und SDK-Live-Transport. Workspace-Alltargets-Check bestanden. Fensterloser gemeinsamer Lauf: 23 bestanden, bekannter 4.40-UTF8-Fehler normal rot, Exit 1. Build-only Exit 2, fehlende Grafik-/GitHub-Freigabe ohne Starts blockiert, gezielter Buildfehler Exit 1; keine Dummy-/Skip-Erfolge. CI verwendet denselben fensterlosen Einstieg und lädt auch Fehlerbelege hoch. Damals blieb CI wegen 4.40 und bestehendem Clippy-int_plus_one in src/session/coordinator.rs:374 rot. 4.40 ist später gezielt behoben (siehe dessen aktuellen Eintrag); Clippy und gesonderte CI-Auswertung des unvollständigen Teilumfang-Exits waren damals offen; beide sind im full-stop-Nachweis korrigiert und lokal geprüft. Belege: target/test-runs/acceptance-integration-checks-gVvC9e/summary.json und target/test-runs/acceptance-74jubrj4/summary.json.
  Historische Cache-/Auswahländerung: 12 getrennte Offline-Runnerkontrollen und alle 15 fensterlosen Python-Regressionen bestanden; 75 Python-Produkttests und 94 Inventareinträge ohne Fehler. --list zeigt die tatsächlich gewählten Cargo-Aufrufe: fensterlos zwei statt zwölf, voller lokaler Build elf ohne externen Driver. Kompilierte Testlisten werden nur für gewählte Rust-Targets verlangt; interne Auswahl prüft weiterhin genau acht echte Tests. Kein neuer Cargo-/App-/Grafikbuild, kein Rust-Testlauf und keine Artefaktbereinigung. Die Runnerkontrollen sind Infrastruktur, keine zusätzlichen Produkttests. Belege: target/test-runs/build-cache-update-pid9xnxe/.

- [ ] **4.80 · Vollständiger lokaler Gesamtlauf · Umsetzung** — `tests/acceptance.py` · Grafik **2+**. Nach abgeschlossener Abdeckung denselben gesamten lokalen Sollumfang auf einem dokumentierten finalen Stand seriell ausführen → nur bei vollständigen erfolgreichen Nachweisen Bestanden; andernfalls Fehlgeschlagen/Unvollständig. Kein zusätzlicher monolithischer Spieltest.
  - [ ] **Geprüft** — Fünf ausdrücklich unter full-stop gestartete Gesamtläufe mit vollständiger lokaler Auswahl. Erster Fixture-Fehler, falsches Kontroll-Abbruchsignal, falsches Pipeline-Reihenfolge-Oracle und Pace-Messvoraussetzung nachvollziehbar korrigiert und gezielt belegt. Im vierten Lauf 91 Einträge einschließlich Kontrolle bestanden, bei Pace Unvollständig; die drei Tick-Fälle danach auf korrigiertem Stand bestanden. Letzter vollständiger Versuch stoppte mit 43 bestandenen Einträgen an 3.12 wegen window surface=false. Keine verbliebenen Produktprozesse und keine Harness-Eskalation. Kein grüner Gesamtlauf auf finalem Stand; Offen bis zur geklärten Render-Voraussetzung. Alle Originalversuche und Zuordnung: `target/test-runs/full-stop/checkpoint-audit.json`.

- [x] **4.81 · Echter GitHub-Provider gegen Testrepository · Code** — `tests/providers/github_live.py` · Grafik **1+X**. Nach gesonderter Freigabe echtes Issue veröffentlichen und erneute gleiche Veröffentlichung erkennen → Created/Existing mit überprüfbarer Referenz; ausdrücklich vereinbarte Testdaten und Bereinigung. Ohne Freigabe externer Umfang offen, keine uneingeschränkte Alle-Features-Aussage. Eine echte Manual-Session ohne Ticks/Inputs über einen öffentlichen SDK-Transport, explizit schließen, dann Run-UUID-Report zweimal submitten: Created/Existing, exakt eine überprüfte Issue-Referenz/Markdown, kein Fallback. Guards verlangen benanntes github.com-Testrepository mit Description-Marker, Grafikfreigabe und erlaubtes Schließen. Cleanup prüft Run-Body/Signatur/URL, schließt nur eigene Issues, löscht nichts; bei defekter Deduplizierung höchstens zwei eigene Issues.
  - [ ] **Erfolgreich ausgeführt** — Noch kein echter GitHub-Lauf. gh ist angemeldet; ein benanntes github.com-Testrepository mit Description-Marker `woodpecker-acceptance-test-repository` fehlt. Keine Issues erstellt oder geschlossen. Lokaler Transport-/Buildbeleg bleibt vom externen Nachweis getrennt.

- [x] **4.82 · Report-Owner vollständig abschließen · Code** — `src/server/tests/test_report_completion.rs` · Grafik **R0**. Ergänzung zu 3.16/3.17 und 4.47–4.53: Öffentliche Activity kann nach terminalem Lifecycle noch wachsen; weder Ended noch eine ruhige Poll-Phase beweist Providerabschluss. Den tatsächlichen Report-Worker instanzlokal am Submit halten, drei unterschiedliche Jobs einreichen, Lifecycle beenden und denselben privaten `complete`-/`finish`-/Join-Pfad wie der Session-Owner ausführen. Positiver Checkpoint vor Join: done bleibt false und Server kann nicht abschließen. Nach Freigabe alle drei korrelierten Outcomes exakt zählen; normale Submission, unterbrochener Provider und Providerpanic getrennt prüfen. Testdaten ersetzen nur den Provideraufruf, nicht Queue, Catch, Activity-Publikation, Join oder Serverabschluss. Keine Session, App, Renderer, externe API oder globale Fehlerflags.
  - [x] **Erfolgreich ausgeführt** — Im full-stop-Abnahmelauf beziehungsweise gezielten Nachweis auf gehärtetem Teststand bestanden. Beleg `target/test-runs/acceptance-ig4cf4ib/summary.json`; Zuordnung und ursprüngliche Fehler bleiben in `target/test-runs/full-stop/checkpoint-audit.json`. Kein Ersatz für den noch offenen gemeinsamen Abschluss 4.80.

### 5. Struktur und Dokumentation abschließend bereinigen

Dokumentation schon bei jeder Implementierung aktuell halten; dies ist der
abschließende Abgleich, kein Grund, bis dahin falsche Ausführungsanweisungen stehenzulassen.

- [ ] **5.1** `tests/` und `Cargo.toml`: unnötige Abstraktionen, doppelte Assertions,
  unbenutzte Fehlerprogramme, tote Targets und verstreute Konfiguration entfernen.
  - [x] Konfigurations-Teilaufgabe: `load_config`/`write_config` in `tests/_support.py`
    laden frische TOML-Daten und prüfen den Roundtrip vor dem Schreiben. Alle 22
    konfigurationserzeugenden Tests ändern Felder statt Text; keine verborgenen
    Starts/Inputs/Ticks, keine neue Dependency. 55 Varianten semantisch gleich
    zum vorherigen Stand, umformatierte Basis und Sonderwerte geprüft; acht
    fensterlose Regressionen bestanden, 28 Grafikfreigabesperren geprüft.
    Belege `target/test-runs/config-maintenance-3zhqM8/`. Kein grafischer Neulauf;
    bisherige grafische Erfolgsbelege bleiben historische Läufe vor der Umstellung.
- [ ] **5.2** `tests/README.md`: kurzen ausführbaren Einstieg mit Auswahl, Abhängigkeiten,
  Grafikfreigabe, lokalem/externem Umfang und Gesamturteilen abschließen.
- [ ] **5.3** `README.md`, `test_app/README.md` und `docs/api/`: veraltete Testpfade
  und historische Erfolgsaussagen berichtigen bzw. als historisch kennzeichnen;
  fachliche Vertragswidersprüche auflösen, keine zweite Arbeitsliste anlegen.
- [ ] **5.4** Diese Liste gegen tatsächlich eingebundene Tests, Zuständigkeiten und
  frische Belege prüfen; Links und dokumentierte Aufrufe validieren. Ein grüner
  Pilot oder Build ersetzt weiterhin keine Gesamtabnahme.

### 6. Anwender-Beispiele — erst nach der lokalen Gesamtabnahme

Diese Beispiele sind nicht Teil eines zweiten vollständigen Feature-Testbestands.
Jedes zeigt einen für Spielentwickler verständlichen Ablauf über öffentliche
Woodpecker-Zugänge, ohne `tests/_support.py` zu importieren.

- [ ] **6.1** `examples/game_tests/movement.py`: Eingabe, explizite Ticks und
  erwartete Bewegung der Signalstation zeigen.
- [ ] **6.2** `examples/game_tests/text.py`: Feld fokussieren, Text eingeben,
  ausdrücklich ticken und sichtbare Feldwirkung prüfen.
- [ ] **6.3** `examples/game_tests/replay.py`: kleinen Spielablauf aufnehmen
  und in frischer Session mit gleicher Ausgangslage wiedergeben.
- [ ] **6.4** Einstieg in `examples/` dokumentieren und jeden Beispieltest wirklich
  ausführen; erforderliche gemeinsame technische Hilfe klein und öffentlich
  nachvollziehbar halten. Grafische Läufe auch hier nur nach frischer Freigabe.

Der Arbeitsstand dieser Suite wird nur hier fortgeschrieben. Historische
Prüfberichte in `docs/api/` und Ergebnisse der entfernten Suite zählen nicht als
Nachweise des Neuaufbaus.

### 7. Fehlersensitivität durch Mutationsprüfungen — zuletzt

Erst nach Abnahmesuite, Bereinigung und Anwender-Beispielen gezielte Produktfehler
einbringen und prüfen, ob die bestehenden Tests sie erkennen. Das geht über
absichtlich falsche Erwartungen hinaus; eine Erkennung aller denkbaren
Produktfehler wird damit nicht behauptet.

- [ ] **7.1** Für zentrale Verträge risikobasiert Mutationen auswählen, etwa
  ausgelassene Inspect-Einträge, ignorierte Release oder falsche Request-Zuordnung.
  Bestehende Tests verwenden, keinen zweiten Szenariokatalog aufbauen.
- [ ] **7.2** Mutationen ausschließlich in testeigenen isolierten Codekopien oder
  temporären Worktrees ausführen; Projekt-Checkout und persönliche Settings
  unverändert lassen. Grafische Läufe und Wiederholungen separat freigeben.
- [ ] **7.3** Pro Mutation den zuständigen Test, erwarteten fachlichen Fehlschlag,
  tatsächlichen Ausgang und Cleanup festhalten. Setup-/Buildfehler zählen nicht
  als Erkennung; überlebende und verhaltensgleiche Mutationen getrennt bewerten.
