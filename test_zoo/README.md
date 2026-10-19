# Modell-Zoo

Eigenständige Bevy-App zum Prüfen der 13 Objektmodelle aus [Signalstation](../test_app/README.md). `test_zoo` ist ein eigenes Workspace-Paket neben `test_app`; Objektdefinitionen, Modellaufbau und Assets kommen aus dessen Library. Es werden keine Modelle kopiert und keine Spielsysteme installiert.

## Starten

Vom Repository-Root:

```sh
cargo run -p test_zoo
```

Alternativ funktioniert `cargo run --manifest-path test_zoo/Cargo.toml`. Die Assets werden unabhängig vom Arbeitsverzeichnis aus `test_app/asset/` geladen. Für Modell-Dateien gilt der gemeinsame [Asset-Standard von `test_app`](../test_app/README.md#asset-formate): glTF mit externen BIN-Dateien und gemeinsam genutzten PNG-Texturen. Die Objektmodelle des Zoos werden weiterhin direkt im gemeinsamen Rust-Code aufgebaut.

## Steuerung

| Eingabe | Aktion |
| --- | --- |
| Pfeil links / rechts | Modell auswählen |
| O | Alle Modelle nebeneinander zeigen |
| Linke Maustaste im Orbit-Bereich ziehen | Um das Modell drehen |
| Mausrad über einer Ansicht | Diese Ansicht zoomen |
| Klick auf Ansichtstitel / F1–F4 | Oben / Front / Rechts / Orbit einzeln anzeigen; erneut drücken für vier Ansichten |
| „Vier Ansichten“ / F5 | Alle vier Detailansichten zeigen |
| 1 / 2 / 3 | Gesperrt / bereit / benutzt beziehungsweise abgeholt |
| R | Ansicht zurücksetzen |
| Esc | Zoo schließen |

Die Detailansicht zeigt vier durch eine horizontale und eine vertikale Linie getrennte Kamerabereiche: **Oben** und **Front** in der oberen Reihe, **Rechts** und **Orbit** darunter. Oben, Front und Rechts blicken fest entlang der Modellachsen. Im Orbit-Bereich lässt sich die Kamera horizontal und vertikal vollständig um das Modell drehen, auch unter das Modell und über beide Pole hinweg. Jede Ansicht hat ihren eigenen Zoom; beim Umschalten zur Einzelansicht bleiben Zoom und Orbit-Ausrichtung erhalten. Die Gesamtübersicht mit O zeigt alle Modelle in einer gemeinsamen Orbit-Ansicht.

Der Boden ist ein transparenter Schattenfänger: Beleuchtete Bereiche sind unsichtbar, nur der von den Modellen geworfene Schatten wird als halbtransparente Abdunklung dargestellt. Material und eingebetteter Shader liegen unter `src/ground.rs` und `src/ground/shadow_catcher.wesl`.

Gezeigt werden Auftragstafel, Montagepresse, Lagerhebel, Ventil, Archivscanner, Generatorbedienung, Sendeschalter, Checkpoint, Belohnungstruhe und vier Ablagevarianten. Die Zustandsvorschau verändert keinen Spielstand.

Der gemeinsame Modellaufbau liegt in `test_app/src/level/objects/`, aufgeteilt nach Bedienobjekten. Der Zoo wählt über `ModelKind` ein Modell, baut es mit `spawn_model` auf und setzt dessen Vorschau über `VisualState`. Er benötigt keinen `Mission`-Zustand. Geometrie und Zustandsdarstellung bleiben dieselben wie im Spiel.

## Bildexport

```sh
cargo run -p test_zoo -- \
  --zoo-capture test_zoo/artifacts/after -NSAppSleepDisabled YES
python3 test_zoo/contact_sheet.py test_zoo/artifacts/after
```

Der Export verwendet Einzelansichten, läuft ohne sichtbares Fenster und erzeugt 67 Bilder: je Modell Vorderseite, Rückseite, Seite, benutzt und gesperrt, dazu zwei Übersichten. `-NSAppSleepDisabled YES` verhindert unter macOS die Drosselung des Hintergrundprozesses; auf anderen Plattformen ist es nicht nötig.

`contact_sheet.py` benötigt Pillow und erzeugt `all-models.jpg` sowie eine Vergleichstafel pro Modell. `test_zoo/artifacts/` ist von der Versionsverwaltung ausgeschlossen. Frühere Aufnahmen unter `test_app/artifacts/zoo/` bleiben unverändert und können ebenfalls als Eingabeverzeichnis verwendet werden.

## Prüfen

```sh
cargo check -p test_app -p test_zoo --all-targets --all-features
cargo test -p test_app -p test_zoo --all-features
```

Die Zoo-Tests prüfen den gemeinsamen Asset-Pfad, den Aufbau der 13 Modelle, die Zustandsvorschau, Vierer- und Einzelansichten, Tasten und Ansichtsschalter, Mauszuordnung, Fenstergrößen und DPI-Skalierung sowie den Einzelansichts-Bildexport ohne Fenster oder Renderer. Der Schattenfänger wird zusätzlich mit den Bevy-Lichtmodulen zu WGSL übersetzt und mit Naga validiert; dieser Test verwendet weder ein Fenster noch ein GPU-Backend.
