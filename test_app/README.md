# Signalstation

Eigenständige Bevy-Demo auf der Grundplatte aus dem [Levelplan](planning/grundplatte_levelplan.pdf). Acht Räume, Galerien auf drei Metern Höhe, sechs Treppen und Brücken verbinden die Aufgaben. Unter den Galerien bleibt der Boden begehbar. Drei versteckte Bereiche bieten zusätzliche Verbindungen und Fundstücke. Der Sprungparcours in der zentralen Halle ist freiwillig.

## Spielen

```sh
cargo run --manifest-path test_app/Cargo.toml
```

Die Demo verwendet Bevy 0.20.0 und Rust 1.97.1 oder neuer. Sie gehört zum gemeinsamen Cargo-Workspace; Woodpecker wird nur mit dem Feature `automation` eingebunden. Der Asset-Pfad funktioniert unabhängig vom Arbeitsverzeichnis. Für einen optimierten Build `--release` ergänzen.

Der Auftrag ist, drei Energiezellen zu beschaffen und das Signal in H zu starten. Die Aufgaben lassen sich in beliebiger Reihenfolge erledigen:

- In Werkhalle B und Lagerhalle D jeweils ein Bauteil vom Ablagetisch nehmen. Der Hebel in D verschiebt das Regal und gibt das zweite Bauteil frei. Am orangefarbenen Montageschalter in B entsteht daraus eine Energiezelle, die anschließend auf dem benachbarten Tisch liegt.
- Beide Bewässerungsventile im Gewächshaus E öffnen. Die Zelle liegt anschließend am Verteiler bereit.
- Drei Tafeln auf den Lesepulten im Archiv F scannen und die Zelle vom Ausgabetisch nehmen.
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
| E | Angezeigtes Objekt bedienen oder Gegenstand aufnehmen; in FPV das Objekt anvisieren |
| Leertaste | Springen |
| Umschalt | Sprinten |
| Backspace | Zum aktivierten Checkpoint oder Start zurückkehren; Fortschritt bleibt erhalten |
| Linksklick in FPV | Blaster abfeuern |
| Esc in FPV | Maus freigeben; Klick ins Level bindet sie wieder |
| M | Einstellungen öffnen und Spiel pausieren |
| R | Aufgaben, Sammelobjekte und Figur zurücksetzen |
| Tab / Umschalt+Tab | UI-Fokus wechseln |

Manuelle Bewegung bricht eine Klickroute ab. Treppen werden automatisch bestiegen. Die Figur besitzt eine Kollisionsbox von 1,2 × 1,2 × 1,82 Metern. Auch ihre Arme und der Kopf bleiben innerhalb dieser Box. Wände, Ecken, Möbel, Zielständer und Galeriekanten halten sie auf. Klickwege verwenden dieselbe Körpergröße und berücksichtigen das Stockwerk; Sprünge steuert man selbst. Der Button „Zum nächsten Auftrag“ führt zum nächsten noch offenen Aufgabenziel. „Zum Schießstand“ führt nach G.

Zwischen Laufen und Sprinten wechselst du nur mit Bodenkontakt. Beim Springen und Fallen bleibt der zuletzt am Boden gewählte Zustand bestehen, auch wenn du Umschalt drückst oder loslässt. Nach der Landung zählt wieder der aktuelle Zustand der Umschalttaste. Die Richtungssteuerung in der Luft bleibt möglich.

Die Laufbewegung hebt und senkt `Dummy_Body` relativ zu seiner importierten Ruheposition – sowohl in der Übersicht als auch in der Egoperspektive, bei Pfeiltasten und Klickwegen. Fußteil und Kollisionskörper wippen nicht mit. Beim Springen und Fallen bleibt der Oberkörper in seiner lokalen Ruheposition; das Laufwippen setzt erst mit Bodenkontakt wieder ein. Kopf und Arme folgen als Kinder des Körpers; die Kamera hat keine eigene Laufanimation. In der Egoperspektive sitzt die Kamera am festen Augenpunkt `EYE_POSITION = (0, 0.65, 0)` im lokalen, unskalierten Koordinatensystem von `Dummy_Head` (`src/player/camera.rs`). Sie übernimmt dessen Bewegung über die Objekthierarchie; nur der Mausblick verändert ihre lokale Drehung. Blaster, Schüsse und Interaktionen verwenden dieselbe Weltpose. Beim Wechsel zur Übersicht wird die Kamera vom Kopf gelöst. Die Kollisionsgeometrie bleibt davon unabhängig.

