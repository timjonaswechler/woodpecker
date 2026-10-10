# Anforderungen an die gemeinsame Bevy-Test-App

Die App stellt eine kleine, deterministische Bevy-Welt bereit, an der Woodpecker
getestet wird. Eine gemeinsame App läuft mit `--mode rendered` oder
`--mode headless`. Beide Modi verwenden dieselben Regeln und beobachtbaren
Zustände. Die App wird noch separat entwickelt; die Beispieldateien starten
keine vorhandene App.

## Was die App selbst implementieren muss

| Bestandteil | Konkrete Aufgabe der App | Nachweis der Suite |
| --- | --- | --- |
| Session-Anbindung | `woodpecker::session::Plugin` einbinden; die App bleibt bis zum verwalteten Shutdown am Leben. | Start, mehrere Sessions, Disconnect, Shutdown und Prozessbereinigung |
| Tick-Zustand | Eine lesbare Resource mit `ticks` und echter `process_id` registrieren. Ein ausgeführtes Update erhöht `ticks` genau um eins. | Warp, Pace, Stop, Recording und Replay |
| Inspect-Welt | Die Resource und einen eindeutig benannten Entity mit lesbarem `Name` anbieten. | Resource-/Entity-Projektionen und Filter |
| Tastatur | Tatsächliche Bevy-Tastenereignisse beobachten; gehaltene Tasten und Press-/Release-Zähler aktualisieren. | Input wird erst beim Tick wirksam, Halten wiederholt keinen Press |
| Pointer | Tatsächliche Pointer-/Mausereignisse beobachten; Position, Buttons und Scroll-Deltas abbilden. | Bewegung, Buttons, Scroll und keine unbeabsichtigte Wiederholung |
| Texteingabe | Ein echtes, fokussiertes Bevy-`EditableText` bereitstellen und dessen verarbeiteten Inhalt beobachten. | Unicode, Eingabegrenze und echte Textverarbeitung |
| Fehlerauslösung | Ein neuer Press von `b` löst im Update genau einen Tracing-Error aus und erhöht `failures`. | Automatischer Report und Persistenz nach Session-Ende |
| Render-Szene | Ein festes Bild mit vorab bekannten Abmessungen und Pixeln erzeugen. | Screenshot-Inhalt, Overwrite und keine zusätzlichen Simulationsticks |

Die App implementiert keinen eigenen Server, JSON-Parser, REPL, Recorder oder
Replay-Player. Diese Aufgaben übernimmt Woodpecker. Die Test-App führt selbst
keine Assertions aus. Sie liefert die tatsächliche Wirkung ihrer Bevy-Systeme,
die die Suite von außen prüft.

## Start und die beiden Modi

Im normalen Betrieb muss Folgendes gelten:

- Bei Ready ist die Zustandsresource vorhanden und `ticks` ist `0`.
- Ohne ausdrücklichen Warp bleibt die Simulation stehen. Inspect und Screenshot
  dürfen den beobachtbaren Zustand nicht verändern.
- Nach genau zehn ausgeführten Ticks ist `ticks` genau `10`, unabhängig von Pace.
- Jede Session ist ein eigener Prozess mit frischem Zustand. Kein Zustand oder
  gedrückter Input wird aus einer früheren Session übernommen.
- Nach Shutdown endet der Prozess. Ein Client-Disconnect allein beendet ihn nicht.
- Physische Eingaben verändern die kontrollierte Simulation nicht. Dafür wird die
  Woodpecker-Integration verwendet, statt eine zweite Eingabesperre zu bauen.

Rendered besitzt ein echtes Fenster und einen Renderer. Headless besitzt keine
native Fensteroberfläche und keinen Renderer. Für Keyboard, Pointer und Text
braucht aber auch Headless genau einen logischen `Window`-Entity mit
`PrimaryWindow`: Woodpecker validiert Input gegen diesen Entity.
Das logische Fenster soll wie das Rendered-Fenster 640 × 360 logische Pixel haben.
Die Ressourcen und Systeme für Input und Textverarbeitung müssen in beiden Modi
vorhanden sein.

