# Anwenderanleitung: Woodpecker am Beispiel von Signalstation

[Signalstation in `test_app/`](../test_app/README.md) ist ein normales, spielbares
Bevy-Spiel: mit Bewegung, Kollisionen, Aufgaben, Einstellungen und eigener UI.
Es ist zugleich unser Beispiel dafür, wie Woodpecker in ein bestehendes Spiel
integriert wird. Wir verwenden hier dieses Spiel — kein separates Testspiel und
keine eigens für Woodpecker geschriebene Bewegungslogik.

Der Ablauf: **Spiel normal starten → seine Integration verstehen → dasselbe
Spiel mit Woodpecker starten → steuern und beobachten → weitere Funktionen nutzen.**

## 1. Das Spiel zuerst normal kennenlernen

Alle Befehle dieser Anleitung gelten vom Repository-Root. Voraussetzungen sind
Bevy 0.20.0, Rust 1.97.1 oder neuer und für Server/CLI Unix-Prozessgruppen
(macOS oder Linux). Grafische Läufe benötigen einen Renderer und ein Display.

```sh
cargo run --locked --manifest-path test_app/Cargo.toml --bin test_app
```

Ohne Feature `automation` ist Woodpecker keine Abhängigkeit des Spiels. Du kannst
dich mit den Pfeiltasten bewegen, auf begehbaren Boden klicken und mit M die
Einstellungen öffnen. Aufgaben, weitere Steuerung und Levelbeschreibung stehen
in der [Spielanleitung](../test_app/README.md#spielen). Beende diesen normalen
Spielstart, bevor du unten eine kontrollierte Session öffnest.

## 2. So ist Woodpecker in dieses Spiel eingebunden

### Optionale Abhängigkeit statt eines zweiten Spiels

Die [Spiel-`Cargo.toml`](../test_app/Cargo.toml) enthält:

```toml
[features]
automation = ["dep:woodpecker"]

[dependencies]
woodpecker = { path = "..", optional = true, features = ["ui", "screenshot"] }
```

Das ist ein Auszug, kein Ersatz für die Datei. `automation` ist ein Feature
**dieses Spiels**. Die relative Pfadabhängigkeit zeigt auf Woodpecker im
übergeordneten Repository-Ordner. `ui` und `screenshot` schalten die hier
verwendeten Zusatzfunktionen frei; Server und CLI werden nicht ins Spiel eingebaut.

In [main.rs](../test_app/src/main.rs) bleibt der normale Spielaufbau bestehen.
Danach wird optional die Anbindung installiert (Auszug):

```rust
app::install(&mut app, &options, log_plugin);
#[cfg(feature = "automation")]
woodpecker::install(&mut app);
app.run();
```

[woodpecker::install](../test_app/src/woodpecker.rs) prüft zusätzlich, ob der
Server diesen Prozess gestartet hat. Ohne dessen `WOODPECKER_ARTIFACT_DIR`
kehrt die Funktion zurück. Auch ein Build mit `automation` lässt sich deshalb
normal starten. Diese interne Umgebung nicht selbst setzen: Der Server richtet
Protokoll, Artefaktordner und Session-Workspace ein.

### Normale Eingaben und Spiellogik bleiben bestehen

Die Integration setzt den kontrollierten Eingabemodus und hält den Winit-Lauf
auch ohne Fensterfokus ansprechbar. Sie installiert das Session-Plugin nach den
Spiel-Plugins. Dieses übernimmt die explizite Ausführung der Haupt-Schedules.

Die vorhandene Eingabeverarbeitung in [input/](../test_app/src/input/mod.rs)
verarbeitet Bevy-Tastaturzustände und Mausbewegungen. Der
[Geräteadapter](../test_app/src/input/devices.rs) unterscheidet bei Fokus und
Cursor zwischen nativem Fenster und virtuellem Pointer. Beide Wege liefern
Spielabsichten an dieselbe Bewegungsfunktion:

```text
native Eingaben oder virtuelle Bevy-Eingaben
  → vorhandene Eingabeverarbeitung / FrameInput
  → move_player mit Bevy-Time, Geschwindigkeit und Navigation
  → normale Bewegung, Kollision und Darstellung
```

[move_player](../test_app/src/player/movement.rs) berechnet Bewegung aus Richtung,
Geschwindigkeit und `Time::delta_secs()`. Für Woodpecker wird dort weder ein Tick
hochgezählt noch eine Position durch eine Testantwort ersetzt. Aufgaben, Portale
und Kollisionen gelten auch bei externer Steuerung unverändert.

Ein Woodpecker-Tick führt die konfigurierte Haupt-Schedule-Reihenfolge aus; er
ist nicht automatisch genau ein `FixedUpdate`. Die gewählte feste Uhr liefert
Bevys Simulationszeit. Eigene Threads, Netzwerk oder externe Zeitquellen werden
dadurch nicht automatisch pausiert.

### Spielzustand beobachten, Laden und Speichern anbinden

Die Anbindung erledigt außerdem drei Dinge am Rand des Spiels:

- **Inspect:** Sie registriert vorhandene reflektierte Typen wie `Player`,
  `LevelLoad`, `Mission` und `Settings`. Damit lesen Clients echte Spielwerte.
  Neue beobachtbare Resources brauchen `Reflect`, `#[reflect(Resource)]` und
  Typregistrierung; Components entsprechend `#[reflect(Component)]`.
  Registrierung allein erzeugt keinen Spielzustand.
- **Ladephase:** `startup::Assets` liest die tatsächlichen Handles aus
  `LevelAssets`. Technisches Engine-Startup kann sie beobachten. Ob das Level
  spielbereit ist, entscheidet weiterhin das vorhandene `LevelLoad` des Spiels.
- **Einstellungen:** [settings/storage.rs](../test_app/src/app/settings/storage.rs)
  verwendet in einer Session `workspace/config/settings.json` statt des normalen
  `test_app/settings.json`. Der bekannte Seed wird einmal in den frischen
  Workspace übernommen. Das verhindert hier gemeinsame Schreibzugriffe;
  Woodpecker ist aber keine Betriebssystem-Sandbox für beliebige Dateien.

**Nicht mitkopieren müssen:** `SimulationTicks` und die explizit zuschaltbaren
Fehler-/Prozessproben in `src/woodpecker.rs` sind Messinstrumente für die
Abnahmetests. Der separate `Last`-Zähler beobachtet Durchläufe; er erzeugt keine
Ticks und steuert keine Bewegung. Er ist keine Voraussetzung für die Integration
oder die einfachen Steuerungsbeispiele.

## 3. Dasselbe Spiel kontrolliert starten

**Diese Befehle öffnen ein echtes Signalstation-Fenster.** Als Agent vorher die
Erlaubnis für den Lauf einholen. Display wach und Fenster teilweise sichtbar
halten; während der Session nicht skalieren, verschieben zwischen Bildschirmen
oder die Größe ändern. Native Gameplay-Eingaben werden in kontrollierten
Sessions verworfen; steuere dann über Woodpecker. Ein unfokussierter Start kann
hinter anderen Fenstern oder einer Vollbildanwendung liegen. Mache das Spielfenster
für den Rendernachweis sichtbar; Eingaben sendest du weiterhin über Woodpecker.

Baue CLI und Spiel mit den Features der Startkonfiguration:

```sh
cargo build --locked --features cli --bin woodpecker
cargo build --locked --manifest-path test_app/Cargo.toml --features automation --bin test_app
```

Terminal 1, weiterhin vom Repository-Root:

```sh
target/debug/woodpecker --address 127.0.0.1:4100 \
  server start --artifact-dir target/game-example-sessions
```

Warte auf `Ready`. Terminal 2:

```sh
wp() { target/debug/woodpecker --address 127.0.0.1:4100 "$@"; }
wp session create --config examples/game_tests/app.toml
```

Nutze eine freie lokale Adresse, deren Server du selbst verwaltest. Ersetze den
folgenden Platzhalter durch die **vollständige ID aus diesem Create**:

```sh
ID=VOLLSTAENDIGE_ID_AUS_CREATE
wp session inspect "$ID"
```

Beobachte bei `Starting`/`SessionReady` weiter bis `EngineReady`. Bei Fehler,
`Failed` oder nach 90 Sekunden ohne Bestätigung abbrechen und die eigene Session
aufräumen; einen unbestätigten Create nicht blind wiederholen.

### Was die verwendete Konfiguration bedeutet

[app.toml](../examples/game_tests/app.toml) ist die vollständige, ausführbare
Startkonfiguration für dieses Spiel. Sie wählt `test_app` als Package und Binary,
aktiviert `automation` und übergibt ausschließlich vorhandene Spielargumente:
Settings-Seed, unfokussiertes Fenster, feste Größe und Skalierung 1. Diese
Argumente sind keine universellen Woodpecker-Flags.

Der relative Manifestpfad gilt vom **Server-Arbeitsverzeichnis**, nicht neben der
TOML. Das gestartete Spiel arbeitet im Ordner des konfigurierten Manifests;
`../examples/game_tests/settings.json` ist deshalb vom Ordner `test_app/` aus
richtig. Im eigenen Projekt kannst du zunächst einen absoluten Manifestpfad nutzen.

Die feste Simulationsdauer beträgt ungefähr 1/60 Sekunde pro Tick. Die Pace
`as_fast_as_possible` bestimmt nur, wie schnell der Rechner sie abarbeitet.
Engine-Startup darf höchstens 3600 normale Vorbereitungsticks/60 Sekunden
verwenden. Diese Vorbereitung zählt zum Startzustand; `EngineReady` ist noch
keine Zusage, dass der Spielzustand `LevelLoad` bereits `Ready` ist.

## 4. Die vorhandene Figur bewegen und ihre Position lesen

```sh
wp session repl "$ID"
```

Gib Zeilen **einzeln** ein und warte jeweils auf `completed` mit passender
Request-ID und passendem Command-Namen. `pending` bestätigt nur Annahme.
Bei `rejected`, `failed` oder ungeklärtem Ausgang abbrechen, nicht neu einreichen.

Prüfe zuerst den echten Ladezustand:

```text
inspect resource test_app::level::LevelLoad
```

Bei `Loading` fordere ausdrücklich `tick warp 1` an, warte auf Abschluss und
prüfe erneut. Bei anderem Fehlerzustand oder nach 120 zusätzlichen
Vorbereitungsticks ohne `Ready` abbrechen. Diese Vorbereitung ist getrennt von
der folgenden Bewegung.

Sobald `Ready`, beginne mit einer Positionsbeobachtung:

```text
inspect resource test_app::player::Player
command {"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}
tick warp 4
inspect resource test_app::player::Player
command {"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}
tick warp 1
inspect resource test_app::player::Player
```

**Erwartung am frischen, mit dem Seed gestarteten Spiel:** `position[0]` nimmt
während der vier Bewegungsticks zu; `position[1]` bleibt gleich. Nach dem
Loslassen bleibt die Position beim nächsten Tick unverändert. Der Warp-Abschluss
meldet die tatsächlich ausgeführten Ticks. Keine Annahme „ein Tick = ein Meter“:
Die Strecke ergibt sich aus Signalstations normaler Geschwindigkeit und Zeit.

Die Taste bleibt bis zum Release gehalten. Inspect liest nur; es schiebt weder
einen Tick noch eine Spielaktion ein. `inspect resources`, `inspect entities`
und `help inspect` erschließen weitere vorhandene Typen und Komponenten.

Nach der Bewegung kannst du ein Bild derselben Szene anfordern:

```text
command {"command":"screenshot.capture","arguments":{"path":"screenshots/signalstation.png"}}
quit
```

Warte auch hier auf `completed`, bevor du `quit` eingibst. Das PNG liegt relativ
zum absoluten `artifact_dir` aus `session inspect`. Capture tickt nicht. Der
Renderer braucht mindestens einen ausgeführten Tick; eine fehlende Surface kann
zur Ablehnung führen. Ein tatsächlich schwarzes Bild ist kein technischer Fehler.
Automatische Bildvergleiche sind keine Woodpecker-Funktion.

### Die eigene Session beenden

`quit` trennt nur die REPL. Zurück im Terminal:

```sh
wp session stop "$ID"
wp session inspect "$ID"
```

Beobachte bis `Ended`, höchstens 40 Sekunden; `Failed` oder ein unbekanntes
Cleanup-Ergebnis ist kein Erfolg. Stoppe nur die bestätigte eigene Session und
wiederhole keinen ungewissen Stop blind. Den eigenen Server kannst du zuletzt
mit `wp server stop` beenden; das betrifft auch dessen andere Sessions.
Der Vordergrundprozess in Terminal 1 bestätigt durch sein Ende den Abschluss.
Artefakte bleiben erhalten. Der Server ist Loopback-only, lokale Clients sind
vertrauenswürdig; Remote-Betrieb wird nicht unterstützt.

## 5. Was du als Nächstes mit diesem Spiel machen kannst

- **Anders steuern:** Die [sechs einfachen Wege](../examples/README.md) zeigen
  dieselbe Bewegung über REPL, einzelne CLI-Aufrufe, feste Scripts, Python mit
  Zustandsentscheidung, eine externe Agent-Werkzeugschleife und Replay. Keine
  zusätzliche Spiellogik und kein eingebauter Modellservice sind nötig.
- **UI und Text:** [text.py](../examples/game_tests/text.py) findet Signalstations
  vorhandene Einstellungen und das Notizfeld, fokussiert es und gibt Unicode ein.
  Die Anwendung besitzt UI, Fokusregeln und `TextInputPlugin`; `woodpecker/ui`
  leitet die kontrollierten Eingaben hinein. Pointer bewegen keinen OS-Cursor.
- **Recording/Replay:** [replay.py](../examples/game_tests/replay.py) zeichnet
  normale Eingaben auf und wiederholt sie in einer frischen Session desselben
  Spiels. Replay setzt den Spielstand nicht automatisch zurück und prüft keine
  alten Sollwerte. Währenddessen ist nur `replay stop` als externer Spielbefehl
  erlaubt. Siehe [Recording und Replay](api/usage.md#recording).
- **Zeit und mehrere Sessions:** `tick warp 10 pace 20` taktet die Bearbeitung
  von zehn angeforderten Ticks, ohne deren simulierte Dauer zu ändern.
  `tick warp stop` beendet einen aktiven Warp; prüfe dessen tatsächliches Ergebnis.
  Ein Server kann mehrere getrennte Spielprozesse mit eigenen IDs, Eingaben und
  Artefakten verwalten. `session ls` überträgt dir keine fremden Sessions.
- **Fehlerberichte:** Das Spiel bindet vor dem Session-Plugin den
  `tracing_error_layer` in sein `LogPlugin` ein. Die Beispielkonfiguration nutzt
  lokale Reports und lässt Tracing-Errors aus; aktiviere dafür zusätzlich
  `report.tracing_errors = true`. Panics werden vom Session-Plugin beobachtet;
  gewöhnlicher stderr-Text allein ist kein Fehlerauslöser. Die
  [Report-Anleitung](api/usage.md#fehlerbeobachtung-und-snapshots) erklärt lokale
  Dateien, optionalen GitHub-Provider und echten Veröffentlichungseffekt. Logs,
  Argumente und Aufnahmen können vertrauliche Daten enthalten.

Die [ausführlicheren Prüfrezepte](../examples/game_tests/README.md) behandeln
zusätzliche Assertions und Messinstrumentierung. Sie sind keine Voraussetzung,
um dein Spiel zu integrieren. Die [Bedienreferenz](api/usage.md) beschreibt
Command-Formate; die [Abnahmesuite](../tests/README.md) prüft Produktverträge,
nicht die Korrektheit deines gesamten Spiels.

## 6. Diesen Integrationsweg auf dein eigenes Spiel übertragen

1. **Normales Spiel behalten.** Bevy-Version und Toolchain abgleichen;
   Spiellogik, Bewegung, Kollisionen und vorhandene Bevy-Eingaben nicht durch
   Woodpecker-Testlogik ersetzen.
2. **Abhängigkeit und Einstieg ergänzen.** Die optionale Cargo-Abhängigkeit mit
   passendem Pfad und benötigten Features aufnehmen; bestehende Features und
   Lockdatei behalten. Eigenen Integrationsadapter wie `src/woodpecker.rs` nach
   den Spiel-Plugins aufrufen, nur für kontrollierte Prozesse aktivieren.
3. **Gerätegrenze prüfen.** Eigene Fokus-/Cursorfilter müssen virtuelle Eingaben
   zulassen. Bei Picking den virtuellen Pointer statt fest die OS-Maus verwenden.
   Normalen und kontrollierten Betrieb in dieselben Spielabsichten überführen.
4. **Beobachtbarkeit wählen.** Vorhandene Ressourcen/Komponenten registrieren,
   die dein Steuerprogramm braucht. Das ist keine Aufforderung, Tickzähler in
   Gameplay-Systeme einzubauen. Eigene Geschäftsregeln definieren den Erfolg.
5. **Start und Dateien anbinden.** Für Engine-Startup tatsächliche Asset-Handles
   liefern und eigene Ladezustände prüfen; alternativ `startup.kind = "manual"`
   mit expliziter Vorbereitung wählen. Persönliche Speicherorte im kontrollierten
   Prozess auf das Verzeichnis aus `WOODPECKER_SESSION_DIR` umstellen.
6. **Startkonfiguration anpassen und prüfen.** Manifest, Package, Target,
   Spiel-Features und Spielargumente aus `app.toml` ersetzen. Normal bauen und
   spielen, kontrolliert starten und eine echte Eingabe samt Zustandsänderung
   und Session-Ende prüfen. UI, Screenshot oder Tracing-Reports benötigen jeweils
   die genannten zusätzlichen Voraussetzungen — nicht jede Anwendung alles.
