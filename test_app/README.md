# Signalstation

Eigenständige Bevy-Demo auf der Grundplatte aus dem [Levelplan](planning/grundplatte_levelplan.pdf). Acht Räume, Galerien auf drei Metern Höhe, sechs Treppen und Brücken verbinden die Aufgaben. Unter den Galerien bleibt der Boden begehbar. Drei versteckte Bereiche bieten zusätzliche Verbindungen und Fundstücke. Der Sprungparcours in der zentralen Halle ist freiwillig.

## Spielen

```sh
cargo run --manifest-path test_app/Cargo.toml
```

Die Demo verwendet Bevy 0.19.1 und Rust 1.97.1 oder neuer. Sie hat einen eigenen Cargo-Workspace und benötigt Woodpecker nicht. Der Asset-Pfad funktioniert unabhängig vom Arbeitsverzeichnis. Für einen optimierten Build `--release` ergänzen.

Der Auftrag ist, drei Energiezellen zu beschaffen und das Signal in H zu starten. Die Aufgaben lassen sich in beliebiger Reihenfolge erledigen:

- In Werkhalle B und Lagerhalle D jeweils ein Bauteil aufnehmen. Der Hebel in D verschiebt das Regal und gibt das zweite Bauteil frei. An der Werkbank in B entsteht daraus eine Energiezelle.
- Beide Bewässerungsventile im Gewächshaus E öffnen. Die Zelle liegt anschließend am Verteiler bereit.
- Drei Tafeln im Archiv F scannen und die Zelle am Terminal abholen.
- Die Zellen mit E aufnehmen und unten in H am Generator einsetzen. Man kann mehrere Zellen tragen oder einzeln liefern. Danach die Treppe zum Signalbalkon nehmen und mit E abschließen.

24 Münzen und sechs Baupläne sind zusätzliche Sammelziele. Ein Bauplan liegt in der Truhe im Übungsraum G. Um sie zu entriegeln, jede Scheibe aus mindestens sechs Metern Entfernung treffen. Die übrigen Baupläne liegen in den Nebenbereichen und auf den Galerien. Eine Münze erfordert den Sprungparcours. Nach dem Signal bleiben alle Räume begehbar.

Der Wartungsgang G1 verbindet A mit H. G3 führt hinter der Werkhalle ins Archiv. Oben verbindet der goldene Weg durch G2 das Archiv mit dem Gewächshaus. Der Checkpoint in C lässt sich mit E aktivieren.

## Steuerung

| Eingabe | Aktion |
| --- | --- |
| Pfeiltasten | Figur bewegen |
| Linksklick auf Boden, Treppe oder Galerie | Per A* zum angeklickten Stockwerk laufen |
| V | Übersicht / Egoperspektive wechseln |
| WASD + Maus | In FPV laufen und umsehen |
| G | Gesamtkarte / Raumübersicht |
| E | Angezeigte Station bedienen oder Gegenstand aufnehmen |
| Leertaste | Springen |
| Umschalt | Sprinten |
| Backspace | Zum aktivierten Checkpoint oder Start zurückkehren; Fortschritt bleibt erhalten |
| Linksklick in FPV | Blaster abfeuern |
| Esc in FPV | Maus freigeben; Klick ins Level bindet sie wieder |
| M | Einstellungen öffnen und Spiel pausieren |
| R | Aufgaben, Sammelobjekte und Figur zurücksetzen |
| Tab / Umschalt+Tab | UI-Fokus wechseln |

Manuelle Bewegung bricht eine Klickroute ab. Treppen werden automatisch bestiegen. Die Figur besitzt eine Kollisionsbox von 1,2 × 1,2 × 1,82 Metern. Auch ihre Arme und der Kopf bleiben innerhalb dieser Box. Wände, Ecken, Möbel, Zielständer und Galeriekanten halten sie auf. Klickwege verwenden dieselbe Körpergröße und berücksichtigen das Stockwerk; Sprünge steuert man selbst. Der Button „Zum nächsten Auftrag“ führt zum nächsten noch offenen Aufgabenziel. „Zum Schießstand“ führt nach G.