`woodpecker/ui` wird für Texteingabe benötigt. Der Screenshot-Pfad benötigt
`woodpecker/screenshot` und im Rendered-Modus einen Renderer. Das kann dieselbe
Binary mit unterschiedlicher Plugin-Konfiguration sein. Die Laufzeitmodi sind
keine zwei unterschiedlichen Szenen.

## Die beobachtbare Resource

Der Name ist frei wählbar, beispielsweise `test_app::TestState`. Der vollständige
Reflection-Type-Path steht in `app.json` unter `state_type`. Der Typ muss als
Resource reflektierbar sein, mit `#[reflect(Resource)]` und `register_type`.
Die Resource muss vor dem ersten angeforderten Tick lesbar sein.

Eine frische Session liefert diese Struktur. `process_id` enthält ihre echte PID:

```json
{
  "ticks": 0,
  "process_id": 1234,
  "keyboard": {"pressed": [], "press_count": 0, "release_count": 0},
  "pointer": {
    "position": [0, 0],
    "pressed": [],
    "press_count": 0,
    "release_count": 0,
    "scroll": [0, 0]
  },
  "text": "",
  "failures": 0
}
```

`ticks` zählt ausgeführte Updates, nicht angeforderte Ticks. `pressed` enthält
normalisierte Tasten-/Button-Tokens; die aktuellen Szenarien verwenden `a`, `b`
und `left`. Die Pointer-Buttons `right` und `middle` sollen über denselben
Bevy-Eingabepfad beobachtet werden. Zähler sind kumulativ über die Session.
`scroll` enthält die kumulierten Scroll-Line-Deltas.

Die Resource wird aus tatsächlich verarbeiteten Bevy-Zuständen und Ereignissen
gebildet. Sie darf nicht direkt aus Woodpecker-Command-Argumenten befüllt werden.
Text wird aus dem echten Textfeld gelesen. Ein Update ohne neue Eingabe erhöht
nur `ticks`; Halten verändert nicht erneut den Press-Zähler.
Zusätzliche Felder sind möglich, müssen aber ebenfalls im Stillstand stabil sein.

## Konkrete Eingabeabläufe

Diese Beispiele beschreiben die aktuellen Verträge, nicht neue App-Commands:

| Ablauf nach einem Initialisierungstick | Erwartete Wirkung |
| --- | --- |
| `press a`, dann Inspect ohne Tick | Zustand unverändert |
| Anschließend ein Tick | `a` in `keyboard.pressed`, `press_count` um eins erhöht |
| Zwei weitere Ticks ohne neue Eingabe | `a` weiter gehalten, kein weiterer Press |
| `release a`, anschließend ein Tick | `a` nicht mehr gehalten, `release_count` um eins erhöht |
| Pointer nach `[20, 20]`, um `[5, -3]` bewegen, links drücken, um `[-2, 3]` scrollen; danach ein Tick | Position `[25, 17]`, links gehalten, ein Press, entsprechende kumulierte Scroll-Deltas |
| Text `Grüße 日本語 🐦` eingeben, anschließend ein Tick | Text wurde unverändert an den Inhalt des fokussierten Feldes angefügt |
| `b` neu drücken, anschließend ein Tick | `failures` um eins erhöht und ein Tracing-Error mit `failure_message` |

Ungültige Eingaben weist Woodpecker zurück. Die App muss ihren beobachtbaren
Zustand dann unverändert lassen. Insbesondere bleiben doppelte Presses,
Release ohne Press, negative Pointer-Positionen und zu großer Text wirkungslos.
Die Textsuite prüft 16.384 UTF-8-Bytes als erlaubte Grenze, beispielsweise
8.192-mal `ä`, und lehnt die nächste Eingabe darüber ab. Die App darf keine
kleinere Textlängengrenze oder einen Filter einstellen, der diese Eingaben kürzt.

