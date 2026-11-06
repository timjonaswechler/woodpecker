# Lokaler Offscreen-Produktnachweis

Acht ausdrücklich gestartete, serielle **Metal-GPUfälle über die öffentliche
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
# Nur die acht Fälle auflisten; keine Dateien, Builds oder GPUausführung.
python3 -B tests/fixtures/offscreen/run.py plan

# Expliziter Offline-Build; maximal900s, startet keine Anwendung.
python3 -B tests/fixtures/offscreen/run.py build

# Erst nach GPUfreigabe; frischer Name, maximal120s pro Fall.
python3 -B tests/fixtures/offscreen/run.py run --allow-gpu --name local-product-proof

# Gezielt nur die vier 3D-Fälle samt Licht-/Schattenvergleich.
python3 -B tests/fixtures/offscreen/run.py run --allow-gpu --profile 3d --name local-3d-proof
```

`plan --profile 2d|3d` zeigt die jeweilige Auswahl; ohne Profil laufen alle acht.
Jeder Aufruf verlangt einen frischen Namen und startet keine automatischen Neubuilds.
`WOODPECKER_OFFSCREEN_ARTIFACT_DIR=target/NEUER_NACHWEIS` wählt einen separaten
Evidenzroot; so bleiben frühere `build-latest.json` und Kampagnen unverändert.
`CARGO_TARGET_DIR` bleibt ein veränderlicher Compilercache; wichtige frühere
Programme vor einem Neubuild als unveränderliche Kopie mit Hash aufbewahren.

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

5. **healthy-3d:** Perspektiv-Camera3d, StandardMaterial-Cuboid, Boden und
   DirectionalLight mit echter Schattenmap. RGB-/Geometrie-/Beleuchtungsorakel,
   identisches Bild und unveränderte Uhren während Pause. Tick2 bewegt den Würfel
   nach rechts und ändert ihn von rot auf grün.
6. **shadow-off-3d:** dieselbe feste Tick1-Szene ohne Schatten. Der nachgelagerte
   Pixelvergleich verlangt tatsächlich abgedunkelte graue Bodenpixel in Fall5;
   ein gesetztes Shadowflag allein reicht nicht.
7. **light-off-3d:** dieselbe Szene ohne direktes Licht, Ambient und Emission sind
   ebenfalls null. Tatsächlich schwarzer RGBframe als Beleuchtungsgegenprobe.
8. **device-loss-3d:** erst gesunder 3D-Pixel-/Pausekontrollfall, dann eigenes
   Device zerstören; typisierte Pending-/History-/EndReason-Diagnose ohne Zieldatei.

Nach einer 3D-Auswahl prüft `audit-3d` die echten RGBdateien der drei Kontrollfälle.
Beleuchtung, Schattenwirkung und identische Würfelgeometrie werden unabhängig
von der Szenenkonfiguration verglichen. Mindestens6 Bodenpixel müssen um20
RGBstufen abgedunkelt sein; im aufgezeichneten Profil waren es27.

Bevy kann bei DeviceLost bereits paniken, bevor sein asynchroner Handler den
Geräteverlust zustellt. Akzeptierte technische Ursachen sind deshalb ausdrücklich
`render_device_lost` oder der terminal abgefangene `render_panicked`, niemals ein
bloßes generisches `Ended` als Ersatz für den Fehlernachweis. Kein Hardware- oder
Treiberreset. Fehlerhafte Läufe und ihre Logs bleiben sichtbar erhalten.

Alle Fälle verwenden Manual-Startup; keine versteckten Vorbereitungs-/Ersatzticks.
Der Driver nutzt vorhandene Binarylaunch-Unterstützung, keine Cargo-Appstarts.
Das tatsächliche Rendererprofil wird protokolliert. Qualifiziert ist die feste
M1-Pro-/Metal-/65×49-Konfiguration: Camera2d/ColorMaterial sowie das beschriebene
Camera3d-/StandardMaterial-Basisprofil. 3D nutzt asynchrone Pipelinekompilierung,
HDR-Zwischenziel, Tonemapping::None, feste Exposure und Hardware2x2-Schattenfilter
mit einer512px-Map/einer Kaskade; MSAA, Dither und temporale Effekte sind aus.
Kein allgemeiner PBR-/Posteffekt-, Vollspiel-, UI-, Async-Spielasset-,
Engine-Startup-, pipelined- oder Plattformnachweis.

Ein initialer synchron kompilierender 3D-Lauf endete an der35-s-Pending-Beobachtung,
nicht mit einem Bild-PASS. Er bleibt lokal erhalten. Danach wurde die asynchrone
Konfiguration explizit geprüft. Keine Deadline-/Orakellockerung, keine versteckten
Ticks oder Cachelöschung; dies ist kein kontrollierter Kaltcache-Zeitvergleich.

Die drei neuen CPU-Ownerfälle gehören zum regulären Testinventar. Die acht
GPUfälle sind kein automatischer Teil der Standardsuite oder pausierten CI:

```sh
cargo test --locked --all-features --lib -- --test-threads=1
python3 -B -m unittest discover -s tests/runner_checks -t . -p '*.py'
```

[Umfang, Opt-in und finaler Erfolgsvertrag](../../../docs/headless-plan/SLICE.md).
