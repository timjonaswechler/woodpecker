# Signalstation

Eigenständige Bevy-Demo auf der Grundplatte aus dem [Levelplan](../planning/grundplatte_levelplan.pdf). Acht Räume, Galerien auf drei Metern Höhe, sechs Treppen und Brücken verbinden die Aufgaben. Unter den Galerien bleibt der Boden begehbar. Drei versteckte Bereiche bieten zusätzliche Verbindungen und Fundstücke. Der Sprungparcours in der zentralen Halle ist freiwillig.

## Spielen

```sh
cargo run --manifest-path test_app/Cargo.toml
```

Die Demo verwendet Bevy 0.20.0 und Rust 1.97.1 oder neuer. Sie gehört zum gemeinsamen Cargo-Workspace; Woodpecker wird nur mit dem Feature `automation` eingebunden. Der Asset-Pfad funktioniert unabhängig vom Arbeitsverzeichnis. Für einen optimierten Build `--release` ergänzen.

Optional setzt `--window-position 640,400` hinter dem Cargo-Trenner `--` die
anfängliche linke obere Fensterecke in physischen Desktop-Pixeln. Ohne das
Argument bleibt die Platzierung beim Fenstermanager. Größe, Skalierung, Fokus
und gespeicherte Spieleinstellungen werden dadurch nicht geändert. Die
Zwei-Session-Tests verwenden unterschiedliche Positionen; ihre gemeinsamen
Koordinaten stehen in `tests/_support.py` als `WINDOW_POSITIONS`.

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

## Bedienobjekte verwenden

Interaktionen funktionieren von allen zugänglichen Seiten des Objekts. In FPV muss es in Reichweite und im Fadenkreuz liegen. Der konkrete E-Hinweis erscheint direkt darunter, sobald die Aktion möglich ist. In der Übersicht erscheint die Aktion des nächsten erreichbaren Objekts unten in der Mitte. Wände, andere Stockwerke, freie Maus, Texteingabe und das Menü verhindern die Bedienung. Interaktionsflächen haben keine sichtbaren Bodenringe.

Jedes Bedienobjekt hat eine Funktion: Der Lagerhebel verschiebt das Regal, das Handrad öffnet ein Ventil, das Lesepult scannt eine Archivtafel und der Montageschalter baut eine Zelle. Die Abholung erfolgt am separaten Tisch. Bauteile und Zellen liegen auf der Tischplatte und verschwinden beim Einsammeln. Die Archivtafeln sind weiß und unterscheiden sich von den blauen Bauplänen fürs Inventar.

Benutzte Objekte zeigen dauerhaft einen grünen Status mit Häkchen und einer Beschriftung wie „Tafel ist gescannt“. Hebel und Handräder ändern ihre Stellung; eingelesene Tafeln verschwinden vom Lesepult. Die Belohnungstruhe öffnet sich nach der Zielübung, zeigt den Bauplan und bleibt nach dessen Entnahme leer und geöffnet. R stellt auch diese Zustände zurück.

## Zielübung, Eingabe und Einstellungen

Der Schießstand in G ist immer bereit. Mit V in FPV wechseln und mit Linksklick schießen. Jeder Treffer zählt erst ab sechs Metern Bodenabstand zur jeweiligen Scheibe. Blaue Linien markieren diesen Abstand vor den Zielen; die Anzeige nennt den Abstand zum nächsten stehenden Ziel. Zu nahe Treffer lassen die Scheibe stehen. Gültige Treffer zerbrechen sie dauerhaft. Die Mitte gibt 50 Punkte, die weiteren Ringe 25 oder 10. Sobald alle Scheiben gültig getroffen wurden, öffnet sich die Truhe hinten rechts. Mit E nimmst du den sichtbaren Bauplan heraus. Es gibt kein Zeitlimit und keine Startinteraktion. R setzt die Ziele zusammen mit dem Level zurück.