Für das Textfeld gilt: Spätestens nach einem ausdrücklichen Initialisierungstick
existiert ein lebender `EditableText`-Entity und `InputFocus` verweist auf ihn.
Die normalen Bevy-Textedit-Systeme müssen die eingereichten Edits verarbeiten.
Die Zustandsresource liest den Inhalt nach deren Verarbeitung, damit die Änderung
nach diesem einen Tick sichtbar ist. Das gilt auch im Headless-Modus.

Für den normalen Report-Vertrag lautet `failure_message` im Beispiel
`suite controlled failure`. Ein gehaltener `b` löst nicht bei jedem Tick erneut
einen Fehler aus. Ohne diese Eingabe erzeugt der Normalbetrieb keine Testfehler.
Den `LogPlugin` mit `woodpecker::session::tracing_error_layer` anbinden; die
Launch-Konfiguration aktiviert `tracing_errors` und den lokalen Report-Provider.

## Szene, Entities und Bilder

Die aktuelle Inspect-Suite benötigt nach einem Initialisierungstick genau einen
Entity namens `suite-target`. Dieser besitzt einen lesbaren Bevy-`Name` und kann
über Summary, Component-Namen, Werte und Hierarchie abgefragt werden.
Andere Entities, etwa Fenster, Kameras und Textfelder, sind erlaubt.

Als gemeinsame Szene eignet sich ein grünes Rechteck im Zentrum auf schwarzem
Hintergrund und ein leeres Textfeld außerhalb der Sollpixel. Ein benannter
Child-Entity macht echte Hierarchieabfragen möglich. Die aktuellen Assertions
prüfen bisher den Hierarchie-Root; Child-Inhalt und zusätzliche Spielwirkungen
würden wir anschließend als fachliche Verträge ergänzen.

Die Beispielkonfiguration erwartet im Rendered-Modus:

- PNG mit 640 × 360 tatsächlichen Bildpixeln.
- Pixel `[320, 180]` ist `[0, 255, 0]`.
- Pixel `[10, 10]` ist `[0, 0, 0]`.

Logische Fenstergröße und Screenshot-Auflösung müssen hierzu passen; eine
Desktop-Skalierung darf die erwartete Bildgröße nicht unbemerkt ändern.
Die Sollwerte werden beim Szenenentwurf festgelegt und nicht aus einem gerade
erzeugten Screenshot übernommen. Nach einem ausdrücklichen Tick prüft die Suite
den ersten Capture und das Überschreiben derselben Datei ohne Capture-Retries.
Wann vor diesem Tick das erste Bild möglich ist, bleibt eine spätere Diagnostik.

Im aktuellen Headless-Vertrag liefert Screenshot `screenshot_unavailable`,
erzeugt keine Datei und verändert den Zustand nicht. Headless-Rendering ist
noch keine Anforderung dieser App-Version.

## Zusätzliche Fehlerfälle zur Ablösung der bisherigen Programme

Die normale App-Matrix braucht zunächst die Fähigkeiten oben. Um danach auch
`activity_retention/producer.rs` und `reports/failure_source.rs` abzulösen,
benötigt dieselbe App ausdrücklich auswählbare Fehlerszenarien. Vorgeschlagen
ist eine Startoption `--scenario <name>`, getrennt von `--mode`:

