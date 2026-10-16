# Stationsmodelle

Für diese Demo in Blender erstellte Modelle. Jede `.gltf`-Datei enthält eine Szene
mit Index 0, ein Mesh mit Materialgruppen und Materialdefinitionen. Die zugehörige
`.bin`-Datei enthält Geometrie und Vertexnormalen und muss mit ausgeliefert werden.
Die Modelle benötigen keine Texturen, Kameras oder Lichter.

Die Koordinaten und Maße unten gelten nach dem glTF-Export in Metern: X quer,
Y nach oben, Z längs. `dimensions.json` enthält die aus den exportierten
Vertexdaten berechneten AABBs und Dreieckszahlen. `bytes` ist die gemeinsame
Dateigröße von `.gltf` und `.bin`.

| Datei | Minimum X/Y/Z | Maximum X/Y/Z | Verwendung |
| --- | --- | --- | --- |
| `energy_cell.gltf` | −0,175 / 0 / −0,175 | 0,175 / 0,65 / 0,175 | Energiezelle, Origin in der Mitte der Standfläche |
| `relay_console.gltf` | −0,55 / 0 / −0,35 | 0,55 / 1,2 / 0,3755 | Terminal, Bildschirm und Steckplatz zeigen nach **+Z** |
| `signal_tower.gltf` | −0,75 / 0 / −0,75 | 0,75 / 5,8 / 0,75 | Antennenturm, Origin in der Mitte der Standfläche |
| `bridge_module.gltf` | −1,3 / −0,39 / −2 | 1,3 / 0,92 / 2 | Brücke entlang Z, Oberkante des Laufdecks auf **Y=0**, offene Enden |
| `energy_gate.gltf` | −1,7 / 0 / −0,2 | 1,7 / 3 / 0,215 | Offener Torrahmen quer zu X, Anzeige nach +Z |
| `cliff_module.gltf` | −2 / −3,5 / −2 | 2 / 0 / 2 | Felsmodul, exakt rechteckige 4×4m Oberseite auf **Y=0** |

Die Brücke hat zwischen den Geländern mindestens 2,2m Platz. Ihr Unterbau liegt
unterhalb der Laufhöhe; Geländer und kleine seitliche Leuchten ragen darüber.
Der Torrahmen lässt mittig mindestens 2,6m Breite und 2,65m Höhe frei. Er enthält
keine Türfläche. Eine schaltbare Sperre und alle Kollisionen legt die Spiellogik an.
Beim Felsmodul liegen sämtliche Vertices auf oder unter Y=0; der Sockel verjüngt
sich nach unten. Nachbarmodule können bündig aneinander gesetzt werden.

Die Palette verbindet warmes Offwhite, dunkles Petrol, korallorange Warnmarkierungen
und türkise Energieanzeigen. Das Felsmodul verwendet drei Sand- und Graubrauntöne.

## Reproduzieren und bearbeiten

Vom Repository-Verzeichnis aus:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --python-exit-code 1 --python test_app/blender/create_station.py
```

Das Skript ersetzt ausschließlich diese sechs glTF-/BIN-Paare, `dimensions.json`,
`test_app/blender/station_kit.blend` und `test_app/artifacts/station-assets.png`.
Es öffnet oder verändert den Blaster nicht.

Die Blender-Datei enthält die Szene `Station source models` mit sechs benannten
Collections an ihren Export-Origins. `Station kit overview` zeigt Instanzen für
die Übersicht und enthält die Studiobeleuchtung. Die Energiezelle ist nur in
dieser Übersicht um Faktor 2,6 vergrößert. Das exportierte Modell bleibt in Originalgröße.

Manuelle Änderungen an `station_kit.blend` werden beim erneuten Generieren
überschrieben. Dauerhafte Varianten sollten im Generator oder in einer
separaten Blender-Datei entstehen.