Brückenstützen und Wände blockieren Schüsse; mit WASD wechselst du zwischen den Schießbahnen. Die Übung ist für den Hauptauftrag nicht erforderlich. Menü, freie Maus, Texteingabe und ein inaktives Fenster sperren Schüsse.

## Ressourcen und Objektzeichen

Die Leiste am oberen Bildschirmrand zeigt den gesamten aktuellen Besitz, ohne Gesamtmengen oder Raumzähler. Gebaute Zellen verbrauchen Bauteile; eingesetzte Zellen verlassen das Inventar.

| Zeichen | Gegenstand und zugehörige Orte |
| --- | --- |
| Goldene Münze | Münzen zum Einsammeln |
| Oranges Zahnrad | Bauteile auf kleinen Ablagetischen; orangefarbener Lagerhebel und Montageschalter |
| Grüne Batterie | Energiezellen auf grünen Ausgabetischen und in den Sockeln des Generators |
| Blaues Blatt | Baupläne im Level und in der blauen Truhe der Zielübung |

Die Form bleibt auch ohne Farberkennung unterscheidbar. Bedienobjekte tragen Beschriftungen für Zweck, Voraussetzungen und Erledigt-Zustand. Die UI-Symbole lassen sich mit `python3 planning/create_resource_icons.py` und die Stationsschilder mit `python3 planning/create_station_labels.py` neu erzeugen. Die räumlichen Gegenstände entstehen in `src/level/objects/pickups/models.rs`.

Das Namensfeld unterstützt Unicode und die Systemzwischenablage. Während der Eingabe bleiben Spieltasten gesperrt. Enter oder Esc beendet die Eingabe. Das Einstellungsmenü bietet Maus-Empfindlichkeit, Sichtfeld, Lauftempo, Schatten, Routenanzeige und eine umkehrbare vertikale Mausachse. Name und Einstellungen werden in `test_app/settings.json` gespeichert.

## Level bearbeiten

### Modell-Zoo

Der [Modell-Zoo](../test_zoo/README.md) ist eine eigene App neben `test_app/`. Er verwendet die Objektmodelle und Assets der Spiel-Library, ohne das Spiel zu starten.

```sh
cargo run -p test_zoo
```

Steuerung, Zustandsvorschau und Bildexport sind in [test_zoo/README.md](../test_zoo/README.md) beschrieben.

### Blender und Level-Export

Die [Blender-Datei](../planning/groundplate/groundplate_concept.blend), der [PDF-Plan](../planning/grundplatte_levelplan.pdf) und das [Style-Handoff](../planning/STYLE_HANDOFF.md) beschreiben den Entwurf. Die Planungsdateien liegen im Repository-Root unter `planning/`. Das Spiel lädt `asset/meshes/environment/courtyard.gltf` samt `courtyard.bin`; Blender muss zum Spielen nicht laufen.

Im Blender-Outliner lassen sich die folgenden Collections über das Augensymbol unabhängig ausblenden. Das Kamerasymbol steuert die Sichtbarkeit im Renderbild.

| Collection | Inhalt |
| --- | --- |
| `01 Notwendig` | Grundplatte, Wände, Treppen, Brücken, Geländer, Parcours, Aufgabenstationen und Sammelobjekte |
| `02 Deko` | Kisten, Bänke, nicht bedienbare Maschinen, Bücherregale, Ordner, Schränke und Pflanzen samt Kübeln |
| `03 Planung` | Raumbeschriftungen, Größenvergleichsfigur und die alte Startkonsole des Schießstands |
| `04 Kamera und Licht` | Kamera und Renderbeleuchtung |

Für die reine Spielstruktur `02 Deko` und `03 Planung` ausblenden. Für die Dekoration allein `01 Notwendig` und `03 Planung` ausblenden. Funktionsobjekte bleiben samt Details zusammen, etwa der Generator mit seinen Zellensockeln. Das versteckende Regal an G2 gehört ebenfalls zur Spielstruktur. Dekorative Möbel behalten ihre Kollision im Spiel. Die Aufteilung in [scene_collections.py](../planning/groundplate/scene_collections.py) gilt auch beim Neuerstellen mit `create_model.py`.