## Zielübung, Eingabe und Einstellungen

Der Schießstand in G ist immer bereit. Mit V in FPV wechseln und mit Linksklick schießen. Jeder Treffer zählt erst ab sechs Metern Bodenabstand zur jeweiligen Scheibe. Blaue Linien markieren diesen Abstand vor den Zielen; die Anzeige nennt den Abstand zum nächsten stehenden Ziel. Zu nahe Treffer lassen die Scheibe stehen. Gültige Treffer zerbrechen sie dauerhaft. Die Mitte gibt 50 Punkte, die weiteren Ringe 25 oder 10. Sobald alle Scheiben gültig getroffen wurden, öffnet E die Bauplan-Truhe hinten rechts. Es gibt kein Zeitlimit und keine Startinteraktion. R setzt die Ziele zusammen mit dem Level zurück.

Brückenstützen und Wände blockieren Schüsse; mit WASD wechselst du zwischen den Schießbahnen. Die Übung ist für den Hauptauftrag nicht erforderlich. Menü, freie Maus, Texteingabe und ein inaktives Fenster sperren Schüsse.

## Ressourcen und Stationszeichen

Die Leiste am oberen Bildschirmrand zeigt den gesamten aktuellen Besitz, ohne Gesamtmengen oder Raumzähler. Gebaute Zellen verbrauchen Bauteile; eingesetzte Zellen verlassen das Inventar.

| Zeichen | Gegenstand und zugehörige Stationen |
| --- | --- |
| Goldene Münze | Münzen zum Einsammeln |
| Oranges Zahnrad | Bauteile, Lagerhebel und Eingang der Werkbank |
| Grüne Batterie | Energiezellen, Werkbank-Ausgang, Ventile, Archivtafeln und Generator |
| Blaues Blatt | Baupläne, Zielscheiben und Belohnungstruhe |

Die Form bleibt auch ohne Farberkennung unterscheidbar. Markierungen an Ventilen, Archivtafeln, Werkbank und Generator leuchten nach Abschluss der jeweiligen Aufgabe stärker. Die UI-Symbole lassen sich mit `python3 test_app/planning/create_resource_icons.py` neu erzeugen. Die räumlichen Zeichen stehen in `src/items.rs`.

Das Namensfeld unterstützt Unicode und die Systemzwischenablage. Während der Eingabe bleiben Spieltasten gesperrt. Enter oder Esc beendet die Eingabe. Das Einstellungsmenü bietet Maus-Empfindlichkeit, Sichtfeld, Lauftempo, Schatten, Routenanzeige und eine umkehrbare vertikale Mausachse. Name und Einstellungen werden in `test_app/settings.json` gespeichert.

## Level bearbeiten

Die [Blender-Datei](planning/groundplate/groundplate_concept.blend), der [PDF-Plan](planning/grundplatte_levelplan.pdf) und das [Style-Handoff](planning/STYLE_HANDOFF.md) beschreiben den Entwurf. Das Spiel lädt `asset/custom/courtyard/courtyard.glb`; Blender muss zum Spielen nicht laufen.

Im Blender-Outliner lassen sich die folgenden Collections über das Augensymbol unabhängig ausblenden. Das Kamerasymbol steuert die Sichtbarkeit im Renderbild.

| Collection | Inhalt |
| --- | --- |
| `01 Notwendig` | Grundplatte, Wände, Treppen, Brücken, Geländer, Parcours, Aufgabenstationen und Sammelobjekte |
| `02 Deko` | Kisten, Bänke, nicht bedienbare Maschinen, Bücherregale, Ordner, Schränke und Pflanzen samt Kübeln |
| `03 Planung` | Raumbeschriftungen, Größenvergleichsfigur und die alte Startkonsole des Schießstands |
| `04 Kamera und Licht` | Kamera und Renderbeleuchtung |

