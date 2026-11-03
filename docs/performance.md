# Tick-Warp-Durchsatz und Performance-Anzeige

FPS zählen gerenderte Fensterframes. TPS zählen vollständig ausgeführte
Woodpecker-Anwendungsticks, nicht `FixedUpdate`-Aufrufe. Mehr TPS beschleunigen
simulierte Zeit nur entsprechend der gewählten Clock-Politik; für reproduzierbare
Warps empfiehlt sich eine feste Tickdauer. Eine höhere Render-FPS-Zahl bedeutet
nicht, dass der Monitor ebenso viele unterschiedliche Bilder anzeigen kann.

## Größere Warp-Batches im eigenen Spiel aktivieren

`session::Plugin` behält ohne weiteres Opt-in sein bisheriges Arbeitsbudget.
Für gerenderte Anwendungen kann danach `ThroughputPlugin` installiert werden:

```rust
app.add_plugins(woodpecker::session::Plugin);
app.add_plugins(
    woodpecker::session::ThroughputPlugin::default()
        .with_asset::<StandardMaterial>()
        .with_asset::<bevy::gizmos::GizmoAsset>(),
);
```

Das Beispiel setzt voraus, dass das Spiel PBR und Gizmos bereits installiert hat,
wie [Signalstation](../test_app/src/woodpecker.rs). Das Plugin benötigt einen
anwendungsseitigen `RenderApp`; es installiert weder Renderer noch Asset-Plugins.
Mesh-, Image-, Shader- und ShaderBuffer-Nachrichten sind bereits enthalten.
**Jede zusätzliche vom Renderer gelesene Asset-Quelle muss mit `with_asset` genannt
werden**, auch eigene Materialien. Die zugehörigen Asset-Plugins müssen zuvor
installiert sein. Ohne vollständige Registrierung können Asset-Änderungen während
langer Warps beim Renderer fehlen. CPU-seitige Szenencontainer brauchen keine
Registrierung allein wegen ihrer Existenz.

Bei `AsFastAsPossible` teilen mehrere kurze Tick-Batches einen Renderframe.
Zwischen den Batches werden Commands, Stop und Inspect weiter bedient. Nach
ungefähr 33 ms wird wieder an Extraction, Rendering und native Fensterereignisse
übergeben. Das ist ein weiches Arbeitsbudget, keine garantierte Bildrate oder
maximale Antwortzeit: Ein laufender Tick wird nicht unterbrochen, und GPU-Arbeit
kann zusätzlich begrenzen. Paced- und Idle-Sessions behalten den normalen
Renderrhythmus. Die Zahl und Reihenfolge der Anwendungsticks ändern sich nicht.

Der Grund für die Asset-Registrierung: Bevy altert die normalen Nachrichtenpuffer
pro Anwendungstick. Ein großer Warp kann damit eine Asset-Nachricht löschen,
bevor die nächste Extraction sie liest. Das Plugin kopiert die Nachrichten mit
eigenen Lesecursorn in einen getrennten Renderstrom. Nur während der Extraction
sieht der Renderer diesen Strom; anschließend bekommt die Anwendung ihren
unveränderten Nachrichtenpuffer zurück. Dessen Lebensdauer und Lesecursor werden
nicht verlängert oder zurückgesetzt. Es werden keine Assets pro Tick neu gebaut
oder pauschal hochgeladen.

## Overlay und unbegrenzte Bildrate in Signalstation

```sh
cargo run -p test_app -- --uncapped --performance-overlay
```

- `--uncapped` wählt `PresentMode::AutoNoVsync`. Backend und Hardware bestimmen,
  welcher Modus tatsächlich verfügbar ist. Ohne Flag bleibt VSync unverändert.
- `--performance-overlay` aktiviert die Anzeige; F8 schaltet sie um. In einer
  kontrollierten Session ist dafür virtuelles F8 mit einem expliziten Tick nötig.
- `--performance-log` schreibt dieselben Messwerte als
  `SIGNALSTATION_PERFORMANCE <JSON>` nach stderr.