```sh
blender --background planning/groundplate/groundplate_concept.blend \
  --python-exit-code 1 --python test_app/asset/blender/export_level.py
```

Unter macOS kann `blender` durch `/Applications/Blender.app/Contents/MacOS/Blender` ersetzt werden. Der Export verändert die gespeicherte Blend-Datei nicht. Er entfernt die statischen Platzhalter für Figur, Sammelobjekte und Zielscheiben, die das Spiel selbst erzeugt. Die Räume, Treppen, Galerien und Fundpositionen kommen aus `planning/groundplate/layout.py`; Hindernisse aus benannten Objekten der Blend-Datei. Geometrieänderungen an den Laufwegen müssen deshalb in beiden Quellen übereinstimmen. Der Exporter schreibt Räume, begehbare Flächen, Hindernisse und Fundpositionen in `courtyard.gltf` unter `extras.signalstation.layout`. `src/level/import.rs` prüft diese Metadaten beim Initialisieren aus der beim Kompilieren eingebundenen glTF-Datei. Danach verwendet das Level die unveränderliche `LevelDefinition` aus `src/level/definition.rs`. Dort liegen auch die spielseitigen Positionen der übrigen Bedienobjekte und Ziele. Der unten beschriebene Pilot importiert seine Positionen bereits aus demselben Export; Navigation, Objektverhalten und Aufgaben bleiben eigene Module.

Der Export enthält auch ausgeblendete Collections. Die Sichtbarkeit in Blender dient nur der Bearbeitung und den Planungsrenders; sie entfernt keine Objekte oder Kollisionen aus der Demo.

Die spielbaren Bedienobjekte aus `src/level/objects/` ersetzen die einfachen Konsolen des Blender-Entwurfs zur Laufzeit. Ihre Kollisionskörper und die unsichtbaren Interaktionsbereiche verwenden die Definitionen aus `src/level/objects/definitions.rs`. `src/level/interactions.rs` prüft Blickrichtung, Reichweite und freie Sicht. Die alten Konsolen und Bodenringe bleiben im Spiel ausgeblendet.

### Modularer Platzierungspilot in Werkhalle B

[kit.py](../planning/groundplate/kit.py) definiert die Treppe und die Arbeitsinsel.
Bau- und Aufgabenplatzierungen verwenden ein **1-m-Grundraster**; größere
Module wie die 3 m breite und 3 m hohe Treppe bauen darauf auf. Das Raster ist
für Räume, Wände, Treppen, Übergänge und aufgabenrelevante Objekte vorgesehen.
Deko wird anschließend ohne Rasterzwang gesetzt; Wandstärken, Stufen und andere
Details innerhalb eines Modells bleiben ebenfalls frei. Die Treppe lässt sich
in 90°-Schritten drehen. Freie Bewegung und das 0,5-m-Navigationsraster bleiben
unabhängig davon.

Im Piloten sind die Treppenanschlüsse sowie die Arbeitsinsel und ihre
Anlaufpunkte auf ganze Meter ausgerichtet. Die übrigen bestehenden Räume und
Aufgabenobjekte sind noch nicht auf dieses Bauraster migriert.

Die Treppe ist ein vollständiges Blender-Modell mit integrierten Geländern:
`asset/meshes/environment/workshop_stair.gltf` plus BIN. Ihr lokaler Einfügepunkt
liegt am unteren Antritt `(0, 0, 0)`, der obere Anschluss bei `(0, 3, -8)` in
Spielkoordinaten. Sie ist 3 m breit und hat zwölf Stufen. Modell, Laufebenen und
Geländer-Kollisionskörper entstehen aus derselben Definition; der Level-Export
enthält nur ihre Instanzplatzierung, nicht noch eine zweite Kopie der Geometrie.

