# Anwenderanleitung: ein eigenes Spiel mit Woodpecker steuern

Diese Anleitung führt von einem eigenen Bevy-Spiel bis zur ersten überprüfbaren
Eingabe. Danach kannst du Spielzustände auslesen, Screenshots aufnehmen,
Steuerfolgen automatisieren und Fehlerberichte sammeln.

Woodpecker wird **in dein Spiel eingebaut**. Ein lokaler Server startet anschließend
einen eigenen Spielprozess pro Session. Es hängt sich nicht nachträglich an ein
beliebiges laufendes Spiel an und simuliert keine Betriebssystem-Tastatur.

## 1. Voraussetzungen prüfen

- Dein Spiel verwendet **Bevy 0.20.0**, die Toolchain **Rust 1.97.1 oder neuer**.
- Server und CLI benötigen Unix-Prozessgruppen, etwa unter macOS oder Linux.
- Du hast dieses Woodpecker-Repository lokal ausgecheckt. Die Beispiele unten
  verwenden eine lokale Cargo-Pfadabhängigkeit auf diesen Checkout.
- Für die gezeigte grafische Anwendung stehen Renderer und Display bereit.
  Als Agent vor einem Lauf die Erlaubnis für echte Spielfenster einholen.
  Display wach und Fenster teilweise sichtbar halten; während der Session
  Fenstergröße, Skalierung und Bildschirm nicht wechseln.

Der Server ist nur über Loopback erreichbar. Lokale Clients sind vertrauenswürdig;
es gibt weder Zugangsschlüssel noch einen unterstützten Remote-Betrieb.

## 2. Die Bibliothek ins Spiel aufnehmen

Ergänze die bestehenden Abschnitte deiner **Spiel-`Cargo.toml`**, statt sie zu
ersetzen. Tausche den Platzhalter gegen deinen absoluten Woodpecker-Pfad aus:

```toml
[features]
automation = ["dep:woodpecker"]

[dependencies]
woodpecker = { path = "/ABSOLUTER/PFAD/woodpecker", optional = true, features = ["screenshot"] }
```

Die vorhandene Bevy-Abhängigkeit, deine Spiel-Features und `Cargo.lock` bleiben
bestehen. `automation` ist hier ein selbst gewähltes Feature des Spiels, kein reservierter
Woodpecker-Name. `screenshot` erlaubt später PNG-Aufnahmen; den Renderer stellt
weiterhin dein Spiel bereit. Für Bevy UI ergänze bei Woodpecker zusätzlich `ui`.
Die Features `cli`, `server` und `client` brauchst du **nicht** im Spiel.

## 3. Das Session-Plugin installieren und Spielzustand sichtbar machen

Installiere `woodpecker::session::Plugin` **nach den Spiel-Plugins**, und nur im
kontrollierten Prozess. Das folgende vollständige `main.rs` zeigt das Muster in
einem kleinen Bevy-Projekt mit `DefaultPlugins`, Winit und 2D-Renderer. Es zeichnet
noch keine Spielfigur: `Spielstand` macht Bewegung und Tickzahl überprüfbar.
Übertrage das Einbaumuster in dein Spiel, nicht dessen gesamte Spiellogik.

```rust
use bevy::prelude::*;

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct Spielstand {
    x: f32,
    ticks: u64,
}

fn bewegen(tasten: Res<ButtonInput<KeyCode>>, mut stand: ResMut<Spielstand>) {
    stand.ticks += 1;
    if tasten.pressed(KeyCode::ArrowRight) {
        stand.x += 1.0;
    }
}

fn kamera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .init_resource::<Spielstand>()
        .register_type::<Spielstand>()
        .add_systems(Startup, kamera)
        .add_systems(Update, bewegen);

    #[cfg(feature = "automation")]
    if std::env::var_os("WOODPECKER_ARTIFACT_DIR").is_some() {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .add_plugins(woodpecker::session::Plugin);
    }

    app.run();
}
```

**Was davon ist nötig?**

- Der Server setzt `WOODPECKER_ARTIFACT_DIR` für seinen Kindprozess. Die Abfrage
  verhindert, dass ein normaler Spielstart auf Session-Steuerung wartet.
  Diese interne Umgebung nicht von Hand setzen; der Server richtet sie ein.
