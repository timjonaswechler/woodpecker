# Rendererfreien Host aus derselben Vertragsimplementierung bauen

CLI und Server brauchen keinen Renderer. Das private Paket `woodpecker_host`
kompiliert deshalb die engineunabhängigen Produktionsmodule aus `src/`, während
`woodpecker` dieselben Quellen zusammen mit dem Bevy-Adapter als unverändertes
öffentliches SDK anbietet. `woodpecker_cli` verpackt das bisherige Binary. Diese
Quellteilung vermeidet kopierte Wire-Typen und einen gleichzeitig nötigen Umbau
sämtlicher öffentlicher Rust-Typidentitäten; zwischen den beiden Varianten wird
nur das vorhandene Prozessprotokoll übertragen. Die minimale ECS-Abhängigkeit
des Hosts unterstützt den vorhandenen Bevy-kompatiblen Koordinator, nicht Rendern
oder native Eingaben.

Die zehn öffentlichen Rust-Verträge teilen ein ausführbares `contracts`-Target,
aber keinen Prozesszustand: Auch beim vollständigen Cargo-Testlauf startet jeder
Fall seriell einen eigenen exakten Unterprozess. Die Abort-Fixture ist dagegen
ein kleines enginefreies Testprogramm mit dem Produktionsprotokoll und echten
Panic-Hooks. Sie prüft `panic=abort`, nicht den Grafikstart des Spiels.

Vorbereitete Programme bleiben unabhängig vom Compilercache ausführbar. Dessen
optionaler verlustfreier Ruhe-Checkpoint erhält Bytes, Modi und Datei-Zeitstempel;
vor einem Build muss er wiederhergestellt werden. Ein Speicherbudget nach der
Vorbereitung ist deshalb ausdrücklich kein Spitzenbudget während des Kompilierens.
Compilerprofile, Assertions und vollständige Testverträge werden dafür nicht geändert.