Bauteilablage, Montagepresse und Zellenausgabe bilden eine gemeinsame Reihe.
Ihre Größen, Positionen und freien Anlaufpunkte werden als `layout.workshop`
exportiert. Spielmodelle, Kollisionskörper, Fundpositionen und Auftragsführung
verwenden diese Daten. Die übrigen fünf Treppen und Räume bleiben unverändert.

Nach Änderungen an `kit.py` zuerst das Blender-Modell neu erzeugen, dann wie
oben exportieren. `--no-render` spart nur die zwei Planungsbilder:

```sh
blender --background --python-exit-code 1 \
  --python planning/groundplate/create_model.py -- --no-render
blender --background planning/groundplate/groundplate_concept.blend \
  --python-exit-code 1 --python test_app/asset/blender/export_level.py
```

Der bearbeitete Blaster liegt in [asset/blender/training_blaster.blend](asset/blender/training_blaster.blend). Nach Änderungen separat exportieren:

```sh
blender --background test_app/asset/blender/training_blaster.blend \
  --python-exit-code 1 --python test_app/asset/blender/export_blaster.py
```

## Prüfen

```sh
cargo check --manifest-path test_app/Cargo.toml --all-targets --all-features
python3 -m unittest discover -s planning/groundplate -p 'test_*.py'
cargo test --locked -p test_app --all-features --lib
```

Die fensterlosen Pilottests prüfen Platzierung, Export, Treppenanschlüsse,
Körperbewegung, freie Anlaufpunkte und die realen Interaktionsregeln der
Arbeitsinsel. Sie ersetzen keine grafische Spielabnahme und keine
Performance-Messung.

Die [Abnahmesuite](../tests/README.md) prüft Woodpeckers Verträge gegen diese
Referenz-App: Verwaltung, Startup/Ticks, Inspect, virtuelle Eingaben, Screenshots,
Recording/Replay, Reports und Shutdown. Der vollständige lokale Lauf und der
separat freigegebene echte GitHub-Nachweis sind bestanden; geprüfter Quellstand
und Belege stehen ausschließlich im [Arbeitsplan](../docs/testing-plan.md).
Die früheren Smoke-Test-Startoptionen gehören nicht mehr zur aktuellen Suite.
In kontrollierten Sessions macht `src/woodpecker.rs` die Last-Schedule-Durchläufe
als reflektierte Resource `SimulationTicks` lesbar. Der Zähler führt keine Ticks
aus und bleibt ohne kontrollierte Session deaktiviert. Ein erfolgreicher
Build-Check belegt keine korrekte Spiel- oder Woodpecker-Funktion.

### Opt-in-Diagnose für Abnahmetests

Nur mit dem Feature `automation`, einer kontrollierten Session und dem
App-Startargument `--woodpecker-diagnostics` werden die Diagnoseaktionen
installiert. **F11 erzeugt absichtlich eine Panic und kann den App-Prozess
beenden.** Ohne diese Voraussetzungen bleibt die normale Bedienung unverändert.

| Virtuelle Eingabe und nächster expliziter Tick | Wirkung |
| --- | --- |
| F12 | Ein Error-Level-tracing-Event mit Target `signalstation::diagnostics` und Meldung `Signalstation diagnostic tracing error` pro Press-Flanke |
| F11 | Panic mit Meldung `Signalstation diagnostic panic` |

Die Diagnoseaktionen führen selbst keine Ticks aus und sind keine neuen
Woodpecker-Commands. Für einen tracing-Report muss zusätzlich
`report.tracing_errors = true` gelten und der Logfilter Error-Events zulassen.
Die Pilotfälle 3.16/3.17 wählen ausschließlich den lokalen Provider. Modul und Build sind
geprüft; F12/tracing-Report und F11/Panic mit Prozessende sind durch die
freigegebenen Tests 3.16/3.17 ebenfalls live belegt.

### Report-Proben 4.47–4.50 und Event-Überlauf 4.68