- `WinitSettings::continuous()` hält den Kontrolllauf auch ohne Fensterfokus
  ansprechbar. Es gibt dadurch keine zusätzlichen Simulationsticks.
- Für Inspect müssen eigene Typen `Reflect` unterstützen und mit
  `register_type` registriert sein. Resources benötigen `#[reflect(Resource)]`,
  Components entsprechend `#[reflect(Component)]`. Registrierung allein erzeugt
  weder eine Resource noch eine Entity; das erledigt weiterhin dein Spiel.
- Normale Bevy-Eingabe über `ButtonInput<KeyCode>` kann bestehen bleiben.
  Eigene Eingabesysteme müssen die virtuellen Bevy-Ereignisse verarbeiten,
  statt den OS-Tastenzustand direkt abzufragen. Im kontrollierten Prozess werden
  native Maus-, Tastatur-, Touch- und IME-Eingaben verworfen.
- Das Plugin kontrolliert die vorhandene Reihenfolge der Haupt-Schedules. Ein
  Woodpecker-Tick ist nicht automatisch genau ein `FixedUpdate`-Durchlauf.
  Eigene Threads, Netzwerk und externe Zeitquellen pausiert es nicht mit.

**Erfolgskontrolle:** Dein Spiel baut sowohl normal als auch mit `--features
automation`. Ein normaler Start ohne Server bleibt normal bedienbar. Diagnoseausgaben
im kontrollierten Spiel gehören auf stderr bzw. ins Logging; stdin/stdout werden
für das Session-Protokoll benötigt.

## 4. Die Startkonfiguration anlegen

Speichere im Spielprojekt `woodpecker.toml`. Ersetze den absoluten Manifestpfad,
den Package-Namen und den Binary-Namen durch die Werte deines Spiels. Das
Beispiel nennt Package und Binary beide `mein_spiel`:

```toml
version = 1

[launch]
manifest_path = "/ABSOLUTER/PFAD/mein_spiel/Cargo.toml"
package = "mein_spiel"
features = ["automation"]
arguments = []

[launch.target]
kind = "binary"
name = "mein_spiel"

[tick.pace]
kind = "as_fast_as_possible"

[tick.clock]
kind = "fixed"
seconds_per_tick = 0.016666667

[startup]
kind = "manual"

[report]
tracing_errors = false
output = "reports"

[report.provider]
kind = "local"
```

`features` sind die Features **deines Spiels**. `arguments` sind dessen eigene
Kommandozeilenargumente; Woodpecker erfindet dafür keine Flags.

Ein relativer `manifest_path` gilt vom Arbeitsverzeichnis des **Servers**, nicht
vom Speicherort der TOML-Datei. Das gestartete `cargo run` arbeitet im Ordner des
konfigurierten Manifests. Beachte das für relative Asset- und Argumentpfade;
ein absoluter Manifestpfad vermeidet die erste Verwechslung.

`manual` führt keine automatischen Vorbereitungsticks aus: Steuerungsbereitschaft
heißt `SessionReady`. Ein fester Tick trägt ab dem ersten gezählten Tick die
konfigurierte Simulationsdauer bei. Die Pace bestimmt, wie schnell der Rechner
diese Ticks abarbeitet, nicht deren simulierte Dauer.

### Einstellungen nicht versehentlich ins persönliche Profil schreiben

Jede Session erhält unter ihrem Artefaktordner einen frischen
`workspace/config`, `workspace/data` und `workspace/tmp`. Dein Spiel muss seine
kontrollierten Einstellungen/Spielstände ausdrücklich unter
`WOODPECKER_SESSION_DIR/config` bzw. `WOODPECKER_SESSION_DIR/data` ablegen und
bei Bedarf einen bekannten Startstand laden. Gemeint sind Unterordner des
Verzeichnisses aus dieser Umgebungsvariable, keine wörtlichen Dateinamen.
Ohne diese Anpassung schreibt ein bestehender absoluter Speicherpfad weiterhin
an seine bisherige Stelle.
Die temporären Umgebungsvariablen werden vom Launcher gesetzt. Das ist eine
Pfadkonvention, **keine Betriebssystem-Sandbox** für beliebige Datei-/Netzwerkzugriffe.

## 5. Server starten und die erste Session öffnen

