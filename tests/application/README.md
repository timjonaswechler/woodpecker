# Vertrag der gemeinsamen Bevy Test App

Alle Systemtests verwenden dieselbe App. Rendered und Headless unterscheiden
ihre Laufzeitkonfiguration und teilen beobachtbare Zustände und Regeln.
Die Suite stellt noch keine neue Spiel-App bereit.

## Konfiguration

`app.example.json` als `app.json` übernehmen und die Launch-Konfigurationen an
die eigene App anpassen. Die Beispieldateien sind keine lauffähige App.

```sh
python3 -m tests.run --app tests/application/app.json --feature warp --mode headless
```

Die JSON-Datei enthält `version: 1`, den vollständigen Reflection-`state_type`
und `modes`. Jeder Modus hat eine `config` relativ zur JSON-Datei und die
Liste tatsächlich implementierter `capabilities`. Die TOML-Datei ist eine
normale Woodpecker-Launch-Konfiguration. Ihre Manifest-Pfade sind relativ zum
Repository-Root oder absolut. Der Runner baut das Target vor den Sessions.
Beide Modi können denselben Binary mit unterschiedlichen Argumenten verwenden.

## Beobachtbarer Zustand

Eine registrierte, lesbare Resource liefert über `inspect.query` diese Werte.
Ihr Rust-Name ist über `state_type` konfigurierbar:

```json
{
  "ticks": 0,
  "process_id": 1234,
  "keyboard": {"pressed": [], "press_count": 0, "release_count": 0},
  "pointer": {
    "position": [0, 0], "pressed": [], "press_count": 0,
    "release_count": 0, "scroll": [0, 0]
  },
  "text": "",
  "failures": 0
}
```

| Fähigkeit | Verbindliche Beobachtung |
| --- | --- |
| `state` | `ticks` zählt tatsächliche Update-Ausführungen, `process_id` bezeichnet den App-Prozess. Vor dem ersten expliziten Tick ist `ticks` null. |
| `keyboard` | `pressed` enthält Woodpecker-Key-Tokens. Zähler erfassen echte Press-/Release-Ereignisse; Halten erzeugt keinen weiteren Press. |
| `pointer` | Position in logischen Pixeln, Buttons `left/right/middle`, kumulierte Press-/Release-Ereignisse und Scroll-Line-Deltas. Mindestens 32 × 32 logische Pixel. |
| `text` | Nach einem Initialisierungstick ist ein gewöhnliches EditableText-Feld fokussiert. `text` spiegelt den echten Inhalt. Unicode wird unverändert eingefügt. |
| `entities` | Nach einem Initialisierungstick existiert genau ein Entity mit konfiguriertem `entity_name`, lesbarem Name-Component und abfragbarer Hierarchie. |
| `reports` | Ein neuer Press von `b` erzeugt beim Tick genau einen Tracing-Error mit konfiguriertem `failure_message`; `failures` zählt die echten Auslösungen. Launch-Konfiguration aktiviert lokale automatische Reports. |
| `screenshot` | Rendered liefert PNGs mit konfigurierten Abmessungen und unabhängig bekannten Sollpixeln. |

Die App beobachtet Bevy-Eingaben und deren Wirkung. Sie darf die TestResource
nicht direkt aus eingehenden Woodpecker-Commands füllen, sonst würden Queuing,
Tick-Verarbeitung und UI-Verhalten umgangen. Zusätzliche Zustandsfelder sind
erlaubt; Stillstandprüfungen vergleichen sie mit.
Headless kann logische Window-/Input-/UI-Daten besitzen, ohne native Fenster
oder einen Renderer zu starten.

Frische Sessions sind deterministisch: keine gedrückten Tasten/Buttons,
Input-Zähler null, Textfeld leer. Pace ändert die Ausführungsfrequenz, nicht die
Wirkung einzelner Ticks. Physische Eingaben verändern kontrollierte Sessions nicht.

## Bilder und Grenzen

Screenshot-Sollwerte legen Breite, Höhe und Pixel mit `at` und `rgb` fest.
Die Beispiele erwarten ein grünes Zentrum auf schwarzem Hintergrund nach einem
expliziten Tick. Diese Werte an das eigene deterministische Bild anpassen;
nicht aus dem gerade erzeugten Screenshot übernehmen.

Headless erwartet aktuell `screenshot_unavailable` und keine Datei. Wenn
Headless-Rendering hinzukommt, dessen Bildvertrag ergänzen. Der Zeitpunkt der
ersten Bildverfügbarkeit gehört zur späteren Diagnostik. Normale Bildtests
starten nach einem ausdrücklichen Tick und haben keine Capture-Retries.

## Anbindungsschritte

1. App mit Session-Plugin und lesbarer Zustandsresource starten.
2. `state` konfigurieren; Warp, Connection, Recording und Replay ausführen.
3. Reale Input-/Entity-/UI-Beobachtung ergänzen und erst danach entsprechende Fähigkeiten eintragen.
4. Rendered anbinden und bekannte Sollpixel festlegen.
5. Die vollständige Matrix ausführen. Fehlende Fähigkeiten bleiben offene Nachweise.

## Ablösung der technischen Bevy-Programme

`connection/activity_retention/producer.rs` erzeugt große Reflection-Werte und
wiederholte Fehler, um begrenzte Activity-Queues zu prüfen.
`reports/failure_source.rs` erzeugt unterschiedliche Fehlerursprünge, Fehler vor
Ready und blockierte Report-Provider. Diese Fälle laufen bis zu ihrer Übernahme
mit den bisherigen Programmen direkt bei den betreffenden Feature-Verträgen.

Die gemeinsame App soll diese Fähigkeiten als ausdrücklich auswählbare
Fehlerszenarien anbieten. Die zugehörigen Verträge müssen mit ihr bestehen,
bevor die bisherigen Programme entfallen. Der Panic-Abort-Vertrag benötigt
zusätzlich einen echten Build mit `panic=abort`; dafür soll derselbe App-Quellcode
verwendet werden.

Der reine Protokollpartner unter `connection/protocol_faults/peer` erzeugt
absichtlich ungültige Protokollantworten und bleibt ein gesondertes Fehlerprogramm.