**Nur mit den obigen Diagnose-Voraussetzungen.** Zusätzliches
`--woodpecker-report-action <mode>` registriert F9 und die reflektierte Resource
`test_app::diagnostics::reports::ReportProbe`. Keine Probe führt selbst Ticks aus.
**`event-overflow` erzeugt absichtlich einen Fehler-Burst: Ohne Eventabholung
beendet Woodpecker die eigene Prozessgruppe wegen Pufferüberlauf.** Nur im
entsprechend freigegebenen Test verwenden, nicht im normalen Spiel.

| Mode, ausgelöst durch F9 und einen expliziten Tick | Wirkung |
| --- | --- |
| `caught-system` | Panic im System abgefangen; Hook bleibt beobachtbar. |
| `caught-task` | Panic innerhalb eines AsyncCompute-Tasks abgefangen. |
| `caught-worker` | Worker-Thread panikt, der Aufrufer verarbeitet das Join-Err. |
| `caught-any` | Abgefangenes `panic_any(47_u32)`, ohne erfundene Stringmeldung. |
| `handled-error` | Behandeltes Err ohne Panic oder tracing-Event. |
| `exit-0`, `exit-17` | **Absichtlicher unerwarteter Prozess-Exit** mit Status 0/17. |
| `event-overflow` | Begrenzt 300 Error-tracing-Events mit Target `signalstation::overflow`, Meldung `Signalstation overflow event 0` bis `299`; kein zusätzlicher Tick oder Thread. |

ReportProbe zählt Aktionen, abgefangene Panics und behandelte Fehler.
`--woodpecker-no-tracing-layer` lässt gezielt die sonst normale
Layer-Registrierung weg; mit aktivierter tracing-Beobachtung muss Start scheitern.
Auch dieses Argument wirkt nur mit kontrollierter Automation und Diagnose-Opt-in.

4.49 baut separat mit `CARGO_PROFILE_DEV_PANIC=abort` in ein eigenes
`CARGO_TARGET_DIR`, ohne das normale Buildprofil zu ändern. F11 beendet dann
absichtlich die App per Abort. Die Fälle 4.47–4.50 einschließlich echtem
Abort-Build sind im vollständigen lokalen Abnahmelauf bestanden.
4.53 verwendet für den Github-Provider nur ein testeigenes externes gh-Werkzeug,
keine echte Veröffentlichung.

4.68 verwendet einen direkten Rust-Handle ohne Server, aktiviert nur den
passenden Error-Target und liest vor dem absichtlichen Pufferüberlauf keine
Events. Auch dieser echte App-Lauf ist im lokalen Gesamtnachweis bestanden.

### Prozess-/Fortschrittsprobe für Shutdown-/Rust-Handle-Tests

Mit `--woodpecker-diagnostics --woodpecker-process-probe`, ausschließlich in
kontrollierter Automation, schreibt die App in ihr eigenes Session-Artefaktverzeichnis:

- `process-probe.json`: PID der wirklichen App; Tests prüfen vor Signalen
  zusätzlich die lebende Abstammung vom eigenen Server beziehungsweise direkten
  Rust-Testprozess.
- `simulation-ticks.json`: atomar ersetzter Spiegel der bereits vorhandenen
  reflektierten Resource `SimulationTicks`, nach deren Zählung im selben Last-Lauf.

Die Probe sendet keine Signale, keine Inputs und fordert keine Ticks an. Sie
ermöglicht während Replay eine unabhängige App-Fortschrittsbeobachtung ohne
zusätzlichen Spiel-Command; interne Replay-Pendings werden nicht als öffentliche
Server-Activity vorausgesetzt. Direkte Rust-Tests beobachten damit auch den
Fortschritt nach Pending-Drop und eigenes Prozess-Cleanup ohne Eventabholung. SIGSTOP/SIGCONT bzw. erzwungenes Produktende stehen
ausdrücklich im jeweiligen Test. Normale App-/Automation-Starts ohne dieses
zusätzliche Opt-in bleiben unverändert. Die zugehörigen Shutdown-/Rust-Fälle
sind Bestandteil des bestandenen lokalen Gesamtnachweises.