Ersetze die Großbuchstaben-Platzhalter. Baue zuerst das Spiel mit denselben
Features wie in der TOML, damit ein Compilerfehler nicht erst beim Start auffällt.
Die erste Zeile verwendet den Cargo-Pfad des Spiels:

```sh
cargo build --manifest-path /ABSOLUTER/PFAD/mein_spiel/Cargo.toml --features automation --bin mein_spiel
```

Terminal 1 — Woodpecker bauen, dann den Server im Vordergrund starten:

```sh
WOODPECKER=/ABSOLUTER/PFAD/woodpecker
cargo build --locked --manifest-path "$WOODPECKER/Cargo.toml" --features cli --bin woodpecker
"$WOODPECKER/target/debug/woodpecker" --address 127.0.0.1:4100 \
  server start --artifact-dir /ABSOLUTER/PFAD/mein_spiel/target/woodpecker-sessions
```

Warte auf die `Ready`-Zeile. Terminal 2 — Variablen werden nicht zwischen
Terminals übernommen, daher den Woodpecker-Pfad hier erneut setzen:

```sh
WOODPECKER=/ABSOLUTER/PFAD/woodpecker
wp() { "$WOODPECKER/target/debug/woodpecker" --address 127.0.0.1:4100 "$@"; }
wp session create --config /ABSOLUTER/PFAD/mein_spiel/woodpecker.toml
```

Übernimm die **vollständige zurückgegebene** ID, nicht eine fremde aus einer Liste:

```sh
ID=VOLLSTAENDIGE_ID_AUS_CREATE
wp session inspect "$ID"
```

Beobachte bei `Starting` weiter bis `SessionReady`, brich bei `Failed` oder einem
Fehler ab. Bleibt die Bereitschaft nach 90 Sekunden unbestätigt, fahre nicht mit
Eingaben fort; prüfe die Startdiagnose und stoppe die eigene Session. Ein
fehlgeschlagener/unbestätigter Create darf nicht blind wiederholt werden.

**Erfolgskontrolle:** Das Spiel öffnet ein Fenster, `session inspect` zeigt
`SessionReady` und einen absoluten `artifact_dir`. Es tickt noch nicht von selbst.

## 6. Eine Eingabe senden und die Wirkung prüfen

```sh
wp session repl "$ID"
```

Gib **eine Zeile nach der anderen** ein. Warte nach jedem Command auf das
`completed`-Ergebnis mit derselben Request-ID und demselben Command-Namen.
`pending` bestätigt nur Annahme, nicht Ausführung. Bei `rejected`, `failed` oder
ungewissem Ausgang abbrechen, nicht die Eingabe erneut senden.

Für das oben gezeigte Binary `mein_spiel`:

```text
inspect resources
inspect resource mein_spiel::Spielstand
command {"command":"input.keyboard.press","arguments":{"key":"arrow_right"}}
tick warp 4
inspect resource mein_spiel::Spielstand
command {"command":"input.keyboard.release","arguments":{"key":"arrow_right"}}
tick warp 1
inspect resource mein_spiel::Spielstand
quit
```

**Erwartung für dieses Minimalspiel:** anfangs `x = 0`, `ticks = 0`; nach dem
Vier-Tick-Warp `x = 4`, `ticks = 4`; nach dem Release-Tick weiterhin `x = 4`,
aber `ticks = 5`. Der Eingabecommand selbst ändert diese Werte nicht.

Im eigenen Spiel hängen Typ-Pfad und Erwartung von deiner Implementierung ab.
`inspect resources` hilft, den tatsächlich registrierten Pfad zu finden. Für
Components beginne mit `inspect entities`; `help inspect` erklärt die Abfragen.
Inspect ist lesend: Es ersetzt weder Eingaben noch beliebige World-Mutationen.

**Wichtig:** Die Taste bleibt zwischen Ticks gehalten. Ein angenommenes Press
ist keine einmalige Bewegung und kein Textzeichen. Text benötigt ein bestehendes,
fokussiertes editierbares Widget; es wird nicht durch Tastaturlayouts erzeugt.

### Aufräumen gehört zum Ablauf

`quit` beendet nur den Client. Zurück im Terminal:

```sh
wp session stop "$ID"
wp session inspect "$ID"
```