| Szenario | Erzeugtes Verhalten |
| --- | --- |
| `activity` | Reflektierbare Werte mit 2 MiB `m` und 5 MiB `x`; ein Tracing-Error beim ersten Tick, damit ein großer Report mit Inspect-Historie entsteht |
| `continuous-failures` | Ein Tracing-Error pro ausdrücklich ausgeführtem Tick für Queue-/Provider-Last |
| `caught` | Gefangener Panic im App-System; die Session bleibt verwendbar |
| `worker`, `task` | Panic in einem Thread beziehungsweise Bevy-Task |
| `unknown` | Gefangener Panic mit nicht textuellem Payload |
| `panic` | Ungefangener Panic im App-System beim ersten Tick |
| `abort` | Panic mit anschließendem Abbruch im Panic-Hook, wie im bisherigen Fehlerprogramm |
| `trace`, `fields` | Tracing-Error mit Nachricht beziehungsweise strukturierten Feldern einschließlich Unicode |
| `filtered`, `handled` | Herausgefilterter Tracing-Error beziehungsweise gewöhnliche stderr-Diagnostik, aus denen kein Report entstehen darf |
| `pre_ready` | Panic während Startup vor Ready |
| `missing_layer` | Tracing-Layer absichtlich nicht registrieren |
| `exit_zero` | Unerwartetes Prozessende mit Exit-Code null beim ersten Tick |
| `corrupt_marker`, `incomplete_marker` | Ungültiger beziehungsweise unvollständiger Report-Marker auf stderr vor Prozessende |

Ein echter `panic=abort` wird durch einen separaten Build desselben App-Quellcodes
geprüft, nicht durch bloßes `process::abort()`. Die Fehlerfälle müssen zudem den
vorherigen Panic-Hook weiter aufrufen und den tatsächlichen Prozess erkennbar
machen. Aktuell verwenden die Verträge dafür `fixture-pid` und `prior-hook` im
von `WOODPECKER_ARTIFACT_DIR` angegebenen Ordner.

Diese Startoption und die Umschaltung der Fehlerverträge auf die gemeinsame App
sind noch umzusetzen. Aktuell baut und startet der Runner für diese Fälle die
bisherigen Programme. Type-Paths, Fehlermeldungen und Startargumente werden bei
Übernahme zusammen mit den Verträgen angepasst; das Eintragen normaler
`capabilities` allein ersetzt diese Anbindung nicht.

Absichtlich falsche Woodpecker-Protokollantworten bleiben Aufgabe von
`connection/protocol_faults/peer`. Blockierte oder fehlschlagende externe
Report-Provider werden in der Testumgebung erzeugt. Die Bevy-App liefert dafür
den echten Fehler; sie implementiert keinen künstlichen GitHub-Provider.

## Konfiguration und schrittweise Anbindung

`app.example.json` als `app.json` übernehmen und die beiden Launch-TOMLs auf
das echte App-Manifest und Target einstellen. `state_type`, `entity_name`,
`failure_message` und Screenshot-Sollwerte müssen zur App passen. TOML-Pfade in
`modes` sind relativ zur JSON-Datei, Manifest-Pfade relativ zum Repository-Root
oder absolut.

Nur bereits implementierte Fähigkeiten eintragen:

1. `state`: Plugin, Tick-Zähler und Prozess-ID. Warp, Connection, Recording und
   Replay ausführen. Beim Replay muss ein vorzeitig gestoppter Warp dieselbe
   tatsächliche Tick-Anzahl in einer frischen Session erzeugen.
2. `entities`, `keyboard`, `pointer`, `text`, `reports`: Welt und echte
   Input-/Textverarbeitung in beiden Modi ergänzen und die Fachverträge ausführen.
3. `screenshot` nur für Rendered: feste Szene und bekannte Sollpixel prüfen.
4. Die vollständige Matrix ausführen.
5. Die Fehlerfälle auf dieselbe App umstellen, gleichwertige Nachweise erbringen
   und erst danach die bisherigen technischen Bevy-Programme entfernen.

```sh
python3 -m tests.run --app tests/application/app.json --feature warp --mode headless
python3 -m tests.run --app tests/application/app.json --layer system --mode headless
python3 -m tests.run --app tests/application/app.json --all --allow-rendered
```

Die vollständige App-Matrix umfasst `state`, `entities`, `keyboard`, `pointer`,
`text` und `reports` in beiden Modi sowie `screenshot` in Rendered. Fehlende
Fähigkeiten bleiben unvollständige Nachweise.
