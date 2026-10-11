# Münzparcours und Zieltraining

Eigenständige Bevy-Demo mit den KayKit-Modellen aus `asset/`. Sammle die sechs Münzen und kehre zur grünen Startplattform zurück. Der Timer stoppt nach der erfolgreichen Runde.

Im Norden liegt ein Schießstand mit drei Bahnen. Der Trainingsblaster wurde in Blender passend zu den kantigen KayKit-Modellen gebaut. Beide Spielbereiche bleiben auch nach einer abgeschlossenen Runde zugänglich.

## Starten

Aus dem Repository-Verzeichnis:

```sh
cargo run --manifest-path test_app/Cargo.toml
```

Oder direkt aus `test_app` mit `cargo run`. Der Asset-Pfad hängt nicht vom aktuellen Arbeitsverzeichnis ab. Für einen optimierten Build `cargo run --release` verwenden. Der erste Build kompiliert auch die 3D-Engine und dauert entsprechend länger.

Die Demo verwendet Bevy 0.19.1 und benötigt Rust 1.97.1 oder neuer sowie eine GPU mit Unterstützung für Bevy. Sie hat keine Abhängigkeit von Woodpecker und ist ein eigener Cargo-Workspace. Bevy ist auf 0.19.1 festgelegt, da die benötigten 3D-Pakete für 0.20.0 beim Einrichten nicht vollständig verfügbar waren.

## Steuerung

| Eingabe | Aktion |
| --- | --- |
| Pfeiltasten | Figur in der Übersicht bewegen |
| Linksklick auf freien Boden | Per A*-Pathfinding zum Ziel laufen |
| V oder „Ansicht wechseln“ | Zwischen Übersicht und Egoperspektive wechseln |
| WASD + Maus | In FPV laufen und umsehen |
| „Zum Schießstand“ | Automatisch zum Eingang des Schießstands laufen |
| E am Schießstand | 30-Sekunden-Zielübung in FPV starten oder neu starten |
| Linksklick in der Zielübung | Einen Schuss abgeben |
| Umschalt | Sprinten |
| Esc in FPV | Maus freigeben |
| Klick ins Level bei freier FPV-Maus | Maus wieder binden |
| M oder „Einstellungen“ | Spiel pausieren und Einstellungen öffnen |
| Esc bei freier Maus | Einstellungen öffnen oder schließen |
| R oder „Neue Runde“ | Münzen, Figur und Timer zurücksetzen |
| Tab / Umschalt+Tab | UI-Fokus wechseln |
| Enter / Leertaste | Fokussierten Button betätigen |

Die Pfeiltasten funktionieren auch in FPV relativ zur Blickrichtung. Manuelle Bewegung bricht die automatische Route ab. Wände, Kisten, Fässer und der Rand des Levels blockieren die Figur. Die Navigation berücksichtigt den Körperradius und verhindert diagonale Abkürzungen durch Hindernisse.

## Zieltraining

Klicke auf **„Zum Schießstand“** und drücke dort **E**. Ziele mit dem Fadenkreuz auf eine Scheibe und schieße mit der linken Maustaste. Treffer in der Mitte geben 50, im mittleren Bereich 25 und außen 10 Punkte. Die Scheiben zerbrechen und erscheinen nach 1,2 Sekunden erneut. Die Trennwände blockieren Schüsse; mit WASD kannst du zwischen den Bahnen wechseln.

Das HUD zeigt Restzeit, Punkte, Trefferquote und den Rekord der laufenden Sitzung. Nach 30 Sekunden endet die Wertung; E startet die nächste Runde. Das Menü, ein inaktives Fenster, die Übersicht und die freigegebene Maus pausieren die Übung. Beim Verlassen des Schießstands in FPV endet sie. Ein Klick zum Binden der Maus oder Fortsetzen aus dem Menü löst keinen Schuss aus.

## Texteingabe und Einstellungen

Das Namensfeld rechts oben unterstützt Unicode, Textauswahl, Bearbeitung und die Systemzwischenablage. Enter oder Esc beendet die Eingabe. Während der Texteingabe bewegen die Tasten die Figur nicht und lösen keine Spielkürzel aus. Im Spiel erscheint der Name neben dem Timer und in der Erfolgsmeldung. In FPV blendet sich das Bedienfeld aus, solange die Maus gebunden ist; Esc macht es wieder zugänglich.

