# Lokaler Offscreen-Produktnachweis

Vier ausdrücklich gestartete, serielle **Metal-GPUfälle über die öffentliche
`woodpecker::session::Session`**. Keine Winit-/Clipboardfeatures, Fenster, Surface
oder OS-Eingaben. Kein Proben-Publisher: kopiert, mappt und schreibt durch den
neuen Produktadapter und den bestehenden Screenshot-Dateiowner.

## Voraussetzungen und Grenzen

macOS/Metal-Hardware, gecachte Rustdependencies oder ausdrücklich erlaubte normale
Cargo-Downloads. Die Featurefreigabe/Testberechtigung muss vor GPUausführung
vorliegen; `--allow-gpu` allein erteilt keine Rechte. Kein Softwarefallback, Skip
als Grün oder automatischer Retry. Native Vollapp-/Fensterverträge werden hier
nicht durch einen Headlesslauf ersetzt. Dieser Nachweis ergänzt sie separat.

Vom Repository-Root:

```sh
# Nur die vier Fälle auflisten; keine Dateien, Builds oder GPUausführung.
python3 -B tests/fixtures/offscreen/run.py plan

# Expliziter Offline-Build; maximal900s, startet keine Anwendung.
python3 -B tests/fixtures/offscreen/run.py build

# Erst nach GPUfreigabe; frischer Name, maximal120s pro Fall.
python3 -B tests/fixtures/offscreen/run.py run --allow-gpu --name local-product-proof
```

`build --online` erlaubt bewusst normale Cargo-Downloads, keine Toolchain- oder
Systeminstallation. `CARGO_TARGET_DIR` kann einen vorhandenen eigenen Buildcache
wählen. Der Runner legt Buildstamp, Binär-/Quellfingerprints und unveränderliche
Fallordner unter `target/headless-product-proof/` an. Eine veränderte Quelle oder
Binary verlangt einen expliziten Neubuild; nie automatische Reparatur/Fallback.

## Orakel

1. **healthy:** vor Tick1 typisierte Ablehnung ohne Datei; nach genau einem Warp-
   Tick rotes Mesh links und vier asymmetrische Marker im echten65×49-RGB-PNG.
   Während Pause identische Pixel und sämtliche Bevy-Uhren/Update-/Fixedzähler
   unverändert. Overwrite wahrheitsgemäß, `../` abgelehnt. Nach explizitem Tick2
   grünes Mesh rechts, exakt640 veränderte Pixel. Regulärer Shutdown.
2. **device-loss:** zuerst derselbe gesunde Pixel-/Pausekontrollfall; dann genau
   ein weiterer Tick und Destroy ausschließlich des eigenen logischen Geräts.
   Eine Dateibarriere stellt sicher, dass der Host die negative Captureanforderung
   bereits angenommen hat. Typisierter Fehler am Pending, entsprechender History-
   Ausgang und `EndReason::RenderFailed`; niemals die angeforderte Zieldatei.
3. **early-loss:** eigenes Device im InitTick1 vor der ersten Extraction zerstören.
   Typisiertes Sessionende statt Fortsetzen uninitialisierter Extraktionsressourcen.
4. **native-default:** ohne `OffscreenTarget` keine automatische Imageauswahl,
   selbst wenn eine Imagekamera existiert. Native Window-Unavailable-Ablehnung
   ohne Tick/Bild; `offscreen`-Capability bleibt aus.

Bevy kann bei DeviceLost bereits paniken, bevor sein asynchroner Handler den
Geräteverlust zustellt. Akzeptierte technische Ursachen sind deshalb ausdrücklich
`render_device_lost` oder der terminal abgefangene `render_panicked`, niemals ein
bloßes generisches `Ended` als Ersatz für den Fehlernachweis. Kein Hardware- oder
Treiberreset. Fehlerhafte Läufe und ihre Logs bleiben sichtbar erhalten.

Alle Fälle verwenden Manual-Startup; keine versteckten Vorbereitungs-/Ersatzticks.
Der Driver nutzt vorhandene Binarylaunch-Unterstützung, keine Cargo-Appstarts.
Das tatsächliche Rendererprofil wird protokolliert. Qualifiziert ist die feste
M1-Pro-/Metal-/Camera2d-/ColorMaterial-Konfiguration, nicht Vollspiel, 3D, UI,
Async-Spielassets, Engine-Startup, pipelined Rendering oder andere Plattformen.

Die drei neuen CPU-Ownerfälle gehören zum regulären Testinventar. Die vier
GPUfälle sind kein automatischer Teil der Standardsuite oder pausierten CI:

```sh
cargo test --locked --all-features --lib -- --test-threads=1
python3 -B -m unittest discover -s tests/runner_checks -t . -p '*.py'
```

[Umfang, Opt-in und finaler Erfolgsvertrag](../../../docs/headless-plan/SLICE.md).
