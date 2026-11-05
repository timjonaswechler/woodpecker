# Echte Produkt-RGB-PNGs

A nach InitTick1, B nach dem zweiten expliziten Tick. Aufgenommen am2026-11-05
mit dem öffentlichen SDK und dem Produkt-Captureowner; keine Sollbilder oder
Desktopaufnahmen. Apple M1 Pro/Metal,65×49, Bevy0.20.0/wgpu30.0.1,
nicht-pipelined, MSAA aus. Pixel- und RGBformatprüfung stehen im ausführbaren
[Driver](../src/main.rs). Pause, Overwrite und negative Fälle werden ebenfalls
dort geprüft; die Bilder allein beweisen diese Abläufe nicht.

- [A](a.png): rotes Mesh links, vier Orientierungsmarker.
- [B](b.png): grünes Mesh rechts, exakt640 veränderte Pixel.

Ausführungsbinary-SHA256: `3194f343bc09941b289596b90dcc43821622299430a6c572e9c65e11ef29c668`.
Quellfingerprint des vorbereiteten Builds: `c7dd8cd64e6d876a980ac74218374cc5549ca5c952c15d3f2d1f7f439a541bf1`.
Die vier Produktfälle wurden in `target/headless-product-proof/runs/product-final-02/`
geprüft; nach Reviewkorrekturen wurden Pixel-/Pause- und Gerätverlustpfad erneut
unter `target/headless-integration-proof/{15-healthy,16-device-loss}/` ausgeführt.
Die hier enthaltenen Bilder stammen aus diesem letzten gesunden Lauf.
Vollständige Logs und Fallmanifeste bleiben lokal.
Dies ist ein aufgezeichneter enger Nachweis, keine allgemeine Plattformfreigabe.
