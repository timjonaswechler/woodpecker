# Stil von Levelrender und PDF

Der aktuelle Entwurf zeigt acht Räume als übersichtliches Architekturmodell. Geometrie, Farben und Beschriftungen erklären die Wege und ihre Funktion. Die PDF ergänzt Renderbilder durch Draufsichten und kurze Erläuterungen.

| Bereich | Vorgabe |
|---|---|
| 3D-Stil | Stilisierter Low-Poly-Blockout. Einfache Körper, leicht abgeschrägte Kanten und matte Farbflächen. |
| Darstellung | Orthografische Vogelperspektive. Dächer ausgeblendet, vordere Außenwände gekürzt. Treppen, Brücken und Wege darunter bleiben erkennbar. |
| Materialien | Rauheit `0.85`, überwiegend Kantenfase `0.04 m`. Pastellfarbene Raumböden, helle Wände und dunkle Geländer. |
| Farben im Modell | Türkis `#277F78` für obere Hauptwege, Gold `#DC9B39` für obere Abkürzungen und Münzen, Violett `#8864AB` für versteckte Bodenwege, Blau `#428FC1` für Baupläne, Orange `#D77438` für Bedienelemente. |
| Beleuchtung | Zwei große Flächenlichter, weiche Schatten, neutraler grauer Hintergrund. |
| Rendering | Blender Cycles, 40 Samples, Denoising, AgX. Übersicht `2000 × 1350 px`, Detail `1600 × 1100 px`, PNG. |
| PDF-Layout | Querformat `16:9`, `960 × 540 pt`. Große Abbildungen links, kurze Erläuterungen rechts. Weiße Karten mit abgerundeten Ecken und großzügigen Abständen. |
| PDF-Typografie | Barlow Regular. Titel `31 pt`, Fließtext etwa `15.5–16 pt`. Hintergrund `#F5F4ED`, Text `#173438`, Akzent `#237F78`. |

Alle folgenden Links beginnen am Dateisystem-Root dieses Arbeitsplatzes.

- [Blender-Modell](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/planning/groundplate/groundplate_concept.blend)
- [PDF mit sechs Folien](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/planning/grundplatte_levelplan.pdf)
- [Übersichtsrender](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/artifacts/groundplate-plan/raeumlicher_entwurf.png)
- [Detailrender der zentralen Halle](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/artifacts/groundplate-plan/zentrale_halle.png)
- [Modell- und Rendergenerator mit Kamera-, Material- und Lichtwerten](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/planning/groundplate/create_model.py)
- [PDF-Generator mit Layout und Typografie](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/planning/groundplate/create_slides.py)
- [Gemeinsame Geometrie- und Positionsdaten](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/planning/groundplate/layout.py)
- [Schriftdatei Barlow Regular](/Users/tim-jonaswechler/GitHub-Projekte/bevy/woodpecker/test_app/asset/fonts/Barlow-Regular.ttf)

Zum Neuerstellen zuerst `create_model.py` mit Blender im Hintergrund ausführen, danach `create_slides.py` mit Python und ReportLab. Die PNGs liegen im von Git ignorierten Ordner `test_app/artifacts/groundplate-plan/` und entstehen beim Rendern erneut. Das Blender-Modell und die PDF sind versioniert.