## Stationen bedienen

Interaktionen funktionieren von allen zugänglichen Seiten des Objekts. In FPV muss es in Reichweite und im Fadenkreuz liegen. Der konkrete E-Hinweis erscheint direkt darunter, sobald die Aktion möglich ist. In der Übersicht erscheint die Aktion des nächsten erreichbaren Objekts unten in der Mitte. Wände, andere Stockwerke, freie Maus, Texteingabe und das Menü verhindern die Bedienung. Interaktionsflächen haben keine sichtbaren Bodenringe.

Jedes Bedienobjekt hat eine Funktion: Der Lagerhebel verschiebt das Regal, das Handrad öffnet ein Ventil, das Lesepult scannt eine Archivtafel und der Montageschalter baut eine Zelle. Die Abholung erfolgt am separaten Tisch. Bauteile und Zellen liegen auf der Tischplatte und verschwinden beim Einsammeln. Die Archivtafeln sind weiß und unterscheiden sich von den blauen Bauplänen fürs Inventar.

Benutzte Stationen zeigen dauerhaft einen grünen Status mit Häkchen und einer Beschriftung wie „Tafel ist gescannt“. Hebel und Handräder ändern ihre Stellung; eingelesene Tafeln verschwinden vom Lesepult. Die Belohnungstruhe öffnet sich nach der Zielübung, zeigt den Bauplan und bleibt nach dessen Entnahme leer und geöffnet. R stellt auch diese Zustände zurück.

## Zielübung, Eingabe und Einstellungen

Der Schießstand in G ist immer bereit. Mit V in FPV wechseln und mit Linksklick schießen. Jeder Treffer zählt erst ab sechs Metern Bodenabstand zur jeweiligen Scheibe. Blaue Linien markieren diesen Abstand vor den Zielen; die Anzeige nennt den Abstand zum nächsten stehenden Ziel. Zu nahe Treffer lassen die Scheibe stehen. Gültige Treffer zerbrechen sie dauerhaft. Die Mitte gibt 50 Punkte, die weiteren Ringe 25 oder 10. Sobald alle Scheiben gültig getroffen wurden, öffnet sich die Truhe hinten rechts. Mit E nimmst du den sichtbaren Bauplan heraus. Es gibt kein Zeitlimit und keine Startinteraktion. R setzt die Ziele zusammen mit dem Level zurück.

Brückenstützen und Wände blockieren Schüsse; mit WASD wechselst du zwischen den Schießbahnen. Die Übung ist für den Hauptauftrag nicht erforderlich. Menü, freie Maus, Texteingabe und ein inaktives Fenster sperren Schüsse.

## Ressourcen und Stationszeichen

Die Leiste am oberen Bildschirmrand zeigt den gesamten aktuellen Besitz, ohne Gesamtmengen oder Raumzähler. Gebaute Zellen verbrauchen Bauteile; eingesetzte Zellen verlassen das Inventar.

| Zeichen | Gegenstand und zugehörige Orte |
| --- | --- |
| Goldene Münze | Münzen zum Einsammeln |
| Oranges Zahnrad | Bauteile auf kleinen Ablagetischen; orangefarbener Lagerhebel und Montageschalter |
| Grüne Batterie | Energiezellen auf grünen Ausgabetischen und in den Sockeln des Generators |
| Blaues Blatt | Baupläne im Level und in der blauen Truhe der Zielübung |

Die Form bleibt auch ohne Farberkennung unterscheidbar. Stationen tragen Beschriftungen für Zweck, Voraussetzungen und Erledigt-Zustand. Die UI-Symbole lassen sich mit `python3 test_app/planning/create_resource_icons.py` und die Stationsschilder mit `python3 test_app/planning/create_station_labels.py` neu erzeugen. Die räumlichen Gegenstände stehen in `src/items.rs`.