Im Menü lassen sich Maus-Empfindlichkeit, FPV-Sichtfeld, Lauftempo, Schatten, Routenanzeige und die vertikale Mausachse ändern. Einstellungen und Name werden automatisch in `test_app/settings.json` gespeichert. „Standardwerte“ setzt die Einstellungen zurück und behält den Namen bei. Das Menü und ein inaktives Fenster pausieren Bewegung und Timer. In FPV pausiert auch die freigegebene Maus die Bewegung.

## Prüfen

```sh
cargo test --manifest-path test_app/Cargo.toml
cargo run --manifest-path test_app/Cargo.toml -- --smoke-test
# Bildprüfung auch bei gesperrtem Desktop:
cargo run --manifest-path test_app/Cargo.toml -- --smoke-test --offscreen
```

Die Unit-Tests prüfen Umwege, unerreichbare Ziele, enge diagonale Lücken, Kollision bei großen Bewegungsschritten, die Blickrichtung der Figur, die Erreichbarkeit aller Münzen und Schießbahnen sowie die Trefferwertung.

Der gerenderte Integrationstest öffnet das Spielfenster, führt synthetische Eingaben durch dieselben Spielsysteme aus und beendet sich nach einem vollständigen Durchlauf. Das Fenster muss dabei den Eingabefokus behalten. Geprüft werden Asset-Laden, Pfeiltasten, Klick-Pathfinding, Unicode-Texteingabe, Eingabefokus, das Einstellungsmenü einschließlich Speicherung, FPV mit WASD und Maus, Neustart und Spielabschluss. Anschließend prüft er den Weg zum Schießstand, Treffer auf den geladenen glTF-Scheiben, Punkte, Trümmer, Wiedererscheinen der Ziele, durch Wände blockierte Schüsse, Pause, Zeitablauf und eine neue Übungsrunde. Screenshots und separate Testeinstellungen landen in `test_app/artifacts/`; persönliche Einstellungen bleiben erhalten. Ein Fehler beendet den Test mit einem Fehlercode.

Mit `--offscreen` bleibt das Testfenster verborgen. Derselbe Test rendert in eine GPU-Textur und simuliert auch den Fensterfokus. So entstehen vollständige Screenshots, wenn macOS für ein gesperrtes oder verdecktes Fenster keine Bilder bereitstellt. Der normale Spielstart nutzt weiterhin den tatsächlichen Fensterfokus.

## Dateien und Assets

- `src/game.rs`: Tastenkürzel und Koordination übergreifender Aktionen wie Neustart und Ansichtswechsel.
- `src/player.rs`: Spielerposition, Ausrichtung, Laufroute, Bewegung und Figurendarstellung.
- `src/camera.rs`: Perspektive, Maussteuerung, Mausbindung und Kameradarstellung.
- `src/coins.rs`: Münzrunde, Timer, Abschluss und Münzendarstellung.
- `src/navigation.rs`: A*, Routenvereinfachung und Kollision.
- `src/level.rs`: Level-Aufbau, Ladezustand, Beleuchtung und Münzpositionen.
- `src/ui.rs`: UI-Zustand, Statusmeldung, HUD, Namensfeld und Einstellungsmenü.
- `src/settings.rs`: Laden und Speichern der Einstellungen.
- `src/shooting.rs`: Blaster, Raycast-Treffer, Zielscheiben und Übungsrunden.
- `src/smoke.rs`: Gerenderter Integrationstest.
- `blender/training_blaster.blend`: Bearbeitbares Blaster-Modell mit Vorschaukamera und Licht.
- `blender/create_blaster.py`: Blender-Skript zum erneuten Erzeugen und Exportieren des Modells.
- `asset/custom/training_blaster.glb`: Export für das Spiel; Blender muss zum Spielen nicht laufen.

Die vorhandenen Modelle und Texturen stammen aus **KayKit: Prototype Bits** von Kay Lousberg. Die mitgelieferte CC0-Lizenz steht in [`asset/License.txt`](asset/License.txt). Der Blaster unter `asset/custom/` wurde für diese Demo erstellt.

Für die Oberfläche liegt die Schrift Barlow unter `asset/fonts/`. Sie stammt aus dem [Google-Fonts-Repository](https://github.com/google/fonts/tree/main/ofl/barlow) und enthält auch deutsche Umlaute. Die SIL Open Font License liegt als [`asset/fonts/OFL.txt`](asset/fonts/OFL.txt) bei.