### Vorbereitung weiterer Abnahmefälle (4.1)

Die folgenden Startargumente wirken **zusätzlich** zu `--woodpecker-diagnostics`,
nur mit `automation` und kontrollierter Session. Ohne das jeweilige Argument
bleiben die bestehende Startup-Registrierung, Zeitpolitik und World unverändert.
F11/F12 allein installieren keine Beispieldaten. Es gibt keine zusätzlichen
Woodpecker-Commands, versteckten Ticks oder eine Headless-Variante.

| Startargument | Vorbereitete Situation |
| --- | --- |
| `--woodpecker-startup pending` | Die Startup-Asset-Liste bleibt unbekannt; der Test wählt Tick-/Zeitbudget in seiner Launchkonfiguration. |
| `--woodpecker-startup missing-scope` | Der vorhandene Startup-Asset-Adapter wird entfernt. |
| `--woodpecker-startup missing-asset` | Der Adapter verfolgt die bewusst nicht vorhandene Datei `__woodpecker_missing_startup__.png` unter dem Asset-Root. |
| `--woodpecker-startup no-camera` | Alle App-Kameras werden in PostStartup deaktiviert; Renderer und echtes Fenster bleiben vorhanden. |
| `--woodpecker-clock steady` | App-eigene Zeitstrategie mit 50 ms pro Tick und virtueller Geschwindigkeit 1. |
| `--woodpecker-clock double` | Dieselbe Bevy-Real-Clock, virtuelle Geschwindigkeit 2. |
| `--woodpecker-clock paused` | Dieselbe Bevy-Real-Clock, virtuelle Zeit pausiert. |
| `--woodpecker-reflection` | Zusätzliche Resources, eine benannte Entity und eine per F10 ersetzbare Hierarchie für Inspect-Sonderfälle. |

Die 50 ms beschreiben Bevys Zeitschritt, nicht die Wartezeit oder Warp-Pace.
Die Clock-Vorgabe initialisiert nur den Real-Zeitursprung mit Dauer null; sie
führt keinen Tick aus. Für den Erhaltungstest 4.4 muss im `[tick.clock]`-Block
`kind = "application"` stehen; die Fixed-Option `seconds_per_tick` entfällt dort.
Eine SDK-Fixed-Clock darf die App-Zeitstrategie weiterhin überschreiben;
virtuelle Pause und Geschwindigkeit bleiben App-Zustand.

Die Reflection-Typen stehen unter `test_app::diagnostics::reflection::`:
- `Values`: String-/Integer-Key-Maps, Set, drei Enum-Varianten und Image-Handles
  mit Pfad, zwei verschiedenen flüchtigen IDs (je typed/untyped) und UUID. Das
  Pfad-Handle verwendet die vorhandene `textures/prototypebits_texture.png`;
  die flüchtigen Handles werden nur reserviert. Vor Ready schreibt die Fixture
  ihre nativen Indexbits unabhängig vom Inspect-Serializer nach
  `reflection-handle-bits.json` im Session-Artefaktverzeichnis. Keine festen
  Allocator-IDs über Sessions hinweg.
  Die Fixture-UUID ist ausdrücklich `12345678-1234-5678-90ab-cdef01234567`, nicht
  ein aus Bevys Default-Handle übernommener Sollwert.
- `NonFinite`, `UnsupportedMapKey`, `OpaqueValue`: NaN, ein nicht als JSON-Key
  darstellbarer Vektor und ein opaker Typ ohne Serializer.
- `OpaqueNull`: opaker Unit-Typ mit Serde-Unterstützung für einen lesbaren Nullwert.
- `NotReflectableResource`: vorhandener, registrierter Typ ohne ReflectResource.
  `AbsentResource` und `AbsentComponent` sind registriert, aber nicht eingesetzt.
- Die Entity heißt `Woodpecker reflection fixture`; sie besitzt den reflektierten
  `Marker`, einen bewusst unregistrierten `Unregistered`-Component und
  `NotReflectableComponent` ohne ReflectComponent.
