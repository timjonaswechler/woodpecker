# Erster Produkt-Slice: explizites Image-Capture

[STATUS.md](STATUS.md) führt den aktuellen Gesamtarbeitsplan und die noch offenen
Pakete. Diese Datei beschreibt den gelieferten Captureadapter und dessen
belegten Betriebsumfang, nicht die vollständige Headless-Abnahme.

Der enge 2D-/Basis-3D-Metalpfad ist im **Produkt-Captureowner** angeschlossen, nicht
nur im Wegwerfprüfstand. Native bleibt unverändert Standard. Dieser Slice ist
kein fensterloser Umbau der Signalstation und keine Vollspiel-/Plattformabnahme.

## Opt-in für Anwendungen

1. `woodpecker` mit Feature `offscreen` bauen; es schließt `screenshot` ein.
2. Selbst einen fensterlosen Metal-Hardwarerenderer installieren: keine primäre
   Window/Surface, kein Winit-Plugin, kein PipelinedRenderingPlugin. Das Resource-
   Opt-in erzeugt und deaktiviert selbst keine Fenster oder Renderer.
3. Ein single-sampled D2-Image mit `Rgba8UnormSrgb`, positiver Größe,
   RENDER_ATTACHMENT- und COPY_SRC-Usage erzeugen. Eine aktive Camera2d oder
   einfache Camera3d darauf richten, MSAA ausschalten und das Image während der
   Aufnahme nicht ersetzen. Das 3D-Basisprofil nutzt asynchrone Pipelinekompilierung.
4. **Vor** `session::Plugin` das ausgewählte Handle als Resource einsetzen:

```rust
app.insert_resource(woodpecker::session::OffscreenTarget(image.clone()));
app.add_plugins(woodpecker::session::Plugin);
```

Die vollständige ausführbare Konfiguration steht in der
[Produkt-/SDK-Fixture](../../tests/fixtures/offscreen/src/main.rs).
Manual-Startup und ein ausdrücklich angeforderter InitTick1 sind der geprüfte
Startvertrag. Capture davor wird mit `render_initial_tick_required` abgelehnt,
ohne Tick oder Dateiänderung. Danach verwenden Aufrufer den normalen
`screenshot::Capture`-Command. Ein neuer Headless-CLI-/Launchschalter ist nicht
Teil dieses Slices.

Native-Ready bleibt bytegleich. Der Opt-in-Modus meldet zusätzlich
`capabilities.offscreen:true`; dadurch lehnt ein alter strikter Host diesen
neuen Modus bereits beim Handshake ab, statt seinen Fehlertransport zu verkennen.
`capabilities.screenshot` bestätigt Installation, nicht Tick0-Bildbereitschaft.

## Was der Adapter übernimmt

- [Rendergate](../../src/session/plugin/render.rs): öffentlicher `pre_extract`-
  Fatalcheck, ohne nach beobachtetem Fatal in uninitialisierte Extraction zu laufen.
- [Imageadapter](../../src/session/offscreen.rs): tatsächliche Texturkopie nach
  Graph-Submit, nicht ein Clear-/CPU-Ersatz. Kopiert erst bei vorbereitetem View
  und stabil fertig kompilierten Pipelines. Retrybare Shader-/Import-Warm-up-
  Zustände bleiben bis zur normalen Capturefrist pending, statt die Session zu beenden. Der Mapcallback trägt die eindeutige
  Request-Entitygeneration der konkreten Kopie; verspätete Ergebnisse dürfen
  keinen neueren Request abschließen.
- Nur technische Asset-Task-/Shaderladeprogression während Pause; keine
  Anwendungsschedules, Timeupdates oder synthetischen Shaderereignisse an
  Anwendungleser. Neue geladene Shader werden direkt an den PipelineCache
  übergeben. Kein allgemeiner Asset-/Hotreload-Readinessvertrag.
- [Capture-/Dateiowner](../../src/session/screenshot/capture.rs): weiterhin
  seriell, asynchrones Encoding/IO, echtes **RGB8-PNG**, bestehender Root-
  Capability-/Pfad-/Symlinkschutz und Tempdatei-Rename. Kein RGBA-/Directorybundle
  aus dem Wegwerf-Publisher wird übernommen.
- Finaler Check nach temporärem Schreiben/Sync vor Rename: beobachteter Fatal,
  Sessionende und 30-s-Frist. Nur Offscreen umfasst diese Frist auch Encoding/
  Schreiben; die native Readback-Frist bleibt unverändert. Ein bereits erfolgreich
  erfolgter Rename wird beim späteren Control-Poll nicht rückwirkend befristet.

Erfolg bestätigt diesen abgeschlossenen Frame und keinen **am letzten Ownercheck
beobachteten** Fatal. Keine atomare Kenntnis jedes Backendereignisses und keine
Garantie zukünftiger Gerätegesundheit. Bereits vor einem späteren Fatal korrekt
veröffentlichte CPU-Pixel werden nicht rückwirkend ungültig.

