# Lokaler Headless-Renderprüfstand

**Ergebnis: Der enge 2D/Metal-Pfad ist technisch umsetzbar und ausreichend belegt,
um eine kleine Produktintegration anzugehen.** Dieser historische Prüfstand ist
nicht die Integration. Sein dokumentierter Ausführungsstand basiert auf Produkt-
Commit `100cc683`; die inzwischen ergänzte [Produktintegration](../../../docs/headless-plan/SLICE.md)
hat eine eigene SDK-Fixture. Die historische Probe/Lockfiles bleiben erhalten;
sie sind keine Behauptung, dass der inzwischen geänderte Produkt-HEAD dieselbe
Baseline ist. Native Defaults und die CI-Pause bleiben bestehen.

## Was belegt ist

Lokale Läufe mit Bevy0.20.0, wgpu/core/hal/Naga30.0.1 auf Apple M1 Pro/Metal,
macOS27.2, Rust1.99.1: echte Camera2d, fünf Meshes/ColorMaterial, 65×49-sRGB-Image,
MSAA aus, Copy/Map und eigene PNGs. Kein natives Fenster, Surface, Displayhandle,
Winit-/Clipboardpfad oder Softwarefallback.

- Ein ausdrücklich angeforderter **InitTick1** öffnet den Renderpfad. Danach laufen
  Rendering, Readback und normale Fehlerbeobachtung ohne weiteren Simulationtick.
- Der zusätzliche öffentliche Fatal-Shortcircuit und die originale Extraction
  funktionieren auch gesund zusammen: RenderStartup genau einmal; keine doppelte
  gemessene Entityübertragung oder Image-Readerverarbeitung.
- Reale Pipeline-/Draw-/Copyvalidation und ein tatsächlicher Mapfehler werden
  erkannt. Der Publisher verweigert Cancel, abgelaufene Frist, alte Request-/
  Generation-/Frameidentität, beobachteten Fatal sowie Schreib-/Renamefehler.
- Eine kleine eigene Publication-Implementation veröffentlicht ein geprüftes
  PNG-/Receipt-Bundle erst nach dem finalen Ownercheck und Directory-Rename.
  Sie ist **nicht** die Produkt-Capture-/Serverimplementation.

Die jüngste feste Serie umfasst15 Prozesse, alle mit erfüllten Positiv-/
Negativorakeln, ohne Retry.52 vollständige PNGs wurden unabhängig dekodiert;
ein absichtlich partieller8-B-Prefix blieb ausschließlich im Staging.
Alle Fälle verwenden das originale öffentliche `woodpecker::session::Plugin`
und genau eine originale Wireantwort mit requested/executed_ticks=1.

## Betriebsvertrag und Grenzen

Erfolg bestätigt **diesen abgeschlossenen Frame**, seine Identität und keinen am
benannten Ownercheck **beobachteten** Fatal. Er garantiert nicht zukünftige
Gerätegesundheit oder die atomare Erfassung sämtlicher noch unbeobachteter
Backendereignisse bis zur Dateisichtbarkeit. Ein späterer Geräteverlust macht
bereits korrekte CPUbytes nicht rückwirkend falsch. Queuecompletion allein ist
kein GPU-Erfolgsnachweis.

Keine Qualifikation für andere Größen/Formate, PBR/3D, UI/Text, Linux,
Softwareadapter, Vollspiel, pipelined Rendering, Mehrsession oder echten
Hardware-/Treiberverlust. Device::destroy betrifft nur das eigene logische Gerät.
Die Shader-Assetbridge und kontrollierten Image-Nachrichten sind Probenhilfen,
kein vollständiger Asset-/Reader-/Trackerbeweis für beliebige Anwendungen.
Scopes sind threadlokal; die direkten negativen APItests sichern nicht sämtliche
Renderworker ab. Produkt-Sandbox, RGB-PNG-Vertrag und Shutdowntransport wurden
nicht ersetzt oder abgenommen. Das reale v3-Ready meldet `screenshot:false`.

## Belege ohne GPU prüfen

```sh
python3 -B tests/prototypes/headless-render-lab/run.py plan
python3 -B tests/prototypes/headless-render-lab/audit.py
python3 -B -m unittest discover -s tests/prototypes/headless-render-lab -p 'test_*.py'
```