Das Namensfeld unterstützt Unicode und die Systemzwischenablage. Während der Eingabe bleiben Spieltasten gesperrt. Enter oder Esc beendet die Eingabe. Das Einstellungsmenü bietet Maus-Empfindlichkeit, Sichtfeld, Lauftempo, Schatten, Routenanzeige und eine umkehrbare vertikale Mausachse. Name und Einstellungen werden in `test_app/settings.json` gespeichert.

## Level bearbeiten

### Modell-Zoo

Der [Modell-Zoo](../test_zoo/README.md) ist eine eigene App neben `test_app/`. Er verwendet die Stationsmodelle und Assets der Spiel-Library, ohne das Spiel zu starten.

```sh
cargo run -p test_zoo
```

Steuerung, Zustandsvorschau und Bildexport sind in [test_zoo/README.md](../test_zoo/README.md) beschrieben.

### Blender und Level-Export

Die [Blender-Datei](planning/groundplate/groundplate_concept.blend), der [PDF-Plan](planning/grundplatte_levelplan.pdf) und das [Style-Handoff](planning/STYLE_HANDOFF.md) beschreiben den Entwurf. Das Spiel lädt `asset/meshes/courtyard/courtyard.gltf` samt `courtyard.bin`; Blender muss zum Spielen nicht laufen.

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

Unter macOS kann `blender` durch `/Applications/Blender.app/Contents/MacOS/Blender` ersetzt werden. Der Export verändert die gespeicherte Blend-Datei nicht. Er entfernt die statischen Platzhalter für Figur, Sammelobjekte und Zielscheiben, die das Spiel selbst erzeugt. Die Räume, Treppen, Galerien und Fundpositionen kommen aus `planning/groundplate/layout.py`; Hindernisse aus benannten Objekten der Blend-Datei. Geometrieänderungen an den Laufwegen müssen deshalb in beiden Quellen übereinstimmen. Der Exporter schreibt Räume, begehbare Flächen, Hindernisse und Fundpositionen in `courtyard.gltf` unter `extras.signalstation.layout`. `src/level/layout.rs` liest diese Metadaten beim Initialisieren aus der beim Kompilieren eingebundenen glTF-Datei; Navigation und Spiellogik verwenden weiterhin ihre eigenen Rust-Module.

Der Export enthält auch ausgeblendete Collections. Die Sichtbarkeit in Blender dient nur der Bearbeitung und den Planungsrenders; sie entfernt keine Objekte oder Kollisionen aus der Demo.

Die spielbaren Bedienobjekte aus `src/expedition/stations.rs` ersetzen die einfachen Konsolen des Blender-Entwurfs zur Laufzeit. Ihre Kollisionskörper und die unsichtbaren Interaktionsbereiche verwenden die Definitionen aus `src/expedition/stations/targets.rs`. `src/expedition/interactions.rs` prüft Blickrichtung, Reichweite und freie Sicht. Die alten Konsolen und Bodenringe bleiben im Spiel ausgeblendet.

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

Der gezielte Aufruf `cargo run --manifest-path test_app/Cargo.toml -- --smoke-test --smoke-stations --offscreen` prüft Bedienung von hinten, Fadenkreuz-Hinweise, Montage und getrennte Abholung, das Verschwinden eingesammelter Gegenstände, erledigte Scanner und Ventile sowie den Neustart. Die Unit-Tests decken alle vier Seiten eines Scanners, Blickrichtung, Reichweite, Sichtschutz und Stockwerke ab.

Unter macOS beim unsichtbaren Test `-NSAppSleepDisabled YES` hinter `--offscreen` ergänzen, damit App Nap den Hintergrundprozess nicht drosselt. Die Option gilt nur für diesen Prozess.