- Vier weitere `HierarchyNode`-Entities: `Inspect root` hat in Children-Reihenfolge
  zuerst `Inspect child Z`, dann `Inspect child A`; Z hat ein unbenanntes Blatt.
  Die Kinderreihenfolge ist absichtlich gegen die Handle-Indexsortierung gewählt.
  **Nur mit Reflection-Opt-in** entfernt F10 beim nächsten ausdrücklich angeforderten
  Tick den ganzen Baum und erstellt einen neuen mit denselben Namen/Beziehungen.
  Kein Renderobjekt und kein automatischer Tick; normale Eingaben bleiben unverändert.

Der unregistrierte Component wird über entdeckte Components abgefragt; sein
Type Path ist kein gültiger Query-Selektor. Die Fixture-Typen sind von der globalen
Auto-Registrierung ausgenommen; erst das Reflection-Opt-in registriert die
oben dafür vorgesehenen Typen. Die Daten erzeugen keine Renderobjekte oder
Spielaktionen. Erwartete Inspect-Ausgaben werden in 4.16–4.19 geprüft, nicht durch
diese Vorbereitung als bestanden behauptet.

Die vorhandenen Buildvarianten ohne Automation und mit `automation` genügen;
keine neuen Dependencies oder Cargo-Profile sind nötig. **Build-/Formatprüfung
bestanden; alle vier Startup- und drei Clock-Modi sind durch 4.2/4.4 live belegt.**
Auch die Reflection-Funktionsnachweise 4.16–4.19 sind live belegt, einschließlich
der F10-Hierarchieersetzung mit genau zwei ausdrücklich angeforderten Ticks,
frischen Handles und Ablehnung aller alten Handles.
Belege stehen im
[Arbeitsplan](../docs/testing-plan.md). Die Report-, Prozess- und Abort-Proben
sind oben mit ihren jeweiligen Opt-ins und Risiken beschrieben.

## Code und Assets

### Asset-Formate

Alle Modelldateien liegen unter `asset/meshes/`, geordnet nach ihrer Verwendung im Spiel:

- `characters/`: Spielfiguren.
- `weapons/`: Waffen wie der Trainingsblaster.
- `environment/`: Level, Wände, Böden, Treppen, Brücken, Tore und Gelände.
- `props/`: Einrichtung, Behälter, Paletten, Tische und Konsolen.
- `collectibles/`: Sammelobjekte wie Münzen und Energiezellen.
- `targets/`: Zielscheiben, Ständer und Bruchstücke für den Schießstand.

Die Herkunft eines Modells bestimmt keinen eigenen Ordner. glTF und zugehörige BIN-Datei liegen jeweils zusammen; gemeinsam verwendete Texturen liegen unter `asset/textures/`.

Für Modell-Dateien beider Apps gilt ein gemeinsamer Standard: `.gltf` für Szene und Materialien, externe `.bin`-Dateien für Geometrie und externe `.png`-Dateien für Texturen. Die 72 KayKit-Modelle teilen genau eine `asset/textures/prototypebits_texture.png`. Geometrie, Materialdefinitionen und Texturen werden nicht zusätzlich als FBX, OBJ, MTL oder GLB vorgehalten. Farbige Materialien ohne Texturen benötigen keine PNG. Direkt im Rust-Code erzeugte Objektmodelle bleiben davon unabhängig.

Alle zu einem Modell referenzierten BIN- und PNG-Dateien müssen mit ausgeliefert werden. Blender-Quellen, Sicherungen und Exportskripte liegen unter `asset/blender/`, Planungsdateien unter [`planning/`](../planning/) im Repository-Root. Der Bearbeitungsordner `asset/blender/` enthält Quellen, keine Laufzeitmodelle. Der bearbeitbare Dummy liegt unter [asset/blender/dummy_base.blend](asset/blender/dummy_base.blend). Seine Pillenform ist noch nicht in den Runtime-Export `Dummy_Base.gltf` übernommen. Schriften und Lizenzen bleiben unverändert.