Beobachte bis `Ended`, höchstens 40 Sekunden; `Failed` oder unbestätigtes Cleanup
ist kein Erfolg. Stoppe nur die eigene bestätigte Session und wiederhole einen
ungewissen Stop nicht blind. Zum Schluss kannst du **deinen eigenen** Server mit
`wp server stop` beenden; das betrifft auch dessen andere Sessions. Der
Serverprozess in Terminal 1 bestätigt durch sein Ende den Abschluss.
Artefakte bleiben erhalten und werden nicht automatisch gelöscht.

## 7. Was du außerdem machen kannst

### Mehrere Sessions verwalten

Ein Server kann mehrere getrennte Spielprozesse verwalten, auch mit verschiedenen
Startkonfigurationen. Jede Session hat eigene Eingaben, Zeitsteuerung und
Artefakte. `session ls` zeigt den Überblick; es überträgt dir nicht die
Verantwortung für fremde Sessions. Verwende zur Steuerung und zum Stoppen immer
die bestätigten IDs deiner eigenen Creates. Jeder grafische Start kann ein
weiteres Fenster öffnen.

### Zeit und Steuerungsarten

Mit `tick warp 10` genau zehn weitere Haupt-Schedule-Durchläufe anfordern,
mit `tick warp 10 pace 20` deren Bearbeitung takten und einen laufenden Warp mit
`tick warp stop` beenden. Ein gestoppter Warp kann weniger Ticks ausgeführt haben;
prüfe das Ergebnis statt die angeforderte Zahl als Erfolg anzunehmen.

Danach führen die [sechs einfachen Steuerungswege](../examples/README.md) von
REPL und einzelnen CLI-Aufrufen über feste Scripts bis zu Python-Feedback und
einer eigenen Agent-Werkzeugschleife. Ein Agent ist ein externer Entscheider,
keine eingebaute Woodpecker-Funktion. Eigene Rust-Werkzeuge können stattdessen
`session::Session` direkt oder mit Feature `client` den lokalen Server verwenden;
der [API-Vertrag](api/target.md) beschreibt diese Schnittstellen.

### Maus, UI und Text

Virtuelle Pointer können positioniert oder relativ bewegt werden; Buttons und
Scrollen sind eigene Commands. Sie bewegen keinen OS-Cursor und brauchen keinen
nativen Fensterfokus. Bei eigenen Pointer-Filtern verwende die kontrollierte
Pointer-ID `woodpecker::session::VIRTUAL_POINTER_ID`, nicht fest die OS-Maus.

Für Bevy UI benötigt die Spielabhängigkeit `woodpecker/ui`. Für vollständige
Bevy-0.20-Textwidgets kommen in der Anwendung `bevy_ui_widgets` und
`TextInputPlugin` hinzu. Das [Textbeispiel](../examples/game_tests/text.py) zeigt
Widget-Suche, Fokus, Unicode-Eingabe und explizite Edit-/Sync-Ticks an Signalstation;
die [Eingabereferenz](api/usage.md) beschreibt die Voraussetzungen und Grenzen.

### Screenshots und Spielzustand zusammen beobachten

Nach mindestens einem ausgeführten Tick kannst du in der REPL anfordern:

```text
command {"command":"screenshot.capture","arguments":{"path":"screenshots/erster-blick.png"}}
```