Die Anzeige aktualisiert sich ungefähr alle 250 ms. Sie enthält FPS, TPS,
den Zuwachs von `Time<Virtual>` pro realer Sekunde, den Warp-Modus und während eines Warps
abgeschlossene/verbleibende Ticks. Bei kontrollierter Pause laufen Rendering und
Anzeige weiter, während TPS und Simulationsfaktor nach dem Messfenster null sind.
Ein eigener Renderpass benötigt dafür keine zusätzlichen Gameplay-/UI-Schedules,
Ticks oder Zeitaktualisierungen. In der nativen App bedeutet TPS weiterhin
Anwendungsticks, nicht ausschließlich aktive Spiellogik.

Die reflektierte Resource `test_app::performance::Metrics` und das Overlay nutzen
dieselben Zähler. Für eigene Anzeigen liefert
`session::Plugin::warp_progress(world)` den aktiven Warp zwischen Ticks,
beispielsweise während der Extraction. Während eines Anwendungsticks ist die
interne Bridge vorübergehend aus dem World herausgenommen; dort liefert diese
Abfrage keinen Fortschritt.

## Wiederholbar messen

[`examples/performance.py`](../examples/performance.py) ist ein separates,
macOS-spezifisches Messwerkzeug. Es benötigt Python, Pillow, `system_profiler`,
`osascript`, eine wache sichtbare Desktop-Sitzung und die nötige Berechtigung zum
Aktivieren des selbst gestarteten Spielfensters. Grafische Läufe vorher freigeben.
Es gibt keine automatischen Wiederholungen oder rendererlosen Ersatzläufe.

```sh
cargo build --locked -p test_app --features automation --bin test_app
python3 examples/performance.py --output target/test-runs/perf-current \
  --app-arg=--performance-overlay --require-metrics --verify --capture
python3 examples/performance.py --native --app-arg=--uncapped \
  --app-arg=--performance-overlay --output target/test-runs/perf-native
```

Das Werkzeug startet die echte App direkt über ihr Spielprotokoll mit isolierten
Settings; es benötigt keinen Server. Nach einer begrenzten Ladephase und 120
expliziten Warm-up-Ticks prüft es die geladene Szene per PNG. Dann misst es drei
Warps mit je 600 Ticks und fester Tickdauer von 16.666.667 ns. Builds und Laden
liegen außerhalb des Zeitfensters. Es kontrolliert Tickzahl, Zeitfortschritt,
Pause, Inspect-/Stop-Latenz und optional F8, Pace-/Maximaltempo-Fortschritt sowie
Screenshots während und nach einem Warp. Die native Variante beobachtet acht
Sekunden, hält dabei ihr eigenes Fenster sichtbar und beendet nur ihren eigenen
Testprozess ausdrücklich per SIGTERM;
sie ist kein Nachweis des kontrollierten Session-Shutdowns.

`--binary` wählt ein vorher separat gebautes Referenzprogramm. Für ältere
Referenzen ohne Metrics kann `--require-metrics` entfallen. `--compare` nimmt
dessen `result.json`: Alle neuen Warp-Messungen müssen schneller als die
schnellste Referenzmessung sein; die gemessenen Inspect-/Stop-Maxima dürfen nicht
steigen. Messbedingungen müssen übereinstimmen. Ein reines Geschwindigkeitsplus
bei fehlender Szene zählt nicht als Erfolg.

Die Ausgabeverzeichnisse dürfen noch nicht existieren. Sie enthalten Manifest,
Binärhash, Commands, stderr, Ergebnisse und angeforderte PNGs. `validated: true` wird erst nach allen
angeforderten Prüfungen gesetzt; ein früher geschriebenes Teilergebnis mit
`validated: false` ist kein bestandener Nachweis. Fenstergröße,
Skalierung und Display-Konfiguration während eines Laufs nicht ändern. Eine
geänderte Umgebung verlangt eine neue Baseline, keinen nachträglich gelockerten
Vergleich. Die Messung ist ein lokaler Vergleich, kein universelles FPS-/TPS-SLA.
