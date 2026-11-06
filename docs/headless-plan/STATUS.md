# Headless: aktueller Umsetzungsplan

**Stand: 2026-11-06 · L1 abgeschlossen, zwei unabhängige Reviews APPROVE · PR [#23](https://github.com/timjonaswechler/woodpecker/pull/23).**
Capturebasis `5493eb1`, L1-Code `d4460e3` (Ausgangsstand `f9a74f6`); die exakten ausgeführten
Runtimequellen und Belege stehen in [L1.md](L1.md). Kein vollständiger L2–L5-Headlessmodus.

Diese Datei ist die aktuelle Übersicht über **erledigte, teilweise umgesetzte und
offene Arbeit**. Der [Gesamtplan R05](PLAN.md) und [COVERAGE.json](COVERAGE.json)
bleiben historische Architektur-/Ownerreferenzen. Deren Angaben „alle Gates offen“,
„keine Implementierung“ und „planned-not-executed“ sind keine heutige Statusliste.
Der konkrete Betriebs-/Erfolgsvertrag des gelieferten Adapters steht in [SLICE.md](SLICE.md).

## Ziel und Statusregeln

Als nächstes vollständiges Lieferpaket soll die **echte Signalstation auf Metal
ohne natives Fenster** gestartet, vorbereitet, aufgenommen und mit dem vereinbarten
Keyboard-/Pointer-/Text-/UIumfang bedient werden können. Ein einzelner erfolgreicher
UI-Klick ist dabei ein Zwischencheck, kein Paketabschluss. Andere Funktionen,
Grafikeffekte und Plattformen werden als eigene vollständige Pakete geliefert.

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

## Arbeitsregel: erst abschließen, dann weiter

Ab jetzt wird **höchstens ein Produkt-Lieferpaket gleichzeitig** bearbeitet.
Die vorhandenen „Teilweise“-Zeilen weiter unten bleiben ein ehrlicher
Bestandsabgleich, sind aber keine Vorlage für weitere halbfertige Lieferungen.

1. **Umfang vor Beginn festlegen:** unterstütztes Profil, sichtbares Ergebnis,
   zugehörige Originalowner und Abschlusskriterien. Nutzbar abgegrenzter Umfang,
   nicht nur ein interner Baustein oder ein nachträglich kleiner gerechnetes Ziel.
2. **Ende zu Ende fertigstellen:** Implementation, Integration, Fehlerpfade,
   erforderliche Regressionen und Anleitung gehören zum selben Paket.
   Zwischenschritte und kleine Commits sind erlaubt, aber kein Fertigstatus
   nur wegen eines erfolgreichen Happy-Path-/PoC-Laufs.
3. **Erst reparieren, dann erweitern:** fehlende oder fehlschlagende notwendige
   Tests, Cleanup, Kompatibilität und Reviewfindings im Paket beheben. Keine
   dafür nötigen Restarbeiten als TODO oder „später“ ins nächste Paket schieben;
   keine zweite provisorische Implementation neben dem Produktpfad stehen lassen.
4. **Abschluss nachweisen:** aktuelle Quellen/Profil und vollständige Kriterien
   prüfen, relevante Reviewfindings auflösen, Dokumentation/Status aktualisieren.
   Bestehende Orakel nicht lockern oder überspringen, um einen Abschluss zu melden.
5. **Blockiert ist nicht fertig:** bei fehlender Freigabe, Zeit oder ungelöstem
   Hindernis den konkreten Rest und Blocker melden. Nicht still das nächste Paket
   anfangen oder den Umfang nachträglich zur grünen Kennzeichnung verkleinern.

Ein Lieferpaket ist erst **abgeschlossen**, wenn sein vereinbartes Ergebnis
benutzbar integriert ist, alle erforderlichen Positiv-/Negativ- und
Regressionsnachweise vorliegen und keine notwendige Restarbeit dieses Umfangs
offen ist. Externe Aktionen und zusätzliche Profile bleiben nur dann außerhalb,
wenn das schon vor Beginn ausdrücklich so abgegrenzt war.

### Verbindliche Lieferreihenfolge

| Lieferpaket | Zustand | Vollständiger Liefergegenstand / Ende |
| --- | --- | --- |
| L1 Signalstation-Basisbetrieb auf Metal | Abgeschlossen | Start-/Zielkonfiguration, echte Assets/Image-Ready, vollständiger vereinbarter Input/UI/Textumfang, Capture und Einzel-Session-Abschluss; alle zehn ursprünglichen Inputowner zusätzlich headless geprüft. Ein einzelner UI-Klick reicht nicht. Details unten. |
| L2 Persistenz und sämtliche Einstiegspunkte | Offen – nicht begonnen | Inspect-/Recording-/Replay-/Reportverträge sowie SDK, Server/Clients, CLI/REPL, Scripts und Messwerkzeuge durchgängig im Appprofil; zugehörige Originalowner und direkte Fehler-/Terminalpfade bestanden. Echter externer Providerlauf bleibt separat. |
| L3 Isolation, Lebensdauer und Gesamtabnahme | Offen – nicht begonnen | Mehrsession, Startabbruch, Shutdown/Drop/Writer-/Gerätefehlerverbund sowie vollständige betroffene Test-/Wiederverwendungs-/Cleanup-Verträge; keine notwendige Restarbeit des zugesicherten Metalumfangs. |
| L4 Zoo | Offen – nicht begonnen | Vollständiger fensterloser Motiv-/Ansichtsexport samt Assets, Shadern, Labels, Bereitschaft und Dateiquittung; bestehender nativer Viewer bleibt erhalten. |
| L5 Weitere Supportprofile | Offen – nicht begonnen | Jeweils ein zuvor ausgewähltes Profil (etwa Linux/Vulkan oder Software) vollständig mit seinen erforderlichen Funktionen und Nachweisen abschließen, dann erst das nächste Profil. |

Die L-Pakete sind die **Ausführungsreihenfolge**, keine neue Architektur und keine
nachträgliche Umbenennung offener Arbeit in „erledigt“. Die W-/G-Matrix darunter
bleibt die querschnittliche Zuordnung zum Gesamtumfang. Sie wird nach jedem
abgeschlossenen L-Paket anhand tatsächlicher Evidenz nachgezogen. CI-Neustart,
Merge und externe Veröffentlichung werden dadurch nicht automatisch freigegeben.

## Bereits erledigt im gelieferten Slice

| Liefergegenstand | Was tatsächlich vorhanden und belegt ist | Implementation / Evidenz |
| --- | --- | --- |
| Opt-in-Image-Capture | Feature `offscreen`, anwendungseigenes `OffscreenTarget`, Auswahl statt nativer Windowcapture; `offscreen:true` im Ready. Noch kein allgemeiner Headless-Launchmodus. | [Adapter](../../src/session/offscreen.rs), [Wirevertrag](../../src/session/protocol.rs) |
| Explizite Zeitsteuerung | Capture vor InitTick1 wird ohne Datei/Tick abgelehnt. Danach echte Bilder während Pause bei unveränderten Real-/Virtual-/Fixeduhren und Update-/Fixedzählern. Tick2 verändert die Szene. | [Rendergate](../../src/session/plugin/render.rs), [SDK-Fixture](../../tests/fixtures/offscreen/src/main.rs) |
| Readback und Veröffentlichung | Echte finale GPU-Texturkopie/Map, Request-Entitygeneration, RGB8-PNG, bestehender Pfad-/Sandboxschutz, Tempdatei/Sync/Rename. Finaler beobachteter Fatal-/Terminal-/Fristcheck; keine erneute Befristung bereits veröffentlichter Bilder beim späteren Poll. | [Capture](../../src/session/screenshot/capture.rs), [Dateiowner](../../src/session/screenshot/destination.rs), [CPU-Regressionen](../../src/session/screenshot/capture/tests/test_readback_lifecycle.rs) |
| Fehlertransport | Öffentlicher früher Fatalcheck, terminale Offscreen-Unwindbehandlung, `Error::Render`, `EndReason::RenderFailed` und technische History. Aktive und während Recordingtransition zurückgestellte Spielcommands werden typisiert beendet. | [Koordinator](../../src/session/coordinator.rs), [Fehler](../../src/session/error.rs), [Übergangsregression](../../src/session/tests/test_transition_races.rs) |
| 2D-Basisprofil | Camera2d/ColorMaterial, asymmetrische Geometrie, echte RGB-Pixel, Pause, Overwrite/Pfadfehler sowie eigener früher/später logischer Geräteverlust. Native Auswahl bleibt ohne impliziten Imagefallback. | [Fixture-Orakel](../../tests/fixtures/offscreen/README.md), [2D-Bilder](../../tests/fixtures/offscreen/evidence/README.md) |
| 3D-Basisprofil | Perspektiv-Camera3d, StandardMaterial, DirectionalLight, tatsächliche Flächenbeleuchtung und Schatten. Licht-/Schattengegenproben, Pause, Pose-/Materialwechsel und typisierter Geräteverlust. Kein neuer Produkt-Capturepfad nötig. | [3D-Orakel](../../tests/fixtures/offscreen/src/scene3d.rs), [3D-Bilder und Fingerprints](../../tests/fixtures/offscreen/evidence/3d/README.md) |
| Historische lokale Teilabnahme (`5493eb1`) | Acht GPU-/SDKfälle in gemeinsamer Serie bestanden; finale vier 3D-Fälle nochmals bestanden. Damals 13 interne Owner- und 81 Harness-Tests sowie Workspace-/Fixture-Clippy bestanden. Historische Proben sind getrennt vom Produktnachweis. | [Expliziter Runner](../../tests/fixtures/offscreen/run.py), [historische Evidenz](../../tests/prototypes/headless-render-lab/evidence/README.md), [Testsuite](../../tests/README.md) |

**Zusätzlich in L1 lokal nachgewiesen:** wirkliche Signalstation 1440×900/Scale 1,
Image-EngineReady, vollständige zehn Original-Inputowner, Legacy-/Widget-UI,
sichtbare Unicodeglyphen, GPUoverlay, Pauseninvarianz und typisierter Geräteverlust.
13/13 L1-Fälle, 99/99 native lokale Einträge, 14 interne Owner-, 21 App-,
93 Harness- und erneut alle acht SDK-GPUfälle bestanden. Zwei Sol-Reviews
APPROVE; S1 erledigt, keine offenen Findings. [Nachweise und Bilder](../../tests/headless/evidence/README.md).
Die 99/99-native Basis entspricht dem funktionalen Stand `8e7de92`. S1 ergänzt
nur eine opt-in Real-Clock-Beobachtung für den Basisowner; alle 13 Headlessfälle
bestehen danach erneut, einschließlich direktem Real-Nachweis. Ein zusätzlicher
nativer Gesamtlauf scheiterte an fehlender Window-Surface; anschließend war der
Desktop nachweislich gesperrt. Dieser Versuch wird nicht grün gerechnet. Exakte
Sourceunterschiede und die Grenze dieser Nachprüfung stehen in [L1.md](L1.md).

**Qualifikationsgrenze der historischen Basisszenen:** Apple M1 Pro/Metal, macOS27.2, Bevy0.20.0/wgpu30.0.1,
65×49, nicht-pipelined; 3D mit asynchroner Pipelinekompilierung, festem HDR-/
Exposure-/Tonemappingprofil ohne temporale Effekte. Das ist weder ein Vollspiel-
noch ein allgemeiner PBR-/UI-/Asset-/Hardwarefehlertoleranznachweis.
Ein synchron kompilierender PBR-Lauf überschritt die 35-s-Pending-Beobachtung;
er bleibt Fehlbeleg, kein PASS und kein kontrollierter Kaltcache-Vergleich.

## Abgleich der bestehenden Arbeitspakete W00–W12

Die IDs stammen unverändert aus [R05, Abschnitt 11](PLAN.md#11-geordneter-integrationsfahrplan).
„Teilweise/offen“ bezeichnet den **übergreifenden R05-Headlessumfang**.
„Erledigt für L1“ schließt den benannten L1-Profilanteil ab, nicht ungeprüfte
Plattformen oder Folgepakete. Diese Bestandsmatrix ist keine Aufforderung,
jedes W-Paket nur teilweise anzufangen. Die Ausführung folgt den vollständigen
L-Lieferpaketen; derzeit ist keines aktiv, L2 ist unbegonnen.

| Paket | Status | Erledigter Beitrag | Noch offen / konkreter Abschluss |
| --- | --- | --- | --- |
| W00 Scope/Proofspec | Teilweise | Gepinnter Slice, konkrete Quellen/Orakel und dieser Statusabgleich. | Aktuelles Testinventar mit den Originalownern der Vollapp-Etappe verbinden; deren Vorbereitung, Tickbudgets und Erfolgskriterien vor Ausführung festlegen. Kein neuer grundsätzlicher Render-Machbarkeitsauftrag. |
| W01 Version/Config | Erledigt für L1 | Gemeinsame SDK-/Server-/CLI-/Appkonfiguration mit angeforderter/bestätigter Geometrie, explizitem Cargoflag und typisierter Ablehnung; Native-Ready unverändert. | Rust-Structliteral-Migration ist dokumentiert; R05-v2/v4-Vorschläge werden nicht nachträglich als notwendige Implementation ausgegeben. |
| W02 Renderziel | Erledigt für L1 | Appinstallation ohne native Features, reale Spielkamera/UI/Overlay auf Image, logische Inputgeometrie, 1440×900/Scale 1; HiDPI-Adapter CPUgeprüft. | weitere Größen/Scales/Plattformen bleiben eigene L5-Supportprofile, keine unfertige L1-Installation. |
| W03 Kontrollierter Runner | Teilweise | Originale Control-/Warpowner, InitTick1, Idle-Rendering, Uhrentrennung und L1-App-Startup-/Asset-/Resetpfad belegt; keine versteckten Schedules. | Übriger Lebensdauer-/Message-/Change-/Removed-Verbund in L3; andere SubApps/pipelined Profile nicht mitbehaupten. |
| W04 Ready | Erledigt für L1 | Image-EngineReady für tatsächliche Assets/Szene, GPUvorbereitung, Freeze, gezählte Ticks; originale Manual-/Engine- und vier Fehlerowner mit unveränderten Budgets bestanden. | Neue App-/SubAppprofile brauchen eigene Qualifikation, keine erneute Grundmachbarkeit. |
| W05 Capture | Teilweise | Produkt-RGB-Copy/Map-/Dateipfad, Korrelation und Publicationcheck; zusätzlich echte L1-App-/UI-/Glyphen-/Overlaybilder in realer Größe und Image-Ready. | Übriger Writer-/Mehrsession-/Lifecycleverbund gehört L3; keine beliebigen App-/Frame-/Effektprofile mitbehaupten. |
| W06 Input/UI | Erledigt für L1 | Alle zehn ursprünglichen Keyboard-/Pointer-/Textowner unverändert bestanden; Image-Picking, logische Keyidentität, Fokus/Unicode/Limits sowie Legacy-/Widget- und sichtbare Glyphenwirkung belegt. | keine übrigen Inputowner oder notwendigen Implementationen in L2 verschoben. |
| W07 Signalstation/Persistenz | Teilweise | Echte Signalstation mit Assets, Kamera, UI/Fonts, MSAA4/Tonemapping und Input jetzt im L1-Metalprofil betrieben; Systemclipboard ausgeschlossen. | Headless-Persistenz-/Inspect-/Recording-/Replay-/Report-/Clientverbund gehört zum noch unbegonnenen L2, nicht zur Freigabe durch den UI-Klick. |
| W08 Isolation/Abschluss | Teilweise | L1-Einzel-Session beendet sich normal sauber und bei eigenem Geräteverlust typisiert/reaped; aktive/deferred Requests und Datei-/Writerbarrieren CPUgeprüft. | Vollapp-Mehrsession, Startabbruch, konkurrierender Shutdown/Drop/Capture-/Writerverbund in L3. Keine physische GPUpartitionierung versprechen. |
| W09 Zoo | Offen | Vorhandener Imageexport ist wiederverwendbare Grundlage. | Hidden Window durch echten fensterlosen Export ersetzen; Ziel-/Assetbereitschaft und Captureack statt fester Framewartewerte; vollständiger Motiv-/Ansichts-/Label-/Shaderumfang. |
| W10 Tests/Entwicklereinstiege | Teilweise | Zehn Original-Inputowner plus drei Headlessowner; expliziter Runner mit Buildvariantenschutz, ursprünglichem Resetvertrag und Bildern. Neuer Viewport-CPUowner registriert; bisherige acht SDKfälle erneut bestanden. | Übrige Originalfallmatrix, direkte Entwickler-/Messwerkzeuge und weitere Testverbünde in L2/L3/L4; keine native Abnahme als Headless umbenennen. |
| W11 Plattform/Software | Teilweise | M1-Pro/Metal: kleine Basisszenen und tatsächlicher L1-Appumfang in 1440×900/Scale 1 nachgewiesen. | Weitere Adapter/Versionen/Größen, Linux/Vulkan und Software als eigene vollständige L5-Profile. Keine stillen Fallbacks oder Featureabschaltungen. |
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
| G01 Kompatibilität | Erledigt für L1 | Gemeinsamer angeforderter/bestätigter Launch-/Viewportvertrag, fehlende/falsche/unangeforderte Acks und Native-Defaults geprüft; Rust-Structliteral-Migration dokumentiert. Review abgeschlossen. |
| G02 Runner | Teilweise | Enger Tick-/Pause-/Hookpfad ist belegt. Vollapp-Lifecycle und tatsächlich benötigte Asset-/SubApp-Profile fehlen. |
| G03 Ready/Graph | Erledigt für L1 | Tatsächliche Image-/Appbereitschaft samt Freeze, Zählung und originalen Fehlerbudgets belegt; Review abgeschlossen. Weitere Profile bleiben eigene Abnahmen. |
| G04 Readback | Teilweise | Enger Produktpfad und Fehler-/Dateiregressionen sind belegt; vollständige App-/Ziel-/Lifecyclekombinationen fehlen. |
| G05 UI/Geometrie | Erledigt für L1 | Image-Picking, Fokus, zehn unveränderte Inputowner sowie wirkliche Legacy-/Widget-/Glyphenpixel belegt; Review abgeschlossen. GPUprofil ist Scale 1, HiDPI-Adapter zusätzlich CPUgeprüft. |
| G06 Adapter/Plattform | Teilweise | Metal-Basisszenen und reales L1-Appprofil belegt; übrige L5-Profile nicht qualifiziert, nicht widerlegt. |
| G07 GPUfeatures/Assets | Erledigt für L1 | Tatsächliche Spielassets, PBR/MSAA/Tonemapping, Fonts, UI/Glyphen und GPUoverlay im L1-Profil belegt; kein allgemeines Effekteversprechen. Review abgeschlossen. |
| G08 Native Dienste | Erledigt für L1 | Isolierter Appbuild ohne Winit/Arboard/X11, logische statt native Windowidentität und OS-Audit mit null eigenen nativen Fenstern; Native Default und Regression bleiben erhalten. |
| G09 Abschluss/Device-loss | Teilweise | Eigene logische Gerätefehler und Publicationbarrieren belegt. Vollständiger Mehrsession-/Start-/Writer-/Shutdownverbund fehlt. |
| G10 Zoo | Offen | Vorhandener Capturemodus erzeugt noch ein verborgenes natives Fenster. |
| G11 Abnahmevertrag | Teilweise | Lokale Slice-/Ownerprüfungen vorhanden; Originalfallmatrix und vollständige Headless-Ausführung nicht abgeschlossen. |
| G12 Externe Veröffentlichung | Separat | Echter Provider-/GitHublauf bleibt eigener Auftrag, kein lokaler Headless-/GPUblocker. |

Der Erfolgsvertrag bleibt endlich: korrektes bezeichnetes CPUbild und kein am
letzten Publicationcheck **beobachteter** Fatal. Keine atomare Kenntnis sämtlicher
zukünftiger/unbeobachteter Backendfehler und kein echter Treiberreset als neues
Pflichtgate. Einzel-Screenshot-Cancel wird nicht als neue Funktion vorausgesetzt;
`Pending`-Drop bleibt keine Cancellation.

## Konkrete Kopplungen: gelöst und noch offen

| Quelle | Heutiger Befund | Zuordnung |
| --- | --- | --- |
| [Launch](../../src/session/launch.rs), [Wire](../../src/session/protocol.rs) | `headless`/`no_default_features` gemeinsam durch SDK/CLI/Server/App; Acknowledgement und Native-Envreset geprüft. | L1 abgeschlossen |
| [Startup](../../src/session/startup.rs), [Render-Ticket](../../src/session/startup/render.rs) | Ticket akzeptiert Window oder Image und scoped GPUassets; gezählte Readiness und ursprüngliche Fehlerbudgets für die App belegt. | L1 abgeschlossen |
| [Viewport](../../src/session/headless.rs), [Pointer](../../src/session/input/pointer/cursor.rs), [UI](../../src/session/input/ui.rs) | Logische Window-Metadaten ohne nativen Handle; Image-Picking und kurzlebige Legacy-Focusprojektion. Originale Key-/Text-/Pointerverträge bestanden. | L1 abgeschlossen |
| [Appinstallation](../../test_app/src/app/headless.rs), [Appfeatures](../../test_app/Cargo.toml) | Expliziter isolierter Headlessbuild ohne Winit/Systemclipboard; native Defaults unverändert. Kameras/UI/Overlay zielen auf Image. | L1 abgeschlossen |
| [Zoo](../../test_zoo/src/app.rs) | `--zoo-capture` setzt weiterhin `visible:false` und installiert ein Window. | L4 offen |
| [Performanceeinstieg](../../examples/performance.py) | Appinternes GPUoverlay ist imagefähig; Script nutzt weiterhin `raise_window`, `osascript`, Displayabfragen und eigenen Dispatcher. | Direkter Headless-Entwicklereinstieg in L2 offen |

## L1: Umfang und Abschlusskriterien

**Lieferpaket L1: Signalstation-Basisbetrieb auf Metal. Zustand: abgeschlossen.**
Konkreter Umfang, Nachweise und akzeptierte Beleggrenzen: [L1.md](L1.md).

Der erste Integrationscheck bleibt: Signalstation headless starten → SessionReady
und kontrollierte Spiel-/Enginebereitschaft unterscheiden → tatsächlichen UIbutton
per Command betätigen → geänderten Spielzustand und reales PNG nachweisen → sauber
beenden. **Das ist nur ein Zwischencheck.** L1 wird erst mit dem vollständigen
vereinbarten Start-/Asset-/Input-/UI-/Text-/Captureumfang und dessen Fehler- und
Regressionsfällen abgeschlossen, nicht nach dem ersten funktionierenden Klick.

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
3. **Logisches Input-/UIziel vollständig (W06):** Keyboard mit Hold/Flanken/
   Release, absoluter/relativer Pointer, Buttons/Scroll, Text/Fokus und Image-
   Picking/UI mit einer gemeinsamen Geometrie verbinden. Unicode, Limits,
   ungültige Übergänge und sichtbare Glyphen gehören dazu. Wirkungen erst am
   expliziten Tick; Renderpump und Inspect/Capture dürfen keine Inputs verbrauchen.
4. **Original-Appfall anschließen (W07/W10):** den Owner
   [`tests.pointer.ui_click`](../../tests/pointer/ui_click.py) mit seiner wirklichen
   Vorbereitung und seinen unveränderten Zustands-/Tickassertions verwenden:
   „Gesamtkarte [G]“, `ViewState` Overview → Map, Move/Press/Release mit drei
   Aktionsticks. Bild-/Layoutbeleg und sauberer Abschluss kommen hinzu; der
   native Owner wird nicht durch eine reduzierte Headlesskopie ersetzt. Anschließend
   im selben Paket alle ursprünglichen Owner aus [Keyboard](../../tests/keyboard/),
   [Pointer](../../tests/pointer/) und [Text](../../tests/text/) vollständig anschließen:
   drei Keyboard-, fünf Pointer- und zwei Textfälle. Keine übrigen Inputowner
   als Restarbeit in L2 auslagern.

### Abschlusskriterien von L1 – alle erforderlich

- [x] Derselbe echte App-/Asset-/UIumfang startet im gewählten Metalprofil ohne
  natives Fenster, Display-/Fokus-/Clipboardaktion oder stillen Ersatzprozess.
- [x] SessionReady ist nicht fälschlich Spiel-/Enginebereitschaft; explizite
  Vorbereitung, Seed, Auflösung/Scale und sämtliche Ticks sind protokolliert.
- [x] Alle zehn ursprünglichen Keyboard-/Pointer-/Textowner bestehen zusätzlich
  im Headless-Appprofil, einschließlich ihrer Vorbereitung, ungültigen Übergänge,
  Fokus-/Unicode-/Limit-/Zustands- und Tickassertions; keine Abschwächung oder Skips.
- [x] Reales UIlayout, sichtbare Textglyphen und der UI-Klick sind im PNG und
  mit Inspect-Zustand belegt; Key/Pointer/Text entfalten vor dem angeforderten
  Tick keine Wirkung.
- [x] Capture/Inspect während Pause erhalten Uhren/Ticks; kein synthetisches Bild
  oder abgeschwächtes Originalorakel ersetzt den Produktnachweis.
- [x] Die für Start, Ready, Input/UI und Capture erforderlichen Fehler-/Fristfälle
  sind abgedeckt; Einzel-Session-Abschluss sammelt eigene Prozesse und
  berücksichtigt Writer/Dateiwirkungen. Native Defaults und betroffene native
  Regressionen bleiben erhalten.
- [x] Benutzbare Konfiguration, Anleitung und reproduzierbarer Nachweis beziehen
  sich auf denselben finalen Stand; relevante Reviewfindings sind aufgelöst und
  keine notwendige Implementation/Test-/Cleanup-Restarbeit von L1 bleibt offen.

**L1 ist abgeschlossen; L2 beginnt erst mit eigenem Auftrag.** Recording/Replay/Reports, weitere direkte
Entwicklereinstiege, Mehrsessionverbund, Zoo und weitere Profile behalten ihre
eigenen vollständigen Abschlusskriterien. Das sind zuvor abgegrenzte andere
Lieferpakete, kein Ablageort für unfertige L1-Arbeit.

## Pflege und nächste Aktualisierung

Während der Arbeit genau ein L-Paket als aktiv führen und seinen konkreten Rest
beziehungsweise Blocker festhalten. Nach dessen vollständigem Abschluss hier
Produktbaseline, erledigten Umfang, L-Zustand, W-/Gatezeilen und den nächsten
Schritt nachziehen. Erst dann ein neues L-Paket aktivieren. Neue Evidenz mit
eigenem Source-/Profilbezug verlinken; alte Fehlbelege und native Assertions erhalten. Eine
Quellprüfung ist kein Runtime-PASS. Keine Prozentzahl oder Komplettfreigabe aus
bloßer Existenz von Code ableiten.

L1 wurde unter dem expliziten `full-stop`-Auftrag implementiert und geprüft; die
neuen App-/GPUbelege sind oben verlinkt. Das ersetzt nicht die separate Freigabe
für CI, Installationen, externe Provider, Merge oder das Beginnen von L2–L5.