Für die reine Spielstruktur `02 Deko` und `03 Planung` ausblenden. Für die Dekoration allein `01 Notwendig` und `03 Planung` ausblenden. Funktionsobjekte bleiben samt Details zusammen, etwa der Generator mit seinen Zellensockeln. Das versteckende Regal an G2 gehört ebenfalls zur Spielstruktur. Dekorative Möbel behalten ihre Kollision im Spiel. Die Aufteilung in [scene_collections.py](planning/groundplate/scene_collections.py) gilt auch beim Neuerstellen mit `create_model.py`.

```sh
blender --background test_app/planning/groundplate/groundplate_concept.blend \
  --python-exit-code 1 --python test_app/blender/export_level.py
```

Unter macOS kann `blender` durch `/Applications/Blender.app/Contents/MacOS/Blender` ersetzt werden. Der Export verändert die gespeicherte Blend-Datei nicht. Er entfernt die statischen Platzhalter für Figur, Sammelobjekte und Zielscheiben, die das Spiel selbst erzeugt. Die Räume, Treppen, Galerien und Fundpositionen kommen aus `planning/groundplate/layout.py`; Hindernisse aus benannten Objekten der Blend-Datei. Geometrieänderungen an den Laufwegen müssen deshalb in beiden Quellen übereinstimmen. `layout.json` enthält die exportierten Kollisions- und Navigationsdaten und wird beim Kompilieren eingebunden.

Der Export enthält auch ausgeblendete Collections. Die Sichtbarkeit in Blender dient nur der Bearbeitung und den Planungsrenders; sie entfernt keine Objekte oder Kollisionen aus der Demo.

Der bearbeitete Blaster bleibt in [blender/training_blaster.blend](blender/training_blaster.blend). Nach Änderungen separat exportieren:

```sh
blender --background test_app/blender/training_blaster.blend \
  --python-exit-code 1 --python test_app/blender/export_blaster.py
```

## Prüfen

```sh
cargo test --manifest-path test_app/Cargo.toml
cargo run --manifest-path test_app/Cargo.toml -- --smoke-test --offscreen
```

Die Tests laufen mit der tatsächlichen Levelgeometrie und Bewegungsphysik. Sie prüfen Aufgaben und Sammelwege, alle Treppen, den Sprungparcours, getrennte Laufwege unter und auf einer Galerie, Kollision mit deren Unterseite, Blickrichtung, den sichtbaren Körper innerhalb der Kollisionsbox, Inventarverbrauch und den Mindestabstand jedes Zieltreffers.

Der gerenderte Test prüft Pfeiltasten, Klickwege, Unicode-Eingabe, Einstellungen und FPV. Anschließend erledigt er den Hauptauftrag, sammelt die Baupläne, trifft alle drei Scheiben und öffnet die Truhe. Er prüft den Checkpoint, die beiden Galerieebenen und den Neustart. Mit `--smoke-test --smoke-range --offscreen` lässt sich die Zielübung separat prüfen, einschließlich verdeckter Schüsse, abgewiesener Nahschüsse und seitlicher Bewegung. Bilder und separate Testeinstellungen landen in `test_app/artifacts/`. Ohne `--offscreen` öffnet der Test ein sichtbares Fenster, das den Eingabefokus behalten muss.

## Code und Assets

`blueprint.rs` lädt die exportierten Raumdaten. `navigation.rs` verwaltet Kollision und Wege, `player.rs` Bewegung und Sprünge. `expedition.rs` enthält Aufgaben und Inventar, `course.rs` deren Darstellung. `items.rs` verbindet Inventar, Farben und Objektformen. `game.rs` koordiniert Befehle; Kamera, Münzen, Zielübung, Oberfläche und Einstellungen haben eigene Module.

Die KayKit-Modelle stammen aus **KayKit: Prototype Bits** von Kay Lousberg, siehe [CC0-Lizenz](asset/License.txt). Blaster, Level und Stationsmodelle wurden für diese Demo erstellt. Die Schrift Barlow liegt mit ihrer [SIL Open Font License](asset/fonts/OFL.txt) unter `asset/fonts/`.
