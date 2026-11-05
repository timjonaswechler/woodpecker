# Echte Basis-3D-Produktbilder

Aufgezeichnet am2026-11-05 über die öffentliche Session und den unveränderten
Produkt-Capturepfad. Keine Sollbilder oder Desktopaufnahmen. M1 Pro/Metal,
65×49, Perspective-Camera3d, StandardMaterial, ein DirectionalLight,
512px-Schattenmap/eine Kaskade. HDR-Zwischenziel, Tonemapping::None, feste Exposure;
MSAA/Dither/TAA aus, asynchrone Pipelinekompilierung, RGB8-PNG.

- [A nach InitTick1](a.png): roter Cuboid mit sichtbarer Flächenbeleuchtung und Schatten.
- [B nach Tick2](b.png): nach rechts bewegter grüner Cuboid; auch der Schatten bewegt sich.
- [Schatten aus](shadow-off.png): ansonsten dieselbe Tick1-Szene.
- [Licht aus](light-off.png): Ambient/Emission ebenfalls null; korrekt gerendertes Schwarz.

Licht-/Schattenorakel vergleichen tatsächliche Pixel statt nur Komponentenflags.
Im Kontrollvergleich sind27 graue Bodenpixel mit Schatten mindestens20 Stufen
dunkler, bei unveränderter sichtbarer Würfelgeometrie. Pause erhält alle Pixel
und Uhren; der DeviceLost-Fall endet typisiert ohne die fehlgeschlagene Zieldatei.

Finale Fallmanifeste/Logs: `target/headless-product-proof/runs/basic-3d-final/`.
Die gemeinsame Acht-Fälle-Serie mit den bestehenden2D-/Fehler-/Nativefällen liegt
unter `target/headless-product-proof/runs/three-d-and-regressions-01/`.
Ein vorheriger synchroner PBR-Timeout bleibt unter
`target/headless-3d-proof/01-healthy-3d/` erhalten. Kein Kaltcache-Performancebeweis;
keine Deadlineverlängerung, Cachelöschung oder zusätzlicher Simulationtick.

Binary-SHA256: `cbfae0f4087d8d716d8f08656c3fbd444b8d1916733c646c108af532c4f6bd5a`.
Vorbereiteter Quellfingerprint: `94385819249a3e4bfd0f22401035c24fc1af1a3d5c22c31934491d1abc375616`.
[Ausführbare Orakel](../../src/scene3d.rs).
