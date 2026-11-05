# Headless: aktueller Umsetzungsplan

**Stand: 2026-11-05 · geprüfte Produktbaseline `5493eb1` · PR [#23](https://github.com/timjonaswechler/woodpecker/pull/23), noch kein vollständig freigegebener Headlessmodus.**

Diese Datei ist die aktuelle Übersicht über **erledigte, teilweise umgesetzte und
offene Arbeit**. Der [Gesamtplan R05](PLAN.md) und [COVERAGE.json](COVERAGE.json)
bleiben historische Architektur-/Ownerreferenzen. Deren Angaben „alle Gates offen“,
„keine Implementierung“ und „planned-not-executed“ sind keine heutige Statusliste.
Der konkrete Betriebs-/Erfolgsvertrag des gelieferten Adapters steht in [SLICE.md](SLICE.md).

## Ziel und Statusregeln

Als nächste nutzbare Etappe soll die **echte Signalstation auf Metal ohne natives
Fenster** über dieselben Woodpecker-Commands gestartet, vorbereitet, bedient und
aufgenommen werden können. Der erste vollständige Anwendungsfall ist noch keine
Abnahme sämtlicher Funktionen, Grafikeffekte oder Plattformen.

- **Erledigt im Slice:** implementiert und für den ausdrücklich genannten Umfang
  ausgeführt/geprüft. Keine pauschale Abnahme des größeren R05-Pakets.
- **Teilweise:** verwertbare Implementation oder Evidenz liegt vor; benannte
  Integrations-/Abnahmeteile fehlen noch. Nicht erneut bei null anfangen.
- **Offen:** der Headless-Liefergegenstand oder sein erforderlicher Nachweis fehlt.
  Vorhandene native Funktionen werden damit nicht als unimplementiert bezeichnet.
- **Separat:** außerhalb der nächsten lokalen Etappe; kein versteckter Blocker
  für deren Abschluss und keine automatische Ausführungsfreigabe.

Native bleibt Default. Linux/Vulkan und Software-Rendering bleiben offene
Supportprofile aus dem Gesamtplan; ihr fehlender Nachweis widerlegt den Metalpfad
nicht. Zoo und die weitere Plattformqualifikation werden nicht aus dem Gesamtziel
gestrichen. CI-Reaktivierung und echter GitHub-/Providerlauf bleiben separat.

## Bereits erledigt im gelieferten Slice

| Liefergegenstand | Was tatsächlich vorhanden und belegt ist | Implementation / Evidenz |
| --- | --- | --- |
| Opt-in-Image-Capture | Feature `offscreen`, anwendungseigenes `OffscreenTarget`, Auswahl statt nativer Windowcapture; `offscreen:true` im Ready. Noch kein allgemeiner Headless-Launchmodus. | [Adapter](../../src/session/offscreen.rs), [Wirevertrag](../../src/session/protocol.rs) |
| Explizite Zeitsteuerung | Capture vor InitTick1 wird ohne Datei/Tick abgelehnt. Danach echte Bilder während Pause bei unveränderten Real-/Virtual-/Fixeduhren und Update-/Fixedzählern. Tick2 verändert die Szene. | [Rendergate](../../src/session/plugin/render.rs), [SDK-Fixture](../../tests/fixtures/offscreen/src/main.rs) |
| Readback und Veröffentlichung | Echte finale GPU-Texturkopie/Map, Request-Entitygeneration, RGB8-PNG, bestehender Pfad-/Sandboxschutz, Tempdatei/Sync/Rename. Finaler beobachteter Fatal-/Terminal-/Fristcheck; keine erneute Befristung bereits veröffentlichter Bilder beim späteren Poll. | [Capture](../../src/session/screenshot/capture.rs), [Dateiowner](../../src/session/screenshot/destination.rs), [CPU-Regressionen](../../src/session/screenshot/capture/tests/test_readback_lifecycle.rs) |
| Fehlertransport | Öffentlicher früher Fatalcheck, terminale Offscreen-Unwindbehandlung, `Error::Render`, `EndReason::RenderFailed` und technische History. Aktive und während Recordingtransition zurückgestellte Spielcommands werden typisiert beendet. | [Koordinator](../../src/session/coordinator.rs), [Fehler](../../src/session/error.rs), [Übergangsregression](../../src/session/tests/test_transition_races.rs) |
| 2D-Basisprofil | Camera2d/ColorMaterial, asymmetrische Geometrie, echte RGB-Pixel, Pause, Overwrite/Pfadfehler sowie eigener früher/später logischer Geräteverlust. Native Auswahl bleibt ohne impliziten Imagefallback. | [Fixture-Orakel](../../tests/fixtures/offscreen/README.md), [2D-Bilder](../../tests/fixtures/offscreen/evidence/README.md) |
| 3D-Basisprofil | Perspektiv-Camera3d, StandardMaterial, DirectionalLight, tatsächliche Flächenbeleuchtung und Schatten. Licht-/Schattengegenproben, Pause, Pose-/Materialwechsel und typisierter Geräteverlust. Kein neuer Produkt-Capturepfad nötig. | [3D-Orakel](../../tests/fixtures/offscreen/src/scene3d.rs), [3D-Bilder und Fingerprints](../../tests/fixtures/offscreen/evidence/3d/README.md) |
| Reproduzierbare lokale Teilabnahme | Acht GPU-/SDKfälle in gemeinsamer Serie bestanden; finale vier 3D-Fälle nochmals bestanden. 13 interne Owner- und 81 Harness-Tests sowie Workspace-/Fixture-Clippy bestanden. Historische Proben sind getrennt vom Produktnachweis. | [Expliziter Runner](../../tests/fixtures/offscreen/run.py), [historische Evidenz](../../tests/prototypes/headless-render-lab/evidence/README.md), [Testsuite](../../tests/README.md) |

**Qualifikationsgrenze:** Apple M1 Pro/Metal, macOS27.2, Bevy0.20.0/wgpu30.0.1,
65×49, nicht-pipelined; 3D mit asynchroner Pipelinekompilierung, festem HDR-/
Exposure-/Tonemappingprofil ohne temporale Effekte. Das ist weder ein Vollspiel-
noch ein allgemeiner PBR-/UI-/Asset-/Hardwarefehlertoleranznachweis.
Ein synchron kompilierender PBR-Lauf überschritt die 35-s-Pending-Beobachtung;
er bleibt Fehlbeleg, kein PASS und kein kontrollierter Kaltcache-Vergleich.

## Abgleich der bestehenden Arbeitspakete W00–W12

Die IDs stammen unverändert aus [R05, Abschnitt 11](PLAN.md#11-geordneter-integrationsfahrplan).
Der Status bezieht sich auf den **vollständigen dort beschriebenen Headlessumfang**,
nicht nur auf die vorstehend erledigten Slice-Liefergegenstände.

| Paket | Status | Erledigter Beitrag | Noch offen / konkreter Abschluss |
| --- | --- | --- | --- |
| W00 Scope/Proofspec | Teilweise | Gepinnter Slice, konkrete Quellen/Orakel und dieser Statusabgleich. | Aktuelles Testinventar mit den Originalownern der Vollapp-Etappe verbinden; deren Vorbereitung, Tickbudgets und Erfolgskriterien vor Ausführung festlegen. Kein neuer grundsätzlicher Render-Machbarkeitsauftrag. |
| W01 Version/Config | Teilweise | Rust-Opt-in, v3-`offscreen`-Capability und Renderterminaltransport vorhanden. | Einheitliche angeforderte/bestätigte Start-/Zielkonfiguration durch SDK, Server/CLI und App; Kompatibilitäts-/Fehlerfälle dieser Konfiguration. R05-v2/v4-Vorschläge sind nicht bereits implementierte oder zwingend nachzubauende Interfaces. |
| W02 Renderziel | Teilweise | Reales Imageziel und Format-/Copyadmission im engen Metalprofil; Renderer/Kamera werden in der Fixture korrekt installiert. | Appseitige fensterlose Installation, logische Geometrie/Scale und Eingabeziel verbinden; primäre Spielkamera samt tatsächlicher UI/Overlays anbinden und reale Spielauflösung qualifizieren. |
| W03 Kontrollierter Runner | Teilweise | Originale Control-/Warpowner, InitTick1-Gate, Idle-Rendering/Readback, öffentlicher Fatalhook und Uhrentrennung belegt. | Vollapp-Asset-/Message-/Change-/Removed-Lifecycle und deren Zusammenspiel mit der Renderiteration nachweisen; keine beliebigen SubApps oder pipelined Profile mitbehaupten. |
| W04 Ready | Offen | Manual-Handshake und InitTick1-Capture sind nutzbar. | `EngineReady` an Imageziel und tatsächliche Spielassets/-szene statt PrimaryWindow binden; Ladefortschritt, Freeze, Zeit-/Ticklimits und Fehler für die echte App nachweisen. |
| W05 Capture | Teilweise | Produkt-RGB-Copy/Map-/Dateipfad, Korrelation, Publicationcheck und kleine echte 2D-/3D-Profile erledigt. | Zusammenschluss mit Vollapp-Ready, realer Zielgeometrie/UI und den noch offenen Lifecyclefällen abnehmen. Kleine Bilder und zwei Szenen sind keine allgemeine Frame-/Appabnahme. |
| W06 Input/UI | Offen | Native Eingabeowner und ihre bisherigen Verträge existieren. | Fensterlose Key-/Textidentität, Image-Pointerlocation, Picking, Legacy-/Widget-UI, Fokus, Layout/Clipping und sichtbare Glyphen implementieren/prüfen. |
| W07 Signalstation/Persistenz | Offen | Native App, Inspect, Recording/Replay, Reports und Clients sind wiederverwendbare Grundlage; grundlegender PBR-Pfad funktioniert. | Echte Signalstation mit ihren Assets, Kameras, UI, Settings-/Clipboardgrenzen und genutzten Effekten headless betreiben. Originale Input-/Spiel-/Persistenz-/Clientowner zusätzlich in diesem Profil ausführen. |
| W08 Isolation/Abschluss | Teilweise | Typisierte Gerätefehler, aktive/deferred Requests und einzelne Writer-/Dateibarrieren geprüft. | Vollapp-Mehrsession, Startabbruch, Shutdown/Drop, konkurrierende Captures/Writer und vollständiger Prozess-/Dateiabschluss mit bestehenden Budgets. Keine physische GPUpartitionierung versprechen. |
| W09 Zoo | Offen | Vorhandener Imageexport ist wiederverwendbare Grundlage. | Hidden Window durch echten fensterlosen Export ersetzen; Ziel-/Assetbereitschaft und Captureack statt fester Framewartewerte; vollständiger Motiv-/Ansichts-/Label-/Shaderumfang. |
| W10 Tests/Entwicklereinstiege | Teilweise | Drei neue CPU-Ownerfälle registriert; separate explizite Acht-Fälle-GPU-Fixture mit Fingerprints, Profilwahl und Belegen. | Originalowner/Orakel headless zusätzlich anschließen; gemeinsame App-/Reset-/Lebensdauerpolitik und direkte Beispiele/Messwerkzeuge integrieren. Keine native Abnahme als Headless umbenennen. |
| W11 Plattform/Software | Teilweise | Enges M1-Pro-/Metal-/65×49-2D-/Basis-3D-Profil tatsächlich qualifiziert. | Zuerst vollständiges Metal-Appprofil in realer Größe; weitere Adapter/Versionen/Größen, Linux/Vulkan und Software separat entscheiden/qualifizieren. Keine stillen Fallbacks oder Featureabschaltungen. |
| W12 Abschluss | Offen | Slice-Vertrag, Anleitung, Grenzen und Evidenz sind vorhanden. | Endgültige Support-/Abnahmematrix, Migration und verbleibende native Aussagen für den gewählten Produktumfang zusammenführen. Kein automatischer Merge, Release oder CI-Neustart. |

**Wichtig:** Ein Beitrag zu W02/W03/W05 ist kein pauschaler Abschluss von
W01–W05. Umgekehrt müssen funktionierender Readback, echter 3D-Schattenwurf oder
der verfügbare öffentliche Fehlerhook nicht erneut als ungelöste Machbarkeit
untersucht werden. Neue Architekturpfade/Versionsnummern nur für konkrete
verbleibende Anforderungen einführen, nicht zur nachträglichen Nachbildung von R05.

## Gate-Abgleich statt „alle noch offen“

Diese Einordnung ersetzt für heutige Entscheidungen die historische Statuszeile
in [R05, Abschnitt 12](PLAN.md#12-offene-proof-entscheidungsgates). Sie ist keine
rückwirkende Änderung der R05-Reviewzustimmung oder eine komplette Gate-Abnahme.
Die [Proof-IDs P01–P22](PLAN.md#102-proofkatalog-alle-geplant-keine-ausführung)
bleiben als Referenzen erhalten; Einzelbefunde schließen nicht automatisch deren
gesamte früher geplante Ausführungsmenge.

| Gate | Heute | Verbleibende Aussage |
| --- | --- | --- |
| G01 Kompatibilität | Teilweise | Native Ready bleibt unverändert; aktueller v3-Opt-in/Terminalpfad vorhanden. Vollständiger angeforderter Launch-/Client-/Renderkonfigurationsvertrag fehlt. |
| G02 Runner | Teilweise | Enger Tick-/Pause-/Hookpfad ist belegt. Vollapp-Lifecycle und tatsächlich benötigte Asset-/SubApp-Profile fehlen. |
| G03 Ready/Graph | Offen für EngineReady | Erfolgreiches Capture ist nicht die noch fehlende Image-/Vollapp-Startupabnahme. |
| G04 Readback | Teilweise | Enger Produktpfad und Fehler-/Dateiregressionen sind belegt; vollständige App-/Ziel-/Lifecyclekombinationen fehlen. |
| G05 UI/Geometrie | Offen | Logische Geometrie, Image-Picking, Fokus und echte UI-/Textwirkung ohne natives Fenster fehlen. |
| G06 Adapter/Plattform | Teilweise | Benanntes Metal-Basisprofil belegt; übrige Profile nicht qualifiziert, nicht widerlegt. |
| G07 GPUfeatures/Assets | Teilweise | 2D und grundlegendes PBR/Licht/Schatten belegt; tatsächliche Spielassets, Fonts, UI, Overlays und weitere genutzte Effekte fehlen. |
| G08 Native Dienste | Teilweise | Kleine Fixture ohne Winit/Clipboard/native Fenster belegt. Signalstation besitzt noch native Window-/Winit-/Clipboardkopplungen. |
| G09 Abschluss/Device-loss | Teilweise | Eigene logische Gerätefehler und Publicationbarrieren belegt. Vollständiger Mehrsession-/Start-/Writer-/Shutdownverbund fehlt. |
| G10 Zoo | Offen | Vorhandener Capturemodus erzeugt noch ein verborgenes natives Fenster. |
| G11 Abnahmevertrag | Teilweise | Lokale Slice-/Ownerprüfungen vorhanden; Originalfallmatrix und vollständige Headless-Ausführung nicht abgeschlossen. |
| G12 Externe Veröffentlichung | Separat | Echter Provider-/GitHublauf bleibt eigener Auftrag, kein lokaler Headless-/GPUblocker. |

Der Erfolgsvertrag bleibt endlich: korrektes bezeichnetes CPUbild und kein am
letzten Publicationcheck **beobachteter** Fatal. Keine atomare Kenntnis sämtlicher
zukünftiger/unbeobachteter Backendfehler und kein echter Treiberreset als neues
Pflichtgate. Einzel-Screenshot-Cancel wird nicht als neue Funktion vorausgesetzt;
`Pending`-Drop bleibt keine Cancellation.

## Konkrete noch offene Kopplungen

Diese Befunde wurden am oben genannten Quellstand geprüft:

| Quelle | Heutiger Befund | Folgearbeit |
| --- | --- | --- |
| [Launch](../../src/session/launch.rs), [CLI-Konfig](../../src/cli/config.rs) | Kein vereinheitlichter Headless-/Renderzielauftrag; die Fixture setzt Renderer/Image/Kamera selbst zusammen. | W01/W02 |
| [Startup](../../src/session/startup.rs), [Render-Ticket](../../src/session/startup/render.rs) | `after_tick`/`Ticket` verlangen PrimaryWindow und Fensterframe. | W04 |
| [Keyboard](../../src/session/input/keyboard.rs), [Pointer](../../src/session/input/pointer/cursor.rs), [Text](../../src/session/input/text.rs), [UI](../../src/session/input/ui.rs) | PrimaryWindow-Bindung; Pointerlocation und Legacy-UI sind fensterbezogen. | W06 |
| [Appinstallation](../../test_app/src/app/mod.rs), [Woodpecker-Anbindung](../../test_app/src/woodpecker.rs), [Appfeatures](../../test_app/Cargo.toml) | Echte App erzeugt Window, setzt WinitSettings und führt native Featurepfade; grundlegende Renderfixture ersetzt das nicht. | W02/W07/G08 |
| [Zoo](../../test_zoo/src/app.rs) | `--zoo-capture` setzt `visible:false`, installiert aber weiterhin ein Window. | W09 |
| [Performanceeinstieg](../../examples/performance.py) | `raise_window`, `osascript`, Displayabfragen und eigener Requestdispatcher sind noch nicht als Headless-Einstieg integriert. | W10 |

## Nächste Etappe: ein echter Anwendungsfall, noch offen

**Ziel:** Signalstation headless starten → SessionReady und kontrollierte
Spiel-/Enginebereitschaft unterscheiden → tatsächlichen UIbutton per Command
betätigen → geänderten Spielzustand und reales PNG nachweisen → sauber beenden.
Das ist ein bleibender Produktslice, keine weitere Wegwerfgrafikdemo.

1. **Start-/Zielkonfiguration (W01/W02):** Modus früh in der echten App auswählen,
   Window/Winit und Systemclipboard nicht initialisieren; eigene Workspace-/Seed-
   und Targetdaten validieren. Native Defaults und die rendererfreie Hosttrennung
   erhalten. Für die erste Appetappe feste reale Größe/Scale bestimmen; 65×49
   wird nicht als Vollspielprofil ausgegeben.
2. **Image-Ready/Assets (W04, Anteile W03/W07):** vorhandene Assetliste und
   Spielladezustände weiterverwenden. Lade-/Startup-Ticks sind nur ausdrücklich
   konfiguriert/angefordert und vollständig gezählt erlaubt. GPUvorbereitung,
   Fristen und Fehler an das richtige Imageziel binden, keine zusätzlichen
   `App.update`-/Reparaturticks oder Deadlineverlängerung zum Grünmachen.
3. **Logisches Input-/UIziel (W06):** Key/Text/Fokus und Image-Pointer/Picking mit
   einer gemeinsamen Geometrie verbinden. Wirkungen erst am expliziten Tick,
   weder Renderpump noch Inspect/Capture dürfen Inputs verbrauchen.
4. **Original-Appfall anschließen (W07/W10):** den Owner
   [`tests.pointer.ui_click`](../../tests/pointer/ui_click.py) mit seiner wirklichen
   Vorbereitung und seinen unveränderten Zustands-/Tickassertions verwenden:
   „Gesamtkarte [G]“, `ViewState` Overview → Map, Move/Press/Release mit drei
   Aktionsticks. Bild-/Layoutbeleg und sauberer Abschluss kommen hinzu; der
   native Owner wird nicht durch eine reduzierte Headlesskopie ersetzt.

### Abschlusskriterien dieser Etappe

- [ ] Derselbe echte App-/Asset-/UIumfang startet im gewählten Metalprofil ohne
  natives Fenster, Display-/Fokus-/Clipboardaktion oder stillen Ersatzprozess.
- [ ] SessionReady ist nicht fälschlich Spiel-/Enginebereitschaft; explizite
  Vorbereitung, Seed, Auflösung/Scale und sämtliche Ticks sind protokolliert.
- [ ] Move/Press/Release haben vor dem Tick keine Wirkung; danach zeigt Inspect
  den erwarteten Spiel-/UIzustand und Capture die wirkliche Szene samt UI.
- [ ] Capture/Inspect während Pause erhalten Uhren/Ticks; kein synthetisches Bild
  oder abgeschwächtes Originalorakel ersetzt den Produktnachweis.
- [ ] Fehler werden typisiert beendet, Abschluss sammelt eigene Prozesse und
  berücksichtigt Writer/Dateiwirkungen; native Defaults bleiben erhalten.

**Danach, nicht dadurch erledigt:** restliche Keyboard-/Pointer-/Textowner,
Recording/Replay/Reports/Clients, weitere Lebensdauer-/Mehrsessionfälle, Zoo,
Entwicklereinstiege und zusätzliche Supportprofile. Erst deren vorgesehene
Abnahmen erlauben die jeweils weitergehende Supportaussage.

## Pflege und nächste Aktualisierung

Nach jeder Etappe hier Produktbaseline, erledigten Umfang, W-/Gatezeilen und
verbleibenden nächsten Schritt nachziehen. Neue Evidenz mit eigenem Source-/
Profilbezug verlinken; alte Fehlbelege und native Assertions erhalten. Eine
Quellprüfung ist kein Runtime-PASS. Keine Prozentzahl oder Komplettfreigabe aus
bloßer Existenz von Code ableiten.

Diese Aktualisierung ist ein Dokumentations-/Statusabgleich. Sie erzeugt keine
neuen GPU-/Appnachweise und erteilt keine Freigabe für CI, Installationen,
Provideraktionen oder weitere Laufzeit-/Produktänderungen.