Der gerenderte Test prüft Pfeiltasten, Klickwege, Unicode-Eingabe, Einstellungen und FPV. Anschließend erledigt er den Hauptauftrag, sammelt die Baupläne, trifft alle drei Scheiben und öffnet die Truhe. Er prüft den Checkpoint, die beiden Galerieebenen und den Neustart. Mit `--smoke-test --smoke-range --offscreen` lässt sich die Zielübung separat prüfen, einschließlich verdeckter Schüsse, abgewiesener Nahschüsse und seitlicher Bewegung. Bilder und separate Testeinstellungen landen in `test_app/artifacts/`. Ohne `--offscreen` öffnet der Test ein sichtbares Fenster, das den Eingabefokus behalten muss.

## Code und Assets

### Asset-Formate

Alle Modelldateien liegen unter `asset/meshes/`: KayKit-Modelle in `kaykit/`, Stationsmodelle in `station/`, der Blaster in `blaster/` und der Level-Export mit seinen glTF-Metadaten in `courtyard/`.

Für Modell-Dateien beider Apps gilt ein gemeinsamer Standard: `.gltf` für Szene und Materialien, externe `.bin`-Dateien für Geometrie und externe `.png`-Dateien für Texturen. Die 72 KayKit-Modelle unter `asset/meshes/kaykit/` teilen genau eine `prototypebits_texture.png`. Geometrie, Materialdefinitionen und Texturen werden nicht zusätzlich als FBX, OBJ, MTL oder GLB vorgehalten. Farbige Materialien ohne Texturen benötigen keine PNG. Direkt im Rust-Code erzeugte Stationsmodelle bleiben davon unabhängig.

Alle zu einem Modell referenzierten BIN- und PNG-Dateien müssen mit ausgeliefert werden. Blender-Quellen und Sicherungen gehören außerhalb von `asset/` nach `blender/` beziehungsweise `planning/`; der bearbeitbare Dummy liegt unter [blender/dummy_base.blend](blender/dummy_base.blend). Seine Pillenform ist noch nicht in den Runtime-Export `Dummy_Base.gltf` übernommen. Schriften und Lizenzen bleiben unverändert.

Die Blender-Skripte verwenden den gemeinsamen Exporter `blender/gltf_export.py` mit `GLTF_SEPARATE`. Texturquellen müssen als PNG vorliegen; andere exportierte Bildformate werden unmittelbar als Fehler gemeldet. Die Rust-Tests prüfen die glTF-Dokumente, externe Abhängigkeiten, aktuelle Ladepfade und die einmalige gemeinsame KayKit-Textur. Dateien mit eingebetteten Buffern oder Texturen sowie parallele Modellformate im Asset-Ordner lassen diese Prüfung scheitern.

### Module

`main.rs` wählt den Betriebsmodus und bindet die optionale Woodpecker-Integration ein. `app.rs` baut das Spiel auf und legt die Systemreihenfolge fest; `launch.rs` liest Startparameter und `commands.rs` koordiniert Befehle. `level/layout.rs` lädt die exportierten Raumdaten, `level/navigation.rs` verwaltet Kollision und Wege, `level/scenery.rs` den gemeinsamen Modellaufbau. `player.rs` enthält Bewegung und Sprünge, `player/camera.rs` die Spielkamera. `expedition.rs` verwaltet Aufgaben und Inventar, `expedition/props.rs` deren Darstellung und `expedition/stations/` die Stationsmodelle und ihre Definitionen. `items.rs` verbindet Inventar, Farben und Objektformen; Münzen liegen unter `items/coins.rs`. Zielübung, Oberfläche, Eingabe und Einstellungen haben eigene Module. Gerenderte Smoke-Tests liegen unter `dev/`, die modulübergreifenden Rust-Tests in `tests.rs`. `lib.rs` stellt die gemeinsamen Module für das Spiel und die separate App `test_zoo` bereit; beide verwenden denselben Asset-Pfad aus `ASSET_DIR`.

Die KayKit-Modelle stammen aus **KayKit: Prototype Bits** von Kay Lousberg, siehe [CC0-Lizenz](asset/License.txt). Blaster, Level und Stationsmodelle wurden für diese Demo erstellt. Die Schrift Barlow liegt mit ihrer [SIL Open Font License](asset/fonts/OFL.txt) unter `asset/fonts/`.