Warte auf `completed`: Das PNG liegt relativ zum `artifact_dir` der Session und
die Aufnahme erzeugt keinen Simulationstick. Dein Spiel braucht einen Renderer,
eine Kamera und `woodpecker/screenshot`. Vor dem ersten Tick bzw. bei fehlender
Renderoberfläche kann der Command abgelehnt werden; ein tatsächlich schwarzes
Bild ist dagegen gültig. Automatische Bildvergleiche liefert Woodpecker nicht.
Details stehen in der [Screenshot-Anleitung](api/usage.md#screenshot).

### Aufzeichnen und wiedergeben

`recording start recordings/weg.jsonl` zeichnet die folgenden Spiel-Commands und
Outcomes auf; `recording stop` schließt die Datei. Vor Start und Stop müssen die
vorherigen Commands abgeschlossen sein. Replay führt Commands erneut aus,
setzt aber weder dein Spiel zurück noch prüft es automatisch alle alten Werte.

Nutze das [Fresh-Session-Rezept](../examples/game_tests/replay.py) für denselben
Startzustand, eine geschlossene Aufnahme und die Gegenprüfung in einer neuen
Session. Während Replay sind keine konkurrierenden Spielbefehle erlaubt außer
`replay stop`. Der [Datei-/Replay-Vertrag](api/usage.md#replay) erklärt Details.

### Fehlerberichte sammeln

Der gezeigte lokale Provider speichert beobachtete Fehlerberichte unter
`artifact_dir/reports`. Das Session-Plugin beobachtet Panics; ein beobachteter
Fehler bedeutet nicht zwangsläufig, dass der Spielprozess bereits beendet ist.
Gewöhnlicher stderr-Text allein ist kein Fehlerauslöser.

Für Tracing-Errors brauchst du **beides**: `report.tracing_errors = true` in der
TOML und den Callback `woodpecker::session::tracing_error_layer` in deinem
`LogPlugin`, bevor das Session-Plugin installiert wird. Die
[Report-Anleitung](api/usage.md#fehlerbeobachtung-und-snapshots) zeigt den Einbau.

Optional kann der GitHub-Provider Issues veröffentlichen. Das hat echte externe
Wirkung: eigenes Zielrepository, Anmeldung von `gh` und Inhalte zuerst prüfen.
Logs, Argumente, Inspect-Werte und Recordings können vertrauliche Daten enthalten.
Bleibe für den Einstieg beim lokalen Provider. Details zu Veröffentlichung,
Deduplizierung und lokalem Fallback: [Provider](api/usage.md#automatische-reports-im-server).

## 8. Wenn dein Spiel Assets oder eigene Ladephasen hat

Das Minimalspiel verwendet `manual`, um den Einbau ab dem ersten Tick zu zeigen.
Ein echtes Spiel ist bei `SessionReady` möglicherweise noch nicht spielbereit.
Dein Ladezustand bleibt Eigentum des Spiels: registriere ihn für Inspect und
prüfe ihn zwischen ausdrücklich begrenzten Vorbereitungsticks.

Für automatisches **technisches** Startup ist `startup.kind = "engine"` mit
`timeout_seconds` und `max_ticks` verfügbar. Dafür muss das Spiel zusätzlich
`woodpecker::session::startup::Assets` mit seinen tatsächlichen Startup-Handles
registrieren. Die [Signalstation-Integration](../test_app/src/woodpecker.rs) zeigt
diesen Adapter. Nur für eine wirklich prozedurale Szene ohne geladene Assets
ist `Assets(|_| Some(vec![]))` passend; eine unbekannte Assetliste ist `None`,
nicht absichtlich leer.

Engine-Startup führt normale, gezählte Anwendungsticks aus und meldet anschließend
`EngineReady`. Diese Ticks gehören zum Startzustand deiner späteren Prüfung.
`EngineReady` ist keine Aussage darüber, ob Mission, Level oder Menü spielbereit
sind. Eigene Spiellogik nicht durch erfundene Testzustände oder endloses
„weiter ticken, bis der Sollwert erscheint“ ersetzen.

## Häufige Stolperstellen

- **Normaler Start panikt wegen fehlendem Artefaktordner:** Session-Plugin nur im
  servergestarteten Prozess installieren, nicht die interne Umgebung nachbauen.
- **Session hängt oder reagiert nur bei Fensterfokus:** Plugin-Reihenfolge,
  Winit-Laufmodus, Builddiagnose und den tatsächlichen `automation`-Build prüfen.
- **`unknown_type_path` oder Resource nicht lesbar:** Typ registrieren,
  `ReflectResource`/`ReflectComponent` ableiten und Existenz/Typ-Pfad prüfen.
- **Input erfolgreich, Spiel unverändert:** Erst nach einem expliziten Tick
  erwarten; eigenes Input-Mapping, UI-Fokus und Ladezustand kontrollieren.
- **Persönliche Einstellungen verändert:** Der Spielcode verwendet noch seinen
  normalen Speicherort statt des Session-Workspaces.
- **CLI beendet, Fenster noch offen:** Clientende ist kein Sessionende. Die
  bestätigte eigene Session ausdrücklich stoppen und `Ended` beobachten.

Für genaue Command-Formate und Fehlercodes verwende die
[Bedienreferenz](api/usage.md); für die vollständige Produktabnahme die
[Testanleitung](../tests/README.md). Diese Anleitung zeigt die Integration und
Nutzung, nicht einen Nachweis, dass dein gesamtes Spiel korrekt ist.