Die Blender-Skripte verwenden den gemeinsamen Exporter `asset/blender/gltf_export.py` mit `GLTF_SEPARATE`. Texturquellen müssen als PNG vorliegen; andere exportierte Bildformate werden unmittelbar als Fehler gemeldet. Die bisherigen automatisierten Asset-Prüfungen sind entfernt; die beschriebenen Formate bleiben die Vorgabe für Laufzeitmodelle.

Die Exporter schreiben Modelle direkt in ihre jeweilige Kategorie. `asset/blender/create_station.py` legt die erzeugte Maßübersicht unter `artifacts/station-dimensions.json` ab.

### Module

`main.rs` wählt den Betriebsmodus und bindet die optionale Woodpecker-Integration ein. `lib.rs` stellt Spiel und Modell-Zoo dieselben Objektmodelle und denselben Asset-Pfad aus `ASSET_DIR` bereit.

| Modul | Zuständigkeit und eigener Zustand |
| --- | --- |
| `app/` | App-Aufbau, Startparameter, Menü und Neustart; benannte Systemphasen in `schedule.rs` |
| `app/settings/` | Gültige Einstellungen, Name und Notizen; Laden und Speichern |
| `input/` | Native und kontrollierte Eingaben, Mausbindung, gemeinsame Eingaberegeln und Spielabsichten pro Frame |
| `player/` | Position, Bewegung, Körpermodell, Kameramodus und gemeinsame Augenpose; Blaster mit Rückstoß und Schussabstand |
| `level/` | Geprüfte Leveldefinition, Import, Umgebung, Navigation und räumliche Interaktionen |
| `level/objects/` | Konkrete Bedienobjekte wie Werkbank, Regal, Ventil und Scanner, ihre Modelle und abgeleitete Darstellung |
| `level/objects/shooting_range/` | Zielscheiben, Treffer und Punkte; meldet den Abschluss an den Auftrag |
| `level/mission/` | Aufgabenfortschritt, auftragsgebundene Objektzustände, Inventar und atomare Aktionen |
| `ui/` | Oberfläche, Eingabefelder und Texte; zeigt Zustand an und reicht Änderungswünsche weiter |

Dateidaten werden beim Eintritt geprüft: Der Levelimport liefert gültige Flächen, Hindernisse, Referenzen und Fundpositionen. `Settings` normalisiert geladene Werte und spätere Änderungen mit denselben Regeln. Private Felder und geprüfte IDs verhindern ungültige direkte Änderungen. Veränderliche Voraussetzungen wie Besitz, Reichweite oder ein bereits bedientes Objekt werden beim Ausführen der Aktion geprüft. Ein angezeigter Bedienhinweis garantiert keinen späteren Erfolg.

Der Auftrag besitzt den Fortschritt und das Inventar. Objektmodelle leiten daraus ihren sichtbaren Zustand ab. Der Schießstand besitzt seine Treffer und Punkte und liefert beim Abschluss einen Nachweis zum Freigeben der Truhe. Der Modell-Zoo verwendet `ModelKind`, `VisualState` und `spawn_model`, ohne einen Auftrag anzulegen.

Jedes Modul registriert seine Systeme selbst. Die Reihenfolge steht in `app/schedule.rs`: Eingaben und Befehle, Weltvorbereitung, Bewegung, aktuelle Körper- und Kamerapose, Interaktionen, Schüsse und Treffer, Fortschritt und Darstellung. Ein verschobenes Regal aktualisiert Kollision und Wegsuche zusammen; Schüsse und Interaktionen verwenden die aktuelle Augenpose.

Die KayKit-Modelle stammen aus **KayKit: Prototype Bits** von Kay Lousberg, siehe [CC0-Lizenz](asset/License.txt). Blaster, Level und Objektmodelle wurden für diese Demo erstellt. Die Schrift Barlow liegt mit ihrer [SIL Open Font License](asset/fonts/OFL.txt) unter `asset/fonts/`.