Diese Befehle bauen/starten keine Anwendung und verwenden keine GPU. Sie prüfen
bereits aufgezeichnete Daten, Probenquell-/Dateihashes, PNG-/Receipt-Konsistenz,
Fehlerorakel und den expliziten Ausführungsschutz. Ein CPUaudit ist **kein neuer
GPU-Nachweis**. [Aufzeichnung und Bilder](evidence/README.md) enthalten auch die
historischen Fehlschläge und die korrigierte öffentliche Hook-Erkenntnis.

Cargo.toml/Lock und die Rustquellen entsprechen der tatsächlich verwendeten
jüngsten Probe; lediglich eine zusätzliche abschließende Leerzeile in
`study_base.rs` wurde entfernt. Beide Hashstände sind dokumentiert, die originalen
Bytes daraus rekonstruierbar. Der portable Runner und die CPU-Prüf-/
Publikationsverpackung entstanden danach; kein neuer GPUlauf wird behauptet.

## Später ausdrücklich genehmigte lokale Wiederholung

**Nur mit neuer ausdrücklicher lokaler GPUfreigabe.** Historische Wiederholung
setzt Export-Commit `3c431d9` mit unveränderter Produktbaseline `100cc683` voraus;
der Runner wechselt den Checkout nicht selbst. Für den aktuellen Produkt-HEAD
stattdessen die [SDK-Fixture](../../fixtures/offscreen/README.md) verwenden.
Der Schalter erteilt selbst keine Rechte. Keine CI-/Workspace-Autoruns oder
stillen Builds/Fallbacks.

```sh
python3 -B tests/prototypes/headless-render-lab/run.py build
python3 -B tests/prototypes/headless-render-lab/run.py run --allow-gpu --name local-proof
python3 -B tests/prototypes/headless-render-lab/audit.py \
  --run target/headless-render-lab/runs/local-proof
```

- Build standardmäßig offline, maximal900s; fehlende Dependencies führen zum
  sichtbaren Fehler. `build --online` erlaubt bewusst normale Cargo-Downloads,
  installiert aber keine Toolchain oder Systempakete.
- Genau die15 aufgelisteten Fälle seriell, je120s äußerer Prozesswatchdog,
  höchstens64 Renderpump-Schritte. Gesamtlauf maximal zwei Stunden mit fünf Minuten
  Abschlussreserve; `--deadline-utc` kann diese Frist nur verkürzen.
- Frisches benanntes Ausgabeverzeichnis, keine Überschreibung oder automatischen
  Wiederholungen. Fehlender Adapter/falsches Profil ist kein grüner Skip.
- Vorbereitete Binary muss zu den aktuellen Rustquellen/Manifesten passen;
  Änderungen verlangen einen ausdrücklichen Neubuild.
- Nur neue eigene Prozessgruppen/Geräte/Dateien. Eigene Stagingbuffer beziehungsweise
  das eigene logische Gerät werden in bezeichneten Negativfällen zerstört; keine
  OOM-, Hardware-/Treiberreset- oder fremden Prozessversuche.

## Inzwischen angeschlossener Produktschritt

Der nachfolgend beschriebene Slice ist jetzt als [Produktadapter](../../../docs/headless-plan/SLICE.md)
mit eigener Fixture implementiert. Die ursprüngliche Empfehlung lautete:

Separat freizugeben ist ein expliziter 2D/Metal-Imageziel-/Readback-Adapter am
bestehenden Captureowner: InitTick1, öffentlicher früher Fatal-Shortcircuit,
korrelierter Frame-/Mapvertrag, finaler Terminalcheck und typisierter technischer
Fehlerausgang. Native Defaults sowie vorhandene RGB-/Pfad-/Sandbox-/Temp-Rename-
Verträge bleiben bestehen. Das Proben-Directorybundle/RGBAformat nicht ungeprüft
übernehmen. Weitere grundsätzliche Machbarkeitsrunden sind dafür nicht nötig;
die Umsetzung samt gezielten Produktregressionen ist der nächste Schritt.

Der übergeordnete [Integrationsplan](../../../docs/headless-plan/PLAN.md) bleibt
historische Entscheidungs- und Umfangsbasis, keine pauschal bestandene Abnahme.