## Fehler und Terminalität

Beobachtete Rendererfehler werden als `render_failed` über denselben v3-Writer
zugestellt. Die Session endet mit `EndReason::RenderFailed { error }`; offene
Spielcommands einschließlich während einer Recordingtransition zurückgestellter
Commands erhalten `Error::Render { code, message }`. Kein zusätzlicher Tick ist
zur Beobachtung notwendig. Bereits vorher abgeschlossene Antworten bleiben gültig.

DeviceLost-Zustellung kann einer bereits fehlschlagenden GPU-Operation nachlaufen.
Deshalb fängt nur dieser Opt-in-Pfad Unwinds in RenderStartup/Render-Schedule ab
und endet mit `render_panicked`, statt den beschädigten Renderlauf fortzusetzen.
Das ist keine Recovery; Aborts und echte OOM-/Treiber-/Hardwareabstürze sind nicht
abfangbar zugesichert. Native Panic-/Errorpolicy bleibt unverändert.

Vor dem finalen Check beobachtetes EOF beziehungsweise Wire-Shutdown sperrt die
Veröffentlichung ebenfalls. `Pending`-Drop ist weiterhin **keine** Cancellation;
der normale SDK-Shutdown verweigert sich bei offenen Commands. Es wird kein neuer
Screenshot-Cancelcommand eingeführt.

## Prüfung und Wiederholung

Die [Fixture-Anleitung](../../tests/fixtures/offscreen/README.md) beschreibt
explizite lokale Builds und acht begrenzte GPUfälle über die echte öffentliche
`Session`: RGB/Geometrie, alle Uhren während Pause, Tick2-Änderung, Overwrite,
Pfadfehler, später und früher eigener logischer DeviceLost sowie unveränderte
native Auswahl ohne Imagefallback sowie vier 3D-Fälle mit Perspektivkamera,
StandardMaterial, DirectionalLight und tatsächlichem Schattenwurf. Kein natives
Fenster oder OS-Input.

Der Gerätverlust-Gegenfall wurde zuerst mit generischem `Ended` reproduziert;
der terminale Unwindpfad schließt diese konkret beobachtete Lücke. Die früheren
Rohbelege bleiben lokal erhalten. CPU-Ownerregressionen prüfen zusätzlich den
finalen Dateicheck nach echtem Staging, veraltete Readbacks, Deadline-/Fatal-/
Terminalguards und unveränderte native Ready-Serialisierung. Drei neue interne
Ownerfälle sind im vorhandenen Acceptance-Inventar registriert; kein Runner-
Schedulingumbau oder automatischer GPUlauf.

### Zusätzlich qualifiziertes 3D-Basisprofil

Die [3D-Fixture](../../tests/fixtures/offscreen/src/scene3d.rs) verwendet
Perspective-Camera3d, einen bewegten Cuboid mit StandardMaterial, eine Bodenfläche,
ein DirectionalLight und einen512px-Schattenatlas mit einer Kaskade. HDR-
Zwischenziel, Tonemapping::None, feste Exposure, MSAA/Dither/TAA aus; Ausgabe
weiterhin RGB8-PNG. Licht-aus und Schatten-aus sind separate echte Renderläufe:
Der Vergleich belegt Lichtabhängigkeit und27 tatsächlich abgedunkelte Bodenpixel,
nicht nur gesetzte Komponentenflags. Pause erhält Bild und sämtliche Uhren;
Tick2 verschiebt den Würfel nach rechts und ändert sein Material auf grün.
Ein eigener logischer DeviceLost wird auch nach gesundem 3D-Capture typisiert
beendet, ohne fehlgeschlagene Zieldatei.

Dafür war kein anderer Produkt-Capturepfad nötig. Ein erster synchron
kompilierender PBR-Lauf überschritt die35-s-Beobachtungsfrist; er bleibt als
Fehlbeleg erhalten. Qualifiziert ist die danach explizit gewählte asynchrone
Pipelinekonfiguration, nicht synchroner oder kalter Startup jeder 3D-Anwendung.
Keine Frist wurde verlängert, kein Warm-up-/Reparaturtick hinzugefügt und kein
System-/Treiber-Shadercache gelöscht. Die Läufe sind kein Kaltcache-Performancevergleich.

Geprüft: Apple M1 Pro/Metal, macOS27.2, Bevy0.20.0, wgpu30.0.1,65×49, Camera2d/
ColorMaterial und das beschriebene Camera3d-/StandardMaterial-Basisprofil,
nicht-pipelined. Andere Größen, weitere PBR-/Posteffekte, UI/Text, Async-Spielassets,
Linux/Softwareadapter, Input ohne Fenster, Engine-Startup und Vollspiel bleiben
separat zu qualifizieren. Das ist fehlende Abdeckung, kein Nachweis ihrer
Unmöglichkeit. CI bleibt pausiert.
