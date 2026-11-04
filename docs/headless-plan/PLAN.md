# Woodpecker vollständig headless integrieren

**Technischer Plan R05 · deutsch · Basis 5d6ccd67321aacf69a3ee6ec9527fe7290af1ed4 · keine Implementierung.**

Veröffentlichungsfassung für den Arbeits-PR: Rechte-, CI- und Herkunftsangaben sind an den Folgeauftrag angepasst. Der technische Inhalt der Abschnitte 3–11 bleibt gegenüber R05 unverändert; zwei überzählige Markdown-Codezäune wurden entfernt. Der nachträglich ergänzte [Abschnitt 15](#15-ergänzung-gemeinsame-testausführung) integriert die Befunde zur gemeinsamen App-Nutzung; er gehört nicht zur historischen R05-Reviewzustimmung.

## 1. Entscheidung in Kürze

Empfohlen wird ein **explizit gewählter Offscreen-Renderpfad** mit echtem Bevy/wgpu-Renderer, Image-Renderziel, überprüfter Submission, asynchronem Readback und denselben Session-/Tick-/Input-/Recording-/Replay-Ownern wie heute. Ein tieferes Renderzielmodul übernimmt die tatsächlich variierende Fenster-/Image-Anbindung; es ersetzt nicht den Host und installiert nicht ungefragt einen Renderer.

Die erste vertikale Etappe ist eine frische kontrollierte Manual-Session: null Ticks → Capture lehnt ohne Dateiänderung ab → genau ein angeforderter Tick → echtes synthetisches 2D-Renderbild → Capture während Pause → Inspektion unveränderter Zeit/Ticks → sauberer Shutdown. Danach folgen EngineReady, 3D-/UI-/Textausgabe, echte UI-/Gameplayinputs, Signalstation, Recording/Replay, Mehrsession-/Fehlerfälle und Zoo. **Auch diese erste Etappe ist noch nicht ausgeführt.**

Der Plan ist an Proof- und Entscheidungsgates gebunden. Insbesondere Runner-/Change-Tracker-Semantik, Bevy-UI-Fokus auf Image-Targets, Frame-/Readbackzuordnung und Softwareadapterfähigkeit sind nicht als gelöst anzunehmen. Ein vollständiges Dokument ist kein bereits ausführbar validiertes Gesamtsystem.

Maßgeblich: [Coverage](COVERAGE.json), [Arbeitsumfang und Herkunft](README.md#arbeitsumfang), [Reviewstatus](README.md#reviewstatus) und [CI-Pause](../../.github/disabled-workflows/README.md). Coverage bleibt der unveränderte historische R05-Datensatz; seine ursprünglichen Rechte-/Evaluatorreferenzen sind keine aktuelle Ausführungserlaubnis. Alle nachfolgenden Typnamen und neuen Pfade sind **Vorschläge**, sofern nicht als Istquelle bezeichnet.

## 2. Auftrag, Rechte und feste Grenzen

Der Folgeauftrag erlaubt ausdrücklich, die CI im Quellcode zu deaktivieren und diesen Arbeits-PR mit Branch, Commits und Push anzulegen. Der einzige Workflow wird dafür inhaltlich unverändert nach `.github/disabled-workflows/` verschoben, bevor Remote-Schreibaktionen stattfinden. Repository-/Actions-Einstellungen und Branchschutz bleiben unberührt. Die Pause gilt für diesen Branch, auf `main` erst nach einem gesonderten Merge.

Produktquellen, Assets, Settings, Tests und ADRs bleiben in diesem PR unverändert. Der anschließende dokumentarische Folgeauftrag nimmt die bereits erstellte [Bestandsaufnahme zur Fensterwiederverwendung](../testing-window-groups.md) in die Teststrategie auf. Ihr Inhalt bleibt unverändert; die Kandidaten sind weder eine beschlossene Ausführungsreihenfolge noch bereits gemeinsam validierte Tests. Dieser Zusatz erlaubt noch keinen Runner- oder Produktumbau.

Keine CI-Ausführung, Workflowdispatches/Reruns, echten Provider-/GitHubtests, Tags oder Releases. Die PR-Freigabe ist insbesondere keine Freigabe für P20/G12. Keine lokalen Builds, Spiel-/GUI-/GPUläufe, VMs, Toolchaininstallation, Desktopaktionen oder Beendigung fremder Prozesse. Produktimplementierung und Runtime-Proofs benötigen weiterhin eine passende neue Freigabe; ein PR oder Testflag ist keine Erlaubnis.

Historische native Abnahme bleibt eigenständig. `--non-graphical` wird weder in headless umbenannt noch auf Exit 0 umgestellt. Der historische Surfacefehler bleibt Fehlnachweis unbekannter Ursache, kein Beleg einer bereits gefundenen Reparatur. Externe GitHubveröffentlichung gehört nicht zum Headless-/Grafiknachweis.

## 3. Vier unterschiedliche Betriebsarten

| Pfad | Native Fenster / Display | Echte Bildberechnung | Aussage und Empfehlung |
|---|---|---|---|
| Native, bisheriger Standard | ja | Hardware-GPU | Bestehende Abnahme und native Eigenschaften bleiben erhalten. |
| Offscreen-Hardware | nein; keine Winit-Instanz, Surface oder Displayverbindung | echter GPU-Adapter, Image, Submission, Readback | Primäres neues Headlessziel. macOS/Metal und Linux/Vulkan gesondert qualifizieren. |
| Offscreen-Software | nein | realer wgpu-Softwareadapter; kein Noop/Mock | Eigenes Fähigkeits-/Treiber-/Performanceprofil. Linux-CPU-Rendering als erforderliches Machbarkeitsgate, nicht behauptete universelle Verfügbarkeit. |
| Rendererfreie Logik | nein | nein | Bestehende Host-/World-/Dateiverträge; nie Ersatz für Shader, UI, Picking oder PNG-Bildinhalt. |

Xvfb, virtuelle Displays, versteckte oder vollständig verdeckte Fenster sind mögliche **andere** Betriebsumgebungen, nicht das definierte surfacelose Ziel. Sie dürfen ihre nativen Beweise liefern, aber nicht Offscreen-Gates schließen.

Plattformentscheidung: bestehende Unix-Prozessgruppen weiterverwenden (README, S01/S20). macOS-Hardwarepfad wegen bestehender Nutzung/CI zunächst priorisieren; Linux-Vulkan-Hardware und Linux-Softwareprofil danach als getrennte Qualifikation. Kein neuer Windows-/Browser-/Androidvollsupport. macOS ohne Metal-fähigen Adapter erhält zunächst klare Nichtverfügbarkeit, keine erfundene CPU-Fallbackfähigkeit. G06 entscheidet konkrete unterstützte Versionen/Adapter und Bereitstellung nach autorisierten Proofs. Kein neues Infrastrukturprojekt vorab beschließen.

## 4. Verifizierter Istfluss und Abhängigkeiten

```text
CLI / REPL / Script                         direktes Rust-SDK
        │ äußerer Vertrag v1                      │ Session::start
        ▼                                         │
loopback Server ── Create / ID / Activity ──────────┤
        │ eine Session pro unabhängiger Spielprozess
        ▼
Session-Koordinator ── Recording/Replay/History/Reports
        │ stdin/stdout JSONL v3; stderr Marker; Prozessgruppe
        ▼
Bevy session::Plugin / Control
        ├─ Commands prüfen / Inputs vormerken / Inspect / Capture poll
        ├─ expliziter Warp oder Engine-Startup
        │      └─ run_tick: Inputs flush → Clock → originale Main-Schedules
        │                      → Assetmessages sichern → ticks += 1
        └─ nach erstem abgeschlossenen Tick: Extraction → RenderApp
                          │ PrimaryWindow + RawHandle + Surface
                          ▼
                    Kamera/ViewTarget/Pipelines/GPU-Assets
                          ▼
                 Swapchainframe / GPU-Queueabschluss
                          ▼
         Bevy Screenshot → CapturedScreenshots → RGB-PNG → Root
                          ▼
               korreliertes Outcome → History/Recording/Activity
```

**S01–S25** in Abschnitt 13 und die ergänzenden S26–S51 in COVERAGE.json benennen echte Dateien/Symbole. Die vollständigen heutigen Acceptance-IDs stehen exakt einmal in COVERAGE.json, nicht in einer zweiten handgepflegten Zählliste.

### 4.1 Kopplungsregister

| Istpfad | Abhängigkeit / erhaltene Semantik | Geplanter Lösungspfad |
|---|---|---|
| F01 Launch/Config/Handshake | TOML v1, outer v1, game v3; `deny_unknown_fields`; Capabilities bestätigen Installation, nicht jedes Frame | Explizite neue Opt-in-Versionen, G01, P01/W01. |
| F02 Lifecycle/Transport | private Koordinatoren, unabhängige Pipe-Leser, Annahme vor Outcome, Requestbesitz, bounded History/Events/Activity, kein Retry | Owner unverändert, P10/P12/W07. |
| F03 Zeit/Renderpump | MainScheduleOrder nur Control; originale Schedules in run_tick; Startup einmal; Renderstart erst Tick 1; TimeReceiver drain ohne Zeitupdate | Kontrollierter Runner ohne Winit, G02, P02/W03. |
| F04 EngineReady | CPU-Assets/rekursive Abhängigkeiten/WorldInstance; stabile Assetmenge; Ticket.window; zwei vorbereitete Renderiterationen; Queuecompletion | Image-Targetticket mit mindestens gleich starken Beweisen, G03, P03/W04. |
| F05 Capture | ein primäres Window, RawHandle, Surfaceprüfung des **ersten Captureframes**, serielle Queue, Readbackfrist, sichere Destination | Offscreen-Copy-/Readback-Adapter und Zielgeneration, G04, P04/P05/W05. |
| F06 Pointer | PrimaryWindow, logische Pixel, Window-NormalizedRenderTarget, exklusive Obergrenzen; MoveBy volles MouseMotiondelta | echte Image-Location plus explizite logische Viewportgeometrie, G05, P06/W06. |
| F07 Keyboard/Text/Fokus | Windowentity, stabile Keytokens; Inputannahme vor Tick; Text an Entity gebunden, 16.384 UTF-8-Bytes | logische Fensteridentität ohne RawHandle; keine native Eingabe, G05/G08, P07/W06. |
| F08 UI/Layouts | heutiger Legacy-Adapter leiht Windowcursor; upstream `ui_focus_system` akzeptiert nur Window-Targets | Image-fähiger UI-Adapter statt gefälschter Surface, G05, P08/W06. |
| F09 Signalstation | DefaultPlugins/Winit, Window-Singles, CursorOptions, Camera, InputGate, Fonts, Settings; Throughput registriert StandardMaterial/Gizmos | App-seitige Modusinstallation, Zielzuordnung und Cursoranpassung; G07/G08, P09/W07. |
| F10 Recording/Replay | geordnetes JSONL v1/v2; effektive Clock; Dateibarrieren, Vorvalidierung, Stop/Blocked, kein Worldsnapshot/Outputvergleich | Formate erhalten; Headlessfixture + eigene Zustands-/Bildorakel, P10/W07. |
| F11 Reports | vertraulicher Session-Snapshot, lokale Sandbox, Hintergrundprovider, abschließender Join | Local/Fake-gh getrennt; keine Liveveröffentlichung; P11/W07. |
| F12 Stop/Shutdown | Clientende stoppt nichts; reservierte Shutdown-ID; gleiche Gesamtfrist; eigene Prozessgruppen; Drop hart | Render-/Readbackressourcen drain/cancel integrieren, G09, P12/W08. |
| F13 Mehrsession/Dateien | unabhängige Prozesse/Workspaces/Settings/TMP, keine globale chdir/Adapterumgebung/Ports | pro Prozess Rendergerät/-ziel/Generationen; Sandbox erhalten, P12/W08. |
| F14 Durchsatz/Assets | pro Tick alternde Appmessages und separater Renderstrom; circa 33-ms-weiches Budget; FrozenShadows bei Idle-Extraction | S14/S15 erhalten und Offscreen nachweisen, G02/G07, P13/W07. |
| F15 Zoo | Image/COPY_SRC und Screenshot::image vorhanden, **trotzdem hidden Window/Winit**, Update-Framestände 80/12 als Verzögerung | echter eigener surfaceloser Export; Ready-/Captureack statt Framezählen, G10, P14/W09. |
| F16 Runner/Produkte | Auswahl/Featurematrix, Manifest-/Asset-/Binaryhashes, shared input fixture, PTY, Cleanup, keine Retries | neuer Headlessumfang zusätzlich; Originalfälle nicht umklassifizieren, G11, P15/W10. |
| F17 Native Restnachweise | Surfaceverfügbarkeit, Fokus-/Cursor-/IME-/Clipboard-/Present-/Display-/DPI-/Occlusion-Verhalten | native Spur bleibt eigenständig; P16/W10. |
| F18 Software/Plattform | wgpu30.0.1/Naga30.0.1, Bevy0.20.0; tatsächliche Features/Limits/Adapter nötig | Capabilityprobe vor Szene, G06/G07, P17/W11. |
| F19 Beispiele/Performance | eigene Pipe-Owner, explizite Tickbudgets, kein CLI-Overheadvergleich mit direkter Messung | alternative Konfigurationen und getrennte Baselines, P18/W10. |
| F20 Externe Integration | echter GitHubfall separat genehmigungspflichtig | G12, kein Headlessblocker und hier streng verboten. |

## 5. Zielarchitektur: kleine Interfaces, echte Adapter

### 5.1 Moduswahl und Anwendungseigentum

Native bleibt Default. Kein `DISPLAY`-Heuristikmodus, keine unsichtbare Ersatzsession und kein automatischer Hardware→Software→rendererloser Fallback. Vorschlag für neue Konfiguration: `render.kind = native | offscreen`, bei Offscreen positive physische Breite/Höhe, endlicher positiver scale_factor und `adapter = hardware | software`. Adaptername/Backend nur als enge Auswahl, nicht als beliebige Environmentweitergabe. Nicht passende Auswahl scheitert sichtbar. **Konkreter Auswahlvertrag:** Der Launcher entfernt beziehungsweise setzt nur für seinen eigenen Kindprozess vor Rendererinitialisierung die gepinnten Auswahl-Overrides `WGPU_ADAPTER_NAME`, `WGPU_FORCE_FALLBACK_ADAPTER`, Backend-/Powerpräferenz und `WGPU_SETTINGS_PRIO` kontrolliert; keine globale Environmentmutation. Weitere backend-/treiberspezifische Variablen werden profilweise inventarisiert, erlaubte Software-ICD-Auswahl explizit gepinnt. Die appseitige Renderinstallation wählt einen zum vollständigen Selektor passenden Adapter und bindet die tatsächlichen Adapterdaten daran, bevor sie Ressourcen via Bevy `RenderCreation::Manual` übernimmt. Mehrdeutige/fehlende exakte Namens-/Vendor-/Deviceauswahl scheitert; ist kein Name/ID gefordert, ist jeder Adapter innerhalb der ausdrücklich gewählten Klasse/Backendmenge zulässig und wird als effektive Wahl protokolliert. Den Bevy-Automatic-Namensfallback nicht verwenden: `initialize_renderer` liest Overrides erneut und fällt bei erfolgloser Namenssuche auf request_adapter zurück. P19 fordert unbekannten Namen, widersprüchliche Overrides und einen falschen Adapter derselben Klasse als Negativfälle. G07.admission muss den genauen öffentlichen Manual-Ressourcen-/Fehlervertrag belegen, keine erfundene Adapter-API voraussetzen.

Die Anwendung muss ihren Renderer, Asset-/UI-/3D-Plugins und alle betroffenen Kameras selbst installieren. Woodpecker konfiguriert den kontrollierten Ablauf, nicht die Spiellogik. Neue appseitige Offscreen-Installation **vor** dem Bauen von DefaultPlugins/Window-/Winit-Plugins; bloßes spätes Entfernen eines existierenden Fensters ist unzureichend.

### 5.2 Vorgeschlagene Module und Interfaces

| Modul / vorgeschlagener Pfad | Kleines Interface und Verantwortung | Nicht enthalten |
|---|---|---|
| Renderziel, `src/session/rendering/target.rs` (neu) | `TargetDescriptor` → validiertes `TargetLease`; Snapshot aus Ziel-ID, Generation, physischer Größe, Scale, Format, Kamerabindung. WindowTarget- und OffscreenImage-Adapter. | kein Session-/Report-/Game-Kontext; keine Rendererinstallation |
| Framebeweis, `rendering/proof.rs` (neu), verwendet von `startup/render.rs` | `request_proof(target, extracted_tick, purpose)` → einmaliges `FrameReceipt` oder Diagnose; GPU-/Pipeline-/Asset-/Submissionwissen intern | kein Weltreset, kein eigener Tick |
| Renderpump, `rendering/pump.rs` (neu) | klarer Kontrollzyklus `poll_control → ggf. run_tick → extract/render → poll_gpu/completions`; getrennte Tick-/Frame-/Wallclock; Owner der App-Lebensdauer | kein Controllerpolling als Fortschrittsvoraussetzung |
| Offscreenreadback, `screenshot/readback.rs` (neu) | `enqueue(capture_token, target_lease)` → CPU-Pixel plus Receipt oder Fehler; map/unmap und Bufferlebensdauer intern | keine Pfadvalidierung, PNG oder Geheimnislogs |
| Capture-Owner, bestehendes `screenshot/capture.rs` | serielle Requests, Deadline vor Encoding, Zuordnung, Workerabschluss; verwendet Windowcapture oder Offscreenreadback | kein globales Ready-Flag statt Framebeweis |
| Eingabegeometrie, bestehendes `session/window.rs` weiterentwickeln | native/logische Fensterentity und Renderziel **getrennt** auflösen; explizite Größen/Scale; keine RawHandlepflicht für virtuelle Inputs | kein zweiter Inputzustand neben Bridge |
| UI-Adapter, bestehendes `session/input/ui.rs` | Image-fähige Cursor-/Kameraauflösung für Legacy-Interaction und aktuelle Picking-/Widgetpfade | keine OS-Cursorbewegung; kein dauerhafter Kamera-Targetswap |
| Anwendungsadapter, `test_app/src/woodpecker.rs`, `app/mod.rs`, Zoo `app.rs` | Modus konfigurieren, native Plugins ausschließen, Kameras/Viewport/UI/Clipboardpfad anbinden; Spiel-Owner bleiben bestehen | kein dupliziertes Level oder vereinfachtes Ersatzspiel |

Ein `TargetLease` ist kein God-Context: nur unveränderliche Ziel-/Geometriedaten und Lebensdauertoken. Framebeweis und Readback kennen keine Server-/Dateipfade. Zwei tatsächlich unterschiedliche Adapter rechtfertigen die Seam; reine Durchreichschichten vermeiden.

### 5.3 Zielidentität, Kamera und UI

Offscreenziel: `Image::new_target_texture`, zunächst `Rgba8UnormSrgb`, `RENDER_ATTACHMENT | TEXTURE_BINDING | COPY_SRC`, Single-Sample-Endbild; MSAA/HDR bleiben in der originalen Kamerapipeline und müssen in das endgültige LDR-Ziel aufgelöst werden. Kein Lesen einer Zwischen- oder Prepass-Textur als vermeintliches Endbild.

Eine logische `Window + PrimaryWindow`-Entity ohne RawHandle kann für Keyboard-/FocusedInput-/App-Singles nützlich sein, ist aber **keine native Fensterinstanz und keine Renderoberfläche**. Die echte Kamera und PointerLocation zeigen auf `NormalizedRenderTarget::Image`. WindowPlugin erzeugt sie im neuen Modus nicht automatisch: `primary_window: None`, `ExitCondition::DontExit`, WinitPlugin ausgeschaltet; logische Entity ausdrücklich vom Adapter verwalten. G05 muss belegen, dass keine andere Installation daraus ein natives Fenster/Display anfordert.

Die Descriptorgröße ist physisch. Logische Größe = physisch / scale_factor; Pointerursprung oben links, Grenzen exklusiv. Kamera-`ImageRenderTarget` besitzt im gepinnten Bevy tatsächlich `scale_factor`; denselben Wert für Target-/Pointervergleich verwenden. `Screenshot::image(handle)` normalisiert auf Scale 1: deshalb nicht blind für Scale ≠1 verwenden. Viewportoffset, Scissor, Kameraorder, aktive Kameras und UI-Zielkamera müssen auf dieselbe Koordinatentransformation verweisen. Phase 1 fixiert 1440×900 und Scale 1; weitere fix gewählte Scales und Odd-Width-Proofs sind Erweiterungsgates, keine dynamischen Resizeversprechen.

Kein stilles Retargeting beliebiger fremder Kameras. Der App-Adapter bindet die dokumentierte primäre Spielausgabe samt UI/Overlay; mehrere Kameras auf demselben Ziel bleiben mit Originalorder/Clears erhalten. Zusätzliche Render-to-Texture-Kameras werden nicht pauschal umgebogen. Mehrere unabhängige Hauptziele bleiben vorerst nicht unterstützt und werden explizit abgelehnt.

Upstream `ui_focus_system` ignoriert Image-Targets. Das heutige Leihen des Windowcursors allein kann Headless-UI nicht reparieren. W06 muss den bestehenden Legacy-Vertrag mit einem Image-fähigen Adapter erhalten und aktuelle `ui_picking`/Widgets/TextInput ebenfalls testen. Migration auf neue Widgets wäre ein eigener, nicht stillschweigend erzwungener Produktumbau. Kein pro Frame temporärer Targetswap als vermeintlich risikofreie Lösung. G05 ist ein **kritischer Freigabeblocker**, bis Auswahl und Proof vorliegen.

## 6. Zeit, Startup und echter Framebeweis

### 6.1 Drei Uhren und erlaubte Arbeit

- **Simulation:** nur run_tick, expliziter Warp oder konfigurierter, gezählter Engine-Startup. Fixedclock vor Tick 1; Applicationclock einschließlich Pause/Geschwindigkeit bleibt ihr Vertrag. Ein Tick ist kein einzelner FixedUpdate.
- **Renderiteration:** Extraction, RenderApp, Pipeline-/GPUfortschritt und Readback können nach Tick 1 auch bei Pause weiterlaufen. Renderzähler wird nicht als Anwendungstick/Replaycommand gespeichert.
- **Wallclock:** Pace, Arbeitsbudgets, Startup-/Readback-/Shutdownfristen; keine garantierte Antwortlatenz und kein ungemessener FPS-/TPS-Zielwert.

**Nicht einfach `ScheduleRunnerPlugin::run_loop` übernehmen.** Das Bevy-Beispiel führt normale `App.update()` aus; `SubApps::update` löscht Mainworld-Change-/Removed-Tracker am Ende. Die vorhandene MainScheduleOrder-Gate verhindert Gameplay-Schedules, beweist aber nicht automatisch Nebenwirkungsfreiheit für Message-Lifecycle, Changes/Removals oder fremde SubApps. G02 muss einen begrenzten kontrollierten Runner mit denselben expliziten Spielticks und einer inventarisierten SubApp-Policy nachweisen. Empfehlung: nur freigegebene Control-/Renderarbeit im Idle, Mainworld-Tracker/Lifecycle nicht pro zusätzlichem Renderpump altern; genau festgelegtes Clearing nach den tatsächlich beobachtenden Tick-/Extractionphasen. Ob das mit öffentlichen Bevy-0.20-Interfaces hinreichend möglich ist, ist vor W03-Abnahme zu belegen; andernfalls Entscheidung über eng begrenzte Upstreamintegration, kein verstecktes Bevy-Forkprojekt.

Startup/Plugin `ready/finish/cleanup`, Taskpoolfortschritt und AppExit müssen auch ohne Winit korrekt laufen. Erst nach Initialisierung/Handshake Zeitbudget starten wie heute. Keine Winit-/Eventloop-Hauptthreadvoraussetzung still übergehen. Anwendungsseitige weitere SubApps mit Spielwirkung dürfen nicht in jeder Idleiteration laufen; G02 inventarisiert und blockiert unbekannte, statt sie heimlich auszuschalten. Single-thread Renderpump kann erste interne Proofvariante sein; pipelined Rendering darf nur nach eigenem Proof aktiviert werden und muss als effektive Laufkonfiguration sichtbar bleiben. Kein Messvergleich mit der bisherigen Pipeline ohne neue Baseline.

### 6.2 EngineReady-Protokoll

```text
Handshake/SessionReady
  → CPU-Assets + rekursive Dependencies + Worldinstances
  → gleiche bekannte Assetmenge in zwei gewöhnlichen gezählten Ticks
  → Freeze {tick, target_id, generation, scope}
  → Extraction des Freezezustands
  → Ziel/ViewTarget + GPU-Mesh/Image + Pipelinezustände prüfen
  → zwei aufeinanderfolgende tatsächlich vorbereitete Renderiterationen
  → finale Rendergrapharbeit an genau dieses Ziel submitted
  → Queuecompletion der zugehörigen Submission
  → EngineReady {ticks}; keine weiteren Startup-Ticks
```

Die heutige Prüfung aller CPU-seitig bekannten Render-World-Meshes/Images und aller gecachten Pipelines nicht einfach auf ein leeres Offscreenziel reduzieren. Custommaterial-/Font-/Shaderbuffer-/Gizmo-Bereitschaft zusätzlich an vorhandenen Appscope/Renderpfad binden (G07); Pipelinefehler bleiben terminaler Startupfehler. Globale Pipelinecacheprüfung kann auch irrelevante Fehler erfassen; eine zukünftige Präzisierung braucht eigenen Vertrag, sie wird hier nicht als notwendiger Headless-Skip benutzt.

`FrameReceipt` vorgeschlagen: Prozess-/Sessionlebensdauertoken (intern), monotone Zielgeneration und Renderepoche, extrahierter abgeschlossener Tick, Kamera-/Zielidentität, geprüfte Asset-/Pipelinegeneration, Submissionindex und einmaliger Ergebniszustand. Queuecallback nach `RenderSystems::Render` allein beweist noch nicht, dass der gemeinte Graphbranch tatsächlich dieses Ziel geschrieben hat. G03 verlangt zielbezogenen Rendergraph-/Submissionbeleg; bei leerem Graph darf kein Receipt entstehen. Zwei Iterationen sind eine Mindestinvariante des heutigen Schutzes, keine universelle Garantie für jede neue Custompipeline.

#### Fehlerentscheid vor Erfolgsfreigabe — kritisches G03/G04/G09

SOURCE_ADDENDUM.json pinnt ergänzende Primärquellen: Bevy `RenderResources::unpack_into` installiert auch bei Manual-Ressourcen den privaten `DeviceErrorHandler`, der Device-loss-/uncaptured-error-Callbacks übernimmt. `error_handler::update_state` verarbeitet sie beim Extraction-Einstieg; der öffentliche `RenderErrorHandler` erzeugt standardmäßig AppExit. wgpu-core `Queue::on_submitted_work_done` garantiert den Callback auch bei DeviceError und liefert keinen Erfolgsstatus. Ein Queuecallback darf deshalb **niemals unmittelbar ein erfolgreiches Receipt setzen**.

Owner ist das Framebeweis-/Pumpmodul mit pro logischem Gerät serialisierter Fehler-/Receipt-Arbitration. Den öffentlichen `RenderErrorHandler` nach Bevys Ressourceninstallation kontrolliert setzen: ersten fatalen Fehler latchen, Device-/Target-/Capturegenerationen invalidieren, StopRendering wählen, niemals Ignore/Recover. Keine zweiten konkurrierenden Devicecallbacks über die privaten Bevycallbacks legen. Deterministischer Fehlervorrang: DeviceLost vor OOM/Validation/Internal, primäre Ursache erhalten.

Submission/Copy/Map/Queuecallbacks erzeugen nur **Kandidaten**. Am benannten Publisher-Cut vor Erfolg müssen ein belegter begrenzter Geräte-/Fehlerbeobachtungsdurchlauf und die zum Kandidaten gehörenden scoped Draw-/Submit-/Mapfehler ausgewertet sein. Bereits im Backend erfasste fatale Fehler müssen vorher in der Arbitration sichtbar werden. Derselbe serialisierte Owner entscheidet einmalig: Fatal → Terminaldiagnose ohne Ready-/Captureerfolg; andernfalls gültiger ziel-/framegebundener Kandidat → Erfolg. Ein erst nach gültiger Veröffentlichung eintretender Gerätefehler beendet die Session separat, ohne frühere gültige Outcomes rückwirkend umzudeuten. Sleep/Ruheintervall beweist keine Fehlerfreiheit.

**Ungelöste kritische Voraussetzung:** `DeviceErrorHandler` und `update_state` sind private Bevyinterfaces. Der bestehende Bootstrap unterdrückt ursprüngliche Extraction vor Tick1 und damit potentiell Fehlerbeobachtung. Der öffentliche Handler allein beweist weder pre-Tick1-Fortschritt noch rechtzeitige Beobachtung bereits gecatchter Fehler. G02/G03/G04/G09 verlangen einen konkreten öffentlichen, nebenwirkungsfreien Poll-/Scoped-error-/Extractionhook und dessen Ordnungsproof. Fehlt er im gepinnten Bevy, ist eine minimale ausdrücklich freizugebende Upstream-/Integrationsänderung zu entscheiden. **Bis dahin keine EngineReady-/Readback-/Headlessgesamtfreigabe**; kein heimliches App.update/Tick und kein behaupteter Zugriff auf private Funktionen.

Der geplante v4-Codec benötigt einen expliziten terminalen Renderdiagnosepfad (Vorschlag `render_failed` mit stabilem Code/Deviceepoch; exaktes Schema und Public-Evententscheidung G01). Koordinator/Server ordnen ihn derselben Session zu und markieren sie Failed; v3 bleibt unverändert. Vor Handshake bleibt Fatal Launchdiagnostik, danach auch vor Tick1 terminale Renderdiagnostik. Kein Ready nach Fatal; offene nicht schreibende Captures invalidieren, keine Erfolge aus späten Kandidaten. Laufende Writer dürfen unter ursprünglicher Stop-/Gesamtfrist auslaufen: Dateiwirkung kann bestehen, aber nach dem Fatal-Cut kein neuer Captureerfolg veröffentlicht werden. Bei Deadline eigenes Prozessende und unvollständige Artefakte, kein unbegrenzter GPUwait. Absichtlich ausgelöstes Prozessende erzeugt keinen zusätzlichen erfundenen ProcessExit-Report; echte bereits beobachtete Tracingfehler behalten ihren Reportpfad.

Die Ausführungsowner sind ausdrücklich getrennt: **P03/W04** prüft pre-Tick1, Ready, finalen Draw/Submit, Queuecallback trotz Fehler und bereits gecatchte, noch nicht zum Mainworldhandler gelieferte Fehler — ohne Capture/Map/PNGwriter vorauszusetzen. **P05/P21 in W05** ergänzt Copy-/Mapfehler, den Capture-Fatal-Cut und laufende PNGwriter. **P12/W08** prüft vollständige Mehrsession-/Shutdownrennen. Sämtliche Interleavings bleiben Pflicht, aber kein früher Paketabschluss setzt die spätere Captureimplementation voraus. Genau ein eindeutiger Terminalausgang, keine Ready-/Capture-Erfolgsquittung für ungültige Kandidaten. Diese Spezifikation ist **kein Nachweis einer verfügbaren Hooklösung**.

Fehler: fehlender Renderer/Scope, fehlendes Ziel/Kamera, Assets/Pipelines fehlgeschlagen, Adapter-/Featuremangel, Device-loss, Budgetgrenze. Bestehende Startupcodes im nativen Pfad erhalten; neue Opt-in-Fehler in G01 exakt festlegen. Ticklimit wie heute: ein bereits eingefrorenes Frameticket darf noch innerhalb der realen Startupfrist Renderfortschritt machen, ohne Zusatzspielticks. Kein Sleep, feste Framezahl oder „nicht schwarz“ als Readybeweis. Kein neuer Warm-up-Command hinter dem Rücken des Aufrufers.

Cancel bei Start-/Serverstop invalidiert Ticketgeneration **vor** Cleanup. Späte Queuecallbacks können nur ihr altes Ticket beenden, niemals neue Bereitschaft melden. Device-loss beendet Renderfähigkeit sichtbar, kein Geräte-/Sessionneustart. Der Server beendet die betroffene Session nach bestehenden Terminalregeln. Andere Sessions laufen weiter, **solange deren eigenes logisches Gerät gesund bleibt**. Prozessgruppen und getrennte logische Geräte sind keine physische GPU-/Treiberpartitionierung: ein gemeinsamer Adapter-/Treiberfehler kann mehrere Sessions treffen; jede meldet dann ihren eigenen typisierten Terminalfehler. Garantiert werden korrekte Zuordnung und keine fremden Prozesssignale, nicht Hardwarefehlertoleranz. G09 bestimmt geordnetes Drain versus erzwungenes Deadlineende.

## 7. Screenshot, Readback und Dateien

Nativer Adapter behält seine Window-/RawHandle-/Surfaceprüfung. Der Offscreenadapter ersetzt sie durch **gleich starken ziel- und framebezogenen Outputbeweis**, nicht durch `surface=true`.

1. Syntax und Destination am bestehenden Owner validieren. Vor Tick 1 kein Queueing/Schreiben/Tick; im neuen Vertrag expliziter Ziel-nicht-bereit-Code. Installation/Capabilities allein reicht nicht.
2. Ein Command bindet bereits bei Annahme Targetidentität und Generation in seinem Queuejob, wie heute `Job.window`. Vor Aktivierung müssen beide noch exakt übereinstimmen; ein Zielwechsel während der Wartezeit endet ohne Capture/Dateiänderung mit Ziel-nicht-verfügbar. Die Aktivierung pinnt zusätzlich TargetLease und Mindest-Extraktionsepoche nach der Aktivierung; ein älteres Bild ist kein Erfolg. Anstehende Ticks/Warps dürfen danach normal weiterlaufen; Capture ist **kein** impliziter Tickzustands-Barriercommand. Receipt benennt den tatsächlich extrahierten Tick, nicht eine erfundene atomare Commandmomentaufnahme.
3. Nach allen ursprünglichen Kamera-/UI-/Postprocesspässen finalen Output in Stagingbuffer kopieren. Zielbezogener Graphbeleg und Submission-/Copyticket gehören zusammen. Nicht bloß im Render-Prepare eine vorhandene GpuImage-Allocation als ausgefüllten Frame ausgeben.
4. `map_async`-Ergebnis typisiert auswerten; Renderer/GPU pollt begrenzt weiter, ohne Time/Input/Spiel-Schedules. CPUbytes erst nach erfolgreichem Copy/Map; Fehler/Timeout/Cancel invalidieren Token. Jede Lieferung trägt Capturetoken, Zielgeneration, Frame-/Submissionkennung, Format/Extent/Stride. Kein unbeschrifteter globaler Receiver für mehrere aktive Lebensdauern.
5. Länge und Überläufe prüfen: Zeilenbytes = width × bytes_per_pixel; Pufferstride auf wgpu-`COPY_BYTES_PER_ROW_ALIGNMENT` (gepinnt 256) aufrunden, Padding je Zeile entfernen, exakte Nutzlänge width × height × bytes_per_pixel verlangen. Odd widths 1/63/65/1441 prüfen. BGRA→RGBA/RGB, sRGB/Linear, Roworientation und Alphaentscheidung explizit testen, nicht heuristisch raten.
6. Bestehender Produktoutput ist RGB-PNG (`to_rgb8`), also keine neue Alphaerhaltung behaupten. Für Offscreen zunächst gleicher LDR/sRGB/RGB-Vertrag; transparenter interner Clear muss korrekt komponiert werden. HDR-/MSAA-Zwischenformate erst nach finalem Resolve/Tonemap erfassen. Neue Roh-/HDR-/RGBAexporte wären separate Version/Scopeentscheidung, keine Pflichtfunktion dieses Plans.
7. Vor Übernahme zum Encoding dieselbe 30-s-Readbackfrist ab Aktivierung prüfen; Queuezeit zählt wie heute nicht. Auch ein bereits im Kanal wartendes verspätetes Bild darf nicht mehr schreiben. Die Frist umfasst weiterhin nicht das bereits gestartete PNGencoding/Schreiben und garantiert keine harte Zustellzeit. CPUworker bindet Ergebnis an eigenen Requesttoken.
8. Ergebnis erst nach erfolgreichem sandboxed PNG-Schreiben: relativer Pfad, tatsächliche physische Maße, `overwritten`. Bestehendes **cap-std-Confinement** und Tempdatei-/sync-/Renameverfahren in `destination.rs::prepare/write` weiterverwenden; beim Schreiben erneut relativ zur geöffneten Rootcapability auflösen. Anders als Recording-/Reportpfade besitzt Screenshot hier **kein pauschales NoFollow-/Symlinkverbot**. Rootinterne Symlinks bleiben zulässig; Rename ersetzt den Link selbst, nicht dessen Referenzdatei. Der positive interne Link-/Replacefall, Linkziel unverändert, sowie Escape-/Pfadaustauschtests und Outside-Sentinel bleiben. Keine freie Zielpathoption im Renderthread.

Bevy `Screenshot::image` zeigt einen vorhandenen echten Renderpfad. Es ist **kein vollständiger Fehler-/Readbackbeweis**: gepinnte Implementierung kann in map_async bei Fehler panicen und intern ein präpariertes statt belegt gefülltes Bufferresultat weiterreichen. Empfehlung für den neuen Adapter: eigener enger finaler Copy-/Map-Owner mit Receipt; vorhandenen Windowcapture nicht opportunistisch umschreiben. G04 muss den präzisen Graphhook, Submitordnung und Fehlertransport am tatsächlichen Bevy0.20-Code belegen. Kein kopierter Fullrenderer und kein `save_to_disk` an der Dateisandbox vorbei.

Cancelumfang: Screenshot ist heute nicht einzeln stoppbar. Replaystop lässt bereits versandte nichtstoppbare Captures abschließen; nicht plötzlich Taildateien eines neuen Requests zulassen. Verwaltungsstop drainiert offene Captures/Writer unter der vorhandenen Gesamtfrist; bei Ablauf invalidieren, eigene Prozessgruppe beenden, unvollständigen Abschluss melden. Encoding kann bereits Dateiwirkung haben; keine atomare Rücknahme versprechen. Ungewisse Dateiwirkung nicht als „garantiert nichts geschrieben“ umdeuten. P05/P12 testen späte Antworten, Doppellieferung, Entity-/Generationwechsel und Shutdownrennen.

## 8. Eingaben, Spiel und Ressourcen

### 8.1 Virtuelle Geräte

Bridge bleibt einziger Owner akzeptierter/gehaltener/pending Inputs. Keytokens/Flanken/Press-Release-Fehler und logische/physische Keyabbildung bleiben erhalten. Inputs werden bei Erfolg nur vorgemerkt; Renderpump/Inspect/PNG verbrauchen sie nicht. MoveBy sendet weiterhin das volle endliche MouseMotiondelta unabhängig vom Cursorclamp; MoveTo erzeugt kein relatives Delta. Scroll in Zeileneinheiten, erlaubtes [0,0], Buttons unabhängig, sessionlokaler Pointer mit derselben Custom-ID pro World.

Keyboard-/Textmessages verwenden die logische Fensterentity, Picking nutzt das echte Imageziel. Textfokus ist Worldlokal, nicht OS-Fokus. Akzeptierter Text bleibt an die ursprüngliche Entity gebunden und wird bei Despawn/Editabilityverlust am Tick verworfen, nicht neu fokussiert. Emptytext, 16.384-Bytegrenze und Widget-/Unicode-/Filter-/Selektionsregeln erhalten. Bevy `TextInputPlugin`/`ui_widgets` bleiben Anwendungsvoraussetzung.

Native Maus/Touch/Key/IME werden auch im neuen Modus nicht in kontrollierte Ticks zugelassen; die vorhandenen Gate-/Owner-Tests bleiben. Headless kann keinen echten OS-Fokusverlust, IME-Kompositionsdialog, Cursorgrab oder Desktopclipboard beweisen. Virtuelles Textcommit ist kein IMEtest. Keine OS-Eingaben, Focusrequests oder Cursorbewegung zur Testreparatur.

### 8.2 Signalstation

`test_app/src/app/mod.rs` und `app/launch.rs`: opt-in Modus früh lesen, native Window/Winit/Presentoptionen nur im nativen Pfad; Offscreen ohne native Pluginausführung. `woodpecker.rs` setzt WinitSettings nur noch im nativen Adapter, behält Startupassets, Reflection, InputMode::Controlled, SimulationTicks, Diagnostics und Throughput. Keine vereinfachte Ersatzszene anstelle der echten glTF-/PBR-/UI-Anwendung.

`input/devices.rs::Cursor::position` vergleicht heute Window-Targets; auf Descriptor-/Imageziel auflösen. `input/policy.rs::InputGate` und Camera-/CursorOptions-Singles an die logische Geometrie anbinden; Headless niemals echten Grab anfordern. `player/camera.rs` erhält Originalperspektive/Transforms/Hierarchie und Projektion, UI-/Settings-/Textfokuspfade bleiben fachlich identisch. `ui::pointer_over_ui`, Viewport-to-world, Meshpicking und Kollision/Navigation müssen denselben logischen→physischen Transform verwenden. Fonts Barlow/NotoCJK/NotoEmoji und originale Assets samt Lizenzen/Hashes verwenden, keine Systemfontannahme.

**Clipboardrisiko:** test_app aktiviert heute `system_clipboard`; Bevy Clipboard::default kann bei Unix/macOS den Systemprovider initialisieren. Winit aus reicht nicht. Headlessbuild muss diesen Featurepfad ausschließen oder einen belegten explizit isolierten Clipboardadapter verwenden; gepinnte Clipboardfelder sind privat, eine erfundene `memory()`-Methode ist keine Lösung. Cargo-Featureunion im Workspace kann `--no-default-features` wirkungslos machen, wenn ein anderes selektiertes Package system_clipboard mitbringt. G08 fordert geprüften getrennten Featuregraph und Nachweis null Systemclipboardzugriffe. Native Bedienung behält ihre bisherige Clipboardfähigkeit. Keine persönliche Clipboard-/Settingsbeobachtung in dieser Planungsphase.

Signalstations Offscreenstart ist zunächst **nur mit kontrollierter Session** erlaubt. Ohne gültigen absoluten eigenen Workspace und expliziten Testseed wird er vor Plugin-/Settingsinitialisierung abgelehnt; kein Rückfall auf `project/settings.json`. Der Sessionbootstrap `workspace::prepare` bleibt zuständig. G08/P09 prüfen fehlenden/ungültigen Workspace und Seed sowie unveränderte gemeinsame Settings. Ein späterer standalone Spielstart bräuchte einen gesonderten Bootstrapvertrag und ist nicht Teil der zugesicherten ersten Integration. Settings bleiben unter `workspace/config/settings.json`, Seed ausdrücklich aus Tests; TMP/TEMP/TMPDIR unter eigenem Workspace. Kein Zurückfallen auf geteilte Schreibpfade. Assets dürfen read-only aus dem Projekt kommen; Woodpecker ist keine allgemeine OS-Sandbox. Fremde Appdateizugriffe müssen appseitig umgestellt werden. Ports nur ausdrücklich gebundenes Loopback, keine automatische Ausweichadresse oder neue externe Renderdienste.

### 8.3 Durchsatz und Zoo

ThroughputPlugin behält capture/begin_extract/end_extract und die getrennten Assetleser. Mesh/Image/Shader/ShaderBuffer standardmäßig, StandardMaterial/Gizmos und jedes weitere tatsächlich gelesene GPUasset ausdrücklich registrieren. Während langer Warps dürfen Nachrichten nicht vor der Extraction verschwinden; Appreader/Messageaging nicht verlängern. FrozenShadows-Schutz bei Idleextraktionen erhalten und echte Schattenausgabe vor/nach Pause prüfen. Performanceoverlay muss ohne zusätzliche Spiel-/UI-Schedules weiter rendern und darf den Zeitbeweis nicht beeinflussen. Dafür ist ein konkreter Adapter erforderlich: `performance/overlay.rs::draw` liest heute `ExtractedWindow` und ein Window-Kameratarget; `performance.rs::count_frame` verlangt Windowtarget und `ViewTarget::needs_present()`. W07 ersetzt im Offscreenpfad diese Auswahl/Geometrie durch TargetLease/Image-Output und zählt tatsächlich geschriebene Offscreenframes; native Presentzählung bleibt getrennt unverändert. Der Overlaypass muss weiterhin nach den Kameras und vor dem finalen Capturecopy liegen. P13 prüft sichtbare Overlaypixel und fortschreitende zielbezogene Framequittungen bei unveränderten Simulationsticks, nicht nur die bisherigen CPU-Samplertests. G07 inventarisiert Customshader `test_app/src/performance/overlay.wgsl` und Zoo `ground/shadow_catcher.wesl` samt Limits.

Zoo besitzt heute keinen Woodpecker-Sessionadapter. W09 ist eine echte Integration, kein behauptetes bereits existentes Feature: Renderer-/Ziel-/Readbackmodule appseitig nutzen, deterministische Exportsteuerung mit begrenzten expliziten Updates oder optionalem Sessionadapter entscheiden. Für kontrollierte Eingaben ist `InputMode`-/Cursoradaption nötig; Exportmodus ignoriert heute Inputs. Bestehende 13 Modelle × fünf Ansichten/Zustände + zwei Übersichten = **67 Outputs**, nicht 67 Acceptancefälle. Vorher-/Nachher-Semantik, Kameraorder, Labels, vier Viewports, transparentes Groundmaterial und Shader erhalten. Heutige 80-/12-Framewartewerte sind kein Bereitschaftsbeweis: asset-/pipeline-/framegebundene Quittung pro Motiv, Timeout und sichtbarer Fehler statt blindes Zählen. Kein hidden Window, kein `save_to_disk` außerhalb des neuen gewählten Exportroots. Für den eigenständigen Zooexport besitzt `test_zoo::app::run` vor Plugininitialisierung einen ausdrücklich gewählten frischen Exportroot mit eigenem TMPbereich; fehlt er oder ist er ungültig/vorhanden/symlinkbehaftet, Start ablehnen. Keine Spielsettings initialisieren oder globale TMPvariablen ändern. Ein späterer Sessionadapter nutzt stattdessen dessen Workspace; beide Bootstrapprofile in G10/P14 getrennt prüfen. G10 entscheidet die schmale Integration ohne fiktiven SDKvollsupport des Zoos.

## 9. Konfiguration, Kompatibilität und vollständiger Ablauf

### 9.1 Versionierte opt-in Einführung (G01)

Aktuelle Strict-Decoder lehnen unbekannte Felder ab. Einfach `render` in v3-Ready ergänzen wäre ein Breaking Change. Empfehlung:

- Bestehende TOML-v1-, `/v1`-Create- und game-v3-Pfade byte-/verhaltenskompatibel belassen; Native als Default.
- Neue TOML-v2-/outer-v2-Createform mit ausdrücklich gewähltem RenderRequest; neue game-v4-Negotiation **nur** bei diesem Opt-in. Striktes Parsing mit eigenen Versionstypen; kein gleichzeitiges Uminterpretieren bekannter Versionen.
- Direkte Rust-Nutzung über einen neuen Einstieg wie `Session::start_with_render(config, render_request)`, nicht durch Hinzufügen eines Pflichtfelds zu allen bestehenden `Config`-Structliteralen. Alte `Session::start(config)` bleibt Native/v3. Neue Appkonfiguration früh über reservierten internen Launchkanal übergeben, nicht über freie globale Environmentmutation.
- Alte `Capabilities` und `capabilities()` nicht still durch neue public Pflichtfelder brechen. Neuer unveränderlicher Render-Snapshot über eigenen Zugriff; Layout und öffentliche `protocol::Message`-Exhaustivitätsfolgen vor Implementierung prüfen. Neue Capabilitydaten nur im neuen Vertrag.
- Neue Capabilities: effektiver Modus, Hardware-/Softwareklasse, Backend, unterstützte fixe Zielgeometrie/Format, Readbackinstallation, virtuelle Input-/UI-Unterstützung. Keine Pfade, Secrets oder unkontrollierten Treiberstrings in öffentliche Reports. Backend-/Treiberdetailmanifest lokal und ausdrücklich vertraulich.
- Angeforderte Offscreenfähigkeit muss vor Sessionbereitschaft bestätigt werden; ein v3-Ready oder falscher Adapter ist ein sichtbarer Start-/Configfehler, niemals ein stiller nativer Fallback. Signalstations Offscreenmodus verlangt zunächst eine kontrollierte Session mit validiertem Workspace/Seed (§8.2); kein ungeregelter standalone Start. Zooexport erhält seinen gesonderten Bootstrap in W09. Kein Renderer wird vom Host installiert.

G01 muss die exakten Feldschemas, zulässigen Kombinationen, Fehlerprioritäten und Versionsdispatch an realen Parsern/Fixtures freigeben. Neue Fehlernamen sind bis dahin Vorschläge: `render_mode_unsupported`, `render_adapter_unavailable`, `render_capability_mismatch`, `render_target_unavailable`, `render_device_lost`. Native v3-Codes und native Originalorakel bleiben unverändert. Explizit geplante v4-Abweichung: `screenshot_window_unavailable` bei fehlendem/unbereitem Imageziel wird `render_target_unavailable`; Byte-/Pixel-/Zeit-/Dateisentinelassertions bleiben identisch. Keyboard-/Pointer-/Text-Windowcodes bleiben bestehen, weil sie weiter die logische Fensterentity bezeichnen. Additive Headlessfixtures, insbesondere before_first_tick und Zielwechsel, verwenden benannte v4-Fehlererwartungen statt pauschal gelockerter Codes. Neue Ready-/Capability-/Renderterminalmetadaten benötigen ebenso exakte versionierte Orakel, keine tolerant akzeptierten unbekannten Felder. Kein willkürliches Gleichsetzen von Backendverfügbarkeit und vollständiger UI-/Screenshotfähigkeit.

### 9.2 Featurematrix und Hosttrennung

Bestehende Librarykombinationen default/ui/screenshot/ui+screenshot/server/client/cli weiterführen; CLI/woodpecker_host bleiben rendererfrei aus derselben Produktionsquelle. Keine wgpu-/Winitpflicht für Server/CLI wegen Capabilitytypen. Neuer Offscreen-Appadapter benötigt explizit Renderer, optional screenshot und ui; `screenshot` allein bleibt keine Rendererinstallation. Eigener Appfeaturegraph ohne system_clipboard/Winit-Laufpfad; normale App behält Defaults. Featureunion/Binaryrollen in `_builds.py`/`_artifacts.py` gegen exakt gewählte Produkte prüfen. Ein anderer Rendererpfad/Shader-/Treiber-/Asset-/Configstand macht vorbereitete Produkte beziehungsweise Bildbaseline ungültig, kein heimlicher Neubuild.

### 9.3 Record/Replay/Inspect/Reports/Clients

- Inspect bleibt lesend ohne Tick; Entityhandles sessionlokal, Reflectionergebnisse unverändert. Keine Renderqueuewaits in Inspect einbauen.
- Recording bleibt Koordinatorarbeit mit Start-/Stopbarrieren; Screenshot-Outcomes erst nach Dateiabschluss terminal. Request-/Frame-IDs nicht in bestehende JSONL-Dateien schmuggeln. Diagnose-Receipts separat im lokalen Laufmanifest; keine Produktversionserhöhung des Recordings allein wegen anderem Renderziel.
- JSONL-v1/Application und v2/Fixedclock unverändert laden. Replay validiert vollständig vor irgendeinem Command, vergleicht effektive Clock und führt nur den effektiven Warp-Prefix aus. Historical Outcomes sind kein Oracle. Rendererwahl ist Launchvoraussetzung, nicht persistierter Worldsnapshot. Cross-mode Replay ist nur nach gleichen Seeds/Clock/Assets/Geometrie und eigenen State-/Bildorakeln qualifiziert, nicht generell versprochen.
- Replaystop stoppt aktive Warps, verhindert Tailcommands und drainiert bereits versandte nichtstoppbare Arbeit. Recording-/Replayrennen, Request-ID-Erschöpfung und Shutdownreservierung bleiben erhalten.
- CLI/REPL/Script verwenden denselben neuen äußeren Codec; einzelne Tools besitzen nie zusätzlich die Spielpipes. Parsingfehler bleiben lokal; Script-Vorvalidierung, Indizes, Paralleloutcomes, Recording-/Shutdownbarrieren, `unknown/not_submitted` und keine Resubmission bei Gap/Disconnect bleiben erhalten. PTY nur testeigen; kein Desktopterminal als Hilfsfenster.
- `rust::report_snapshot` und `rust::report_signature` besitzen explizite Protokoll-v3-/Capability-/Markdownliterale. Ihre native/v3-Originalausführung bleibt unverändert; die zusätzliche Offscreen/v4-Ausführung benötigt **benannte v4-Erwartungsfixtures** für ausschließlich bewusst neue Protokoll-/Render-Capabilitymetadaten, nicht pauschal gelockerte Assertions. Beide gehören zu P01/G01 und P11. Der Snapshotowner behält seinen eigenen tatsächlich gebauten Packagequellstand **0.56.1** und seine spätere Mutation/Unveränderlichkeitsprüfung; ein normaler vorbereiteter Appstand ist kein Ersatz. Keine neue Ausführung in dieser Planungsphase.
- Reports bleiben vertrauliche Start-/Historysnapshots. Renderfehler höchstens über vorhandenen Failure-/Diagnosepfad beobachten; keine automatische Screenshotaufnahme oder Veröffentlichung hinzufügen. Local als Entwicklungsstandard; Fake-gh nur im später freigegebenen lokalen Providercontract. Echter GitHubfall bleibt extern, kein Headless-Abnahmekriterium.
- Mehrere Spielprozesse erhalten eigene Geräte/Targets/Queues/Workspace-/TMP-/Settingsbereiche. Keine geteilt veränderlichen globalen Targetmaps oder Adapterenv. Capturetoken braucht Lebensdauergeneration über Entity-/Requestnummern hinaus. Ein lokal injizierter Deviceverlust in A darf keine fremden Prozesssignale oder falschen Outcomes für B erzeugen; B arbeitet weiter, wenn sein eigenes Gerät gesund ist. Ein gemeinsamer Adapter-/Treiberfehler darf bei B nur dessen eigenen typisierten Fehlerpfad auslösen, keine behauptete Hardwarepartitionierung. Clientende lässt angenommene Arbeit fortlaufen.
- Geordneter Stop/Shutdown integriert Render-/Readback-/PNGworker in die vorhandene Prozess-/Deadlinepolitik. Kein neues unbegrenztes GPUwait, keine zusätzliche Frist pro Session oder Provider. Der direkte SDK-Shutdown bleibt heute ohne eigenes Timeout; lokale Entwicklungswerkzeuge müssen ihren eigenen ausdrücklich gewählten Watchdog besitzen, nicht das öffentliche Verhalten heimlich ändern. G09 prüft beide Einstiege.

## 10. Coverage und zukünftige Beweise

### 10.1 Autoritative Fallmatrix

COVERAGE.json enthält **jede** der 96 heutigen IDs mit Originalsource/-symbol, Aussage, Lösungspfad, zukünftigen Proofs, Arbeitspaketen, Gate und Native-Restnachweis. 95 Produkttests + control::3.18; 94 lokale Produkttests, einer extern. Zusätzlich einzeln erfasst: 81 Harness-, zehn Groundplate- und 21 Spieltests. Die Originalfunktionen bleiben Testowner; neue Headlessvarianten dürfen ihre Erfolgsaussage nicht ersetzen.

Dispositionen:

- `unchanged-logic`: derselbe rendererfreie Vertrag bleibt; er beweist keine Renderfähigkeit.
- `offscreen-equivalent`: gleiche fachliche Assertions in neuer expliziter Headlessfixture; alte native Abnahme bleibt historisch getrennt.
- `dual-native`: Offscreen beweist den bezeichneten Teil, native Eigenschaften verlangen weiterhin eigenen Originalnachweis.
- `external-gate`: separat genehmigungspflichtig und für Headless nicht erforderlich.

Jeder Originalowner mit echter Signalstation-Appfixture benötigt zusätzlich das volle Appprofil F09/P09/W07 samt Appgates — auch bei Startup, null Ticks, Report oder Shutdown. P03/W04 ist ein früher Modulproof, nicht der vollständige EngineStartup-Owner mit LevelLoad und bis zu 120 expliziten Spielbereitschaftsticks. Dasselbe gilt für Manual-/Faultprofile; ungültige Config ohne Appstart bleibt rendererfrei. P09 ist ein unterstützender Profilqualifikationsnachweis, kein zusätzlicher versteckter Spielablauf innerhalb jedes Testfalls.

Kein offscreen-equivalent-Eintrag ist heute bestanden. Die Quellklassifikation dient Planung, nicht Testskip oder Exitcodeänderung. Vier Startup-Faultvarianten, Mehrsession-, Prozessende-/SDK-/Clockvarianten bleiben innerhalb ihres jeweiligen Owners erhalten; ein Fall mit mehreren Starts wird nicht zu einem vereinfachten warmen Reset. Nur die zehn bisherigen Keyboard-/Pointer-/Textfälle dürfen ohne zusätzliche Fixtureentscheidung ihre App teilen. Weitere Gruppierung ist unabhängig von Headless, nicht Voraussetzung dieser Migration.

### 10.2 Proofkatalog (alle **geplant**, keine Ausführung)

Alle Runtime-Proofs: gepinnte Binary-/Asset-/Font-/Shaderhashes, expliziter Settings-Seed `tests/settings.json`, Fixedclock 16.666.667 ns wo der Originalvertrag keine Applicationclock verlangt, feste Kamera/Geometrie, protokollierte Commands und genaue Tickdifferenzen. Keine globale Randomseed-Wirkung behaupten: ein späterer Zufallsnutzer muss seinen tatsächlich wirksamen Seed selbst bereitstellen. Keine Daten aus persönlichen Sessions übernehmen.

| ID | Fixe Eingabe / Oracle / wesentlicher Negativfall |
|---|---|
| P01 Vertrag/Capabilities | Alte v1/v3-Fixtures unverändert; neue opt-in-TOML/outer/game-Versionen strikt; fehlendes/falsches Ready, Adapterklasse, unbekannte Felder oder Modus führen ohne Fallback zum richtigen Fehler; Host ohne Renderer bleibt baubar. |
| P02 Keine versteckten Ticks | Früher Runner-Core ohne Capture: Manual/nulltick-Start, dann exakt 1/3/64 Ticks; Idle mit mehrfacher Extraction/Renderfortschritt; unabhängig SimulationTicks, alle Timevarianten, FixedUpdate-Zähler, Inputqueues, Changes/Removals/Assetreadercursor und App-Startupzähler vergleichen. Kein Idle-Lifecycleverlust; bewusst zusätzlicher App.update-/Tracker-Clear-Fehler muss erkannt werden. |
| P03 Ready | Echte prozedurale Szene und glTF-/Fontscope; fehlender Renderer/Scope/Kamera, fehlendes Asset, ungültiger Shader, spätes Pipelinespecializing, kein tatsächlicher Graphwrite, max_ticks und Timeout. Receipt muss an Freeze-Tick/Ziel/Generation hängen; zwei vorbereitete Iterationen + zugehörige GPUcompletion, keine Zusatzspielticks. |
| P04 Bild/Copy | Synthetisches 2D-Farbraster mit unterscheidbaren Ecken/Zeilen und explizit schwarzer gültiger Fixture; rot/grün A/B-Statewechsel nur durch expliziten Tick, Capture danach ohne Tick. Dieser frühe Proof prüft nur Render-/Copypfad; vollständige 3D-PBR/Depth/Shadows/UI/Text/Postprocessnachweise bleiben zusätzlich P08/P09/P13 in W06/W07. Falsches/leeres/stales/falsch orientiertes oder paddingverschobenes Bild scheitert am erwarteten Raster, nicht an non-black. |
| P05 Readback/Sandbox | Breiten 1/63/65/1441, Format/BGRA/sRGB/Alpha/Stride/Länge, fehlendes COPY_SRC/Target, Mapscheitern/Deviceverlust; Timeout vor bereits gepufferter Lieferung; Cancel/Entity-/Generationwechsel, Doppellieferung, falsches Requesttoken; kein Outside-Sentinel verändert, Overwriteflag korrekt, spätes Schreiben einem neuen Request unmöglich. Positiver rootinterner Symlink: Link wird durch PNG ersetzt, Referenzdatei unverändert; kein pauschales Symlinkverbot. Wartender Request → Zielgeneration wechselt → Aktivierung muss eindeutig ablehnen und Sentineldatei erhalten; Lebensdauer/Target bereits bei Annahme binden. |
| P06 Pointer/3D-Picking | Bekannte UIrechtecke und Weltobjekt-/Rayschnittpunkte bei Scale1 sowie fixem Scale2/Viewportoffset; bounds/exclusive clamp, volles MoveBydelta, Button/Scrollsemantik; falsches Ziel/Scale muss am State-/Hitentityoracle scheitern, Screenshot allein reicht nicht. |
| P07 Key/Text | **Früher Modulproof W06** in kleiner echter Bevy-UI-/Renderfixture ohne Signalstation: Press/Hold/Release, Flanken, Fokus/UTF-8-/Widgetlimits, CJK/Emoji/Zeilenumbrüche, gebundene Entity despawn/readonly; null Wirkung vor Tick; echte PNGglyphen und Fixture-State-/Textwerte. Kein Ersatz für die zehn Original-Appfälle: deren vollständige Ausführung ist verpflichtend P22/W07. Keine native IME-/Clipboardbehauptung. |
| P08 UI | Früher Modulproof W06 mit echten Legacy-Interaction-/UIwidget-Activate-/TextInput-/Tabfokuspfaden, Zorder/Blockierung, Geometrie/Clip/Viewport auf Imageziel; reale Widgetwirkung auf expliziten Fixture-State plus Layout/Pixeloracle. Signalstations tatsächliche Spieler-/Settingswirkung bleibt zusätzlich P22/P09 in W07. Bild vorhanden, aber UIbutton wirkungslos ist gezielter Negativfall. |
| P09 Signalstation | Tatsächliche 3D-PBR-/Depth-/Shadow-/UI-/Text-/Postprocesspfade und Original 1440×900-Startfixture mit unveränderten drei RGB-Landmarkorakeln aus screenshot/capture.py; LevelLoad separat; 3Dbewegung/FPV/Map, UI/Text, glTF/Fonts, Settings persistieren nur im Workspace; Actions auf expliziten Ticks. Kein Spielunit-Test als Ersatzbild. |
| P10 Persistenz/Replay | Original Recording-/Replayfälle und direkte SDK-Owner unverändert; frische Session B, gleiche Clock/Seed/Assets, Checkpointzustände plus ausgewählte Bilder. Ganze Datei vorvalidieren, invalid Tail führt null Prefixwirkung, effektive Warps, Stop/Blocked/Exklusivität/I/O und Footer nach Ende belegen. |
| P11 Reports/Clients | Lokale Reports/Fake-gh-Owner, Snapshot/Signatur/History/Pending/Gap/REPL/Script nach ursprünglichen nativen v3-Orakeln und zusätzlich ausdrücklich definierten Offscreen/v4-Metadatenorakeln; Last-/Out-of-order-Abschlüsse; dauerhafte Dateimenge nach tatsächlichem Server-/Providerjoin; keine Liveveröffentlichung. Keine G12-Abhängigkeit. |
| P12 Isolation/Cleanup | Zwei echte Prozesse/Workspaces/Rendergeräte mit gleicher lokaler Requestnummer; Input/Dateien/Capture/Ready bleiben adressiert. Lokal injizierter A-Deviceverlust: B mit gesundem Gerät läuft weiter, keine fremden Signale/Zuordnungen. Gemeinsamer Adapter-/Treiberfehler: mehrere jeweils eigene typisierte Terminalfehler zulässig, keine physische Isolation behaupten. Capture/Encoding während Stop, Starts abbrechen, gemeinsame Frist, SIGINT/TERM/harte Enden, SDKdrop; keine PID-/Gruppenreste oder Workerdateitails als Erfolg ausgeben. |
| P13 Assets/Pause/Overlay | Gezählt lange Warps mit Material-/Mesh-/Image-/Shader-/Gizmoänderungen über mehrere Tickmessages; Appreader unverändert, Renderer sieht letzte gültige Zustände; Idle-/Shadow-/Overlayframes weiter, TPS/Ticks/Zeit bleiben pausiert; bewusst verlorener Renderassetstrom muss am Bild **und** Readeroracle scheitern. |
| P14 Zoo | 13 definierte Modelle, front/back/side/used/locked und zwei Übersichten; pro Shot feste Kamera/VisualState/Assets und Receipt, PNGmaße/Label/Objekt-/Statevergleich. Verzögerte Shader/Assets, falscher Viewport oder falscher Zustand darf nicht nach bloßer Framezahl als Erfolg gelten. |
| P15 Harness/Produkte | 96 Original-IDs plus klar getrennte neue Headlessvarianten; Auswahl/List/Featurebuildplan/Präparatvalidierung/Lease/StopnachFehler/typed Control3.18, Exit2, Activitycursor, PTY und Cache-/Mutationsharness unverändert. Kein tatsächlicher Mutations-/Buildlauf in dieser Planung. |
| P16 Native Residuen | Eigene native Spur für RawHandle/Surface/Presents, tatsächliche Fokus-/Cursor-/Clipboard-/IME-/Displayintegration und bestehende Fehlercodes; nur was im ursprünglichen Scope zugesichert ist. Dynamische Resize/DPI/Occlusion nicht neu als universell gelöst versprechen. |
| P17 Adapterprofile | Pro zugelassener OS/Backend/Treiberkombination tatsächlichen nicht-Noop-Adapter, Render-/Copyformate, Shader-/PBR-/Storage-/Texturelimits, Device-loss und null Displayverbindung belegen. Hardware und Software getrennt, kein `force_fallback_adapter` als Beweis CPUfähigkeit. Gleiche Szenenorakel, eigene Performancebaseline; fehlender Adapter ergibt fehlende Voraussetzung/klare Produktdiagnose, kein grüner Skip. |
| P18 Beispiele/Messung | REPL/CLI/Script, movement/feedback/text/replay und performance.py als getrennte Anwendungseinstiege mit expliziten Konfigurationsvarianten; Sample-/Tick-/Clock-/Inspect-/Stopdefinitionen erhalten. Offscreen Render-FPS sind keine Monitorpresents; native FPS separat; keine ungemessenen Speedups. `performance.py::ready/raise_window/run/verify` fordert heute osascript-Fokus und system_profiler-Displayabfragen auch während der Messung. W10 braucht einen ausdrücklich getrennten Headless-Messadapter ohne diese Aktionen: keine Fensterposition/-aktivierung/Displayabfrage, Manifest mit `raise_owned_window=false`, festem TargetDescriptor und tatsächlichem Adapter-/Treiberprofil statt Displaymetadaten. Auswahl vor erstem Appstart, nicht nur andere Appflags. P18 enthält einen Negativguard, der jeden osascript-/system_profiler-/Fokus-/Displayaufruf im Headlessprofil ablehnt; native Messung bleibt unverändert separat. Der direkte Pipe-Owner `Game.__init__/ready` übernimmt **selbst** den gemeinsamen Bootstrap-/Admissionvertrag, da Session::start hier nicht beteiligt ist: frischer eigener Workspace/config/data/tmp vor Appstart, expliziter Seed, TMP/TEMP/TMPDIR nur im Kind auf diesen tmp-Root, kontrollierte WOODPECKER-/WGPU-Umgebung und qualifizierter Binary-/Asset-/Config-/Profilhash. V4-Ready und komplette Render-/Adapteranforderung strikt gegen gemeinsame versionierte Schemafixtures prüfen, nicht nur status==ready. `--binary` darf im Headlessprofil weder alte native v3-Referenzen noch falsche Capabilities akzeptieren; kein Ersatzprozess. Negativproofs: v3-Ready, falsche Adapter-/Rendercapability, fremder geerbter TMPpfad (im Kind überschrieben, Elternumgebung/Sentinel unverändert), ungültiger/vorhandener Workspace und abweichendes Binaryprofil. P18/W10 gehört zusätzlich G01/G08/G09. Der direkte `Game._read/result/close` erhält einen v4-Dispatcher mit getrennten korrelierten Antworten und asynchronen Terminalnachrichten statt blindem Zugriff auf `message[request_id]`: Zustände Starting → Ready → Terminal → Closed, Fatal einmalig latchen, bereits terminale Ergebnisse erhalten, offene Requests einmalig technisch beenden beziehungsweise ungewisse Einreichungen als unknown erhalten. Nach Fatal keine neuen Commands, kein Stop-/Shutdownversuch an einen unbrauchbaren Renderer und keine Resubmission. Reader weckt alle Wartenden mit der primären Renderdiagnose; close drainiert Pipes/Writer/Kind unter **einem** ausdrücklich gewählten Gesamtbudget, dann nur identitätsgeprüfte eigene Prozessgruppe erzwingen und als Cleanupfehler/unvollständig dokumentieren. Die ursprüngliche öffentliche SDK-Shutdownsemantik bleibt unverändert. Gemeinsame strikte v4-Varianten-/Schemafixtures verhindern einen zweiten permissiven Codec. P18 prüft Fatal vor Ready als Launchfehler, Fatal während Warp/Capture/Pause, konkurrierende Response/Terminalreihenfolge, verlorenen Transport und laufenden Writer; kein Parser-KeyError oder Folge-Timeout als vermeintlich korrekter Renderterminalausgang. |
| P19 Frühes Ziel-/Adapteradmission | Nur ausgewähltes lokales Hardwareprofil: tatsächlicher nicht-Noop-Adapter passend zum vollständigen Selektor; unbekannter Name, falscher Adapter derselben Klasse und widersprüchliche geerbte WGPU-Overrides müssen scheitern statt Bevy-Namensfallback. Für das 2D-Ziel erforderliche Format-/Render-/Copylimits; begrenzte direkte Device-Texturallocation und Descriptor/Generation/Geometrie/Scale konsistent, kein nativer Handle/Display. Dieser Admissionprobe benötigt weder RenderApp-Extraction noch den W03-Runner oder App.update; tatsächliche Frameausgabe folgt P04/W05. Noch kein UI-/Picking-/Software-/Vollspielnachweis; vollständige P06–P09/P17 bleiben Pflicht. |
| P20 Externer Realprovider | Genau der ursprüngliche separat freizugebende echte GitHubfall einschließlich Created→Existing und eigenem Cleanup. Nur F20/G12, niemals lokaler Headlessproof oder Voraussetzung einer lokalen Paketfreigabe. Hier strikt verboten/nicht ausgeführt. |
| P21 Runner-Capture-Kombination | Verpflichtende W05-Regression nach vorhandenem P02/W03-Core und Offscreen-Copy/Map: mehrfacher Idle-Capture erhält dieselben unabhängigen Time-/Tick-/Fixed-/Input-/Change-/Removed-/Assetreaderorakel. Zusammen mit P05 Copy-/Map-/Writer-Fatalinterleavings nachweisen. Ein absichtlich zusätzlicher App.update-/Repairtick oder Trackerverlust muss scheitern. Früher Core und spätere Kombination jeweils vollständig. |
| P22 Vollständige Originalinputgruppe | Verpflichtende W07-Appabnahme aller **zehn** heutigen Keyboard-/Pointer-/Textowner gegen echte Signalstation, nicht gegen die frühe Modulfixture. Originale `_shared.py`-Vorbereitung, manuell gewählter Startup, höchstens 120 explizite Ladeticks, F9 und drei gezählte Resetticks sowie fallweise Aktionsticks bleiben sichtbar. Vollständige ursprüngliche Spieler-/Kamera-/UI-/Text-/Settingsassertions, reale Inputs/Picking und selektive Bildorakel; keine OSinputs, keine zusätzliche unprotokollierte Aktion. Frühe P06/P07/P08 bestehen vorher separat, ersetzen keinen Originalowner. |

Bildstrategie: synthetische LDR-Raster auf demselben Backend exakt; komplexe Bevy-Ausgabe mit vorhandenen fachlichen Region-/Stateorakeln, fixen Kameras und getrennten Treiberbaselinebildern. Treibertoleranzen **vor** Prüflauf anhand genehmigter Referenzcaptureserie festlegen, nicht nach Fehlschlag erweitern. Semantische State-/Objektprüfungen bleiben exakt, wo deterministisch. Neue Renderingqualitätsoptionen dürfen nicht still Shader/PBRfeatures abschalten, um Software grün zu machen. Größere Toleranz/Featuremangel = Gate-/Produktentscheidung, nicht automatischer Erfolg. Schwarze Fixture explizit gültig; schwarzes Bild der Signalstation scheitert dagegen an konkreten Landmarks.

## 11. Geordneter Integrationsfahrplan

**Alle Pakete sind zukünftige Implementierungsarbeit, hier nicht freigegeben.** Kein fiktiver Termin, kein pauschaler Komplettumbau. Fachlicher Implementierungsowner ist jeweils das genannte Modul; Produkt-/Freigabeentscheidungen beim Projektverantwortlichen. Review/Tests sind Liefergegenstände, nicht nur „später testen“.

| Paket / hängt ab | Inputs, echte betroffene Dateien und Liefergegenstand | Abnahme / Gate / Rückweg |
|---|---|---|
| W00 Scope/Proofspec / — | BASELINE/COVERAGE, aktuelle target.md/Testowner; zukünftige versionierte Headless-Abnahme und Rechte spezifizieren, keine alten Regeln überschreiben | IDs/Originalorakel vollständig, neue Proofinputs fest; Dokumentstand zurücknehmen ohne Produktwirkung |
| W01 Version/Config / W00 | `src/cli/config.rs`, `server/protocol.rs`, `session/{host,protocol,coordinator,launch}.rs`, `report/snapshot.rs`, beide crate-Fassaden: neue opt-in-Typen/Dispatch/Anforderungsbestätigung | P01, G01; v1/v3-Regression und rendererfreier Host; nur native Default aktiv lassen, neue Version verweigern statt uminterpretieren |
| W02 Renderziel / W01 | neues `rendering/target.rs`, bestehendes `session/window.rs`; TargetDescriptor/Lease, native Adapter 1:1, Offscreen-Image und logische Geometrie; Appinstallation vor Winit | P19, G05.target/G07.admission; kein nativer Handle im Offscreen, invalid Größen/Limits; Offscreen opt-in abweisen, alte Adapter behalten |
| W03 Kontrollierter Runner / W02 | neues `rendering/pump.rs`, `plugin.rs`, `plugin/render.rs`, `clock.rs`, `throughput.rs`; Lifecycle-/Tracker-/SubApp-Policy und Idlefortschritt | P02, G02.core; P13/G02.assets folgt ausdrücklich erst in W07; ohne erfüllten Coregate **nicht** zu Vollintegration weitergehen; Native unverändert, kein blindes ScheduleRunner-Fallback |
| W04 Ready / W03 | `startup.rs`, `startup/render.rs`, neues `rendering/proof.rs`; zielbezogene Tickets, Freeze-/Rendergeneration, Rendergraph-/Submissionbeleg | P03/P19, G03/G07.startup; P17 ist spätere Profilqualifikation, keine Vorbedingung dieses Paketabschlusses; native Prüfungen gleich stark; falsche/leere Graphen fail, bei Fehler keine Ready, Offscreen deaktiviert lassen |
| W05 Capture / W04 | `screenshot.rs`, `screenshot/capture.rs`, `capture/surface.rs`, neues `screenshot/readback.rs`, bestehendes `destination.rs`; Offscreen Copy/Map, Tokens, Timeout/Writerordnung, RGB-PNG | P04/P05/P21, G04/G09.capture; vollständige Mehrsession-/Device-loss-Abnahme P12/G09.full folgt W08; erste vertikale Manual-Etappe besteht einschließlich nulltick/1tick/pause/shutdown; kein neues Captureformat für alte Clients, keine halben Dateien als Erfolg |
| W06 Input/UI / W05 | `input/{keyboard,pointer,text,ui}.rs`, `pointer/{cursor,motion}.rs`, app Cursor-/UIadapter; Image-Location, logisches Window für Keyfokus, Legacy/current UI | P06/P07/P08, G05.module/G08.input; vollständige frühe Modulfixtures und negative Inputs, keine OSwirkungen; die zehn Original-Key/Pointer/Textowner erst vollständig mit P22/W07 abnehmen; bei Gatefehler Headless-UI unsupported, keine Gesamtfreigabe |
| W07 Ganze Signalstation/Persistenz / W06 | `test_app/src/{app,woodpecker,input,player,ui,performance}`; gleiche Assets/Renderer, Featuregraph, EngineReady, Recording/Replay/Report/Clientcode nur dort wo Version/Adapter erfordert | P09/P10/P11/P13/P22, G02.assets/G05.app/G07.game/G08.app; echte vollständige Spielausgabe; alle betroffenen Originalfälle Headless zusätzlich, nicht skippen; Nativeprofil/RGB-/JSONLverträge erhalten |
| W08 Isolation und Abschluss / W07 | `session/{process,coordinator,workspace}.rs`, `server/{entry,reports,transport}.rs`, Captureworker; Device-loss-/Drain-/Deadline-/Process-Ownerintegration | P12, G09.full; zwei echte Sessions, Shutdown/Drop/Startabbrüche und Dateitails; vor Freigabe eigene neue Prozesse beenden, kein Neustartfallback |
| W09 Zoo / W08 | `test_zoo/src/{app,camera,ground}.rs`, `test_zoo/Cargo.toml`; echter windowfreier Export mit 67 Motiven, deterministische Kontrolle und Captureack | P14/P19, G10; P17 folgt in W11; nicht einfach hidden Window „headless“ nennen; ursprünglicher nativer Viewer/Exportmodus bleibt wählbar |
| W10 Gesamte Test-/Entwicklerintegration / W09 | `tests/{acceptance,_builds,_artifacts,_shared,_support,interactive}.py`, Testowner, feature/build contracts; examples/game_tests und **examples/performance.py mit getrenntem Headlessadapter ohne osascript/system_profiler/raise_window**; neue Headless-Konfigurationen und eindeutige Ergebnisachsen | P15/P16/P18, G01.direct/G08.direct/G09.direct/G11; jeden Originalfall tracebar, v1/native unverändert, separate lokale Headlessausführung ohne CI; Rollback neues Profil entfernen, keine alten Nachweise überschreiben |
| W11 Plattform/Softwarequalifikation / W10 | gepinnte Bevy/wgpu-/Treiber-/Adapterprofile; lokale Bereitstellungsanleitung nach separater Freigabe | P17 + relevante P02–P14 pro Profil, G06/G07; fehlendes Profil blockiert dessen Supportaussage, kein unsichtbarer Featureabbau |
| W12 Abschluss/Handoff / W11 | Freigabematrix, Belege, Migrationsanleitung und Restnative-/externe Gates | lokale Headlessfähigkeit nur für tatsächlich qualifizierte Profile erklären; externe G12 bleibt separat; optionaler CI-Zielentwurf erst mit neuer ausdrücklicher Freigabe |

Die Tabelle zeigt konservative Lieferreihenfolge; einzelne reine Specs können vorher geschrieben werden. Kein Zirkelschluss „vollständiger Lauf beweist jeden ungeprüften Treiber“. Hardware-Vertikalslice W01–W05 liefert früh einen überprüfbaren Pfad; UI/Persistenz/Zoo/Software sind erst nach ihren eigenen Paketen behauptbar. Für W05 wird eine kleine echte 2D-Farbfixture benutzt; PBR/UI/Text sind keine Vorbedingung dieses begrenzten Slices, sondern bleiben eigenständige Pflichtnachweise in W06/W07. Die spätere Gesamtabnahme darf sie nicht auslassen.

### 11.1 Abnahmereihenfolge und Teilgates

Ein Proof besitzt genau ein **Implementierungs-/Ausführungspaket** (`proof.owner_package`). Die Zuordnung zu mehreren späteren Konsumenten in `proof.packages` ändert diesen Owner nicht. `package.acceptance_proofs` nennt nur für diesen Paketabschluss nötige Proofs; Kanten zu deren Ownern und `requires_packages` werden zusätzlich zu `depends` auf Zyklen geprüft. Vollqualifikation ist kein rückwirkender Blocker früherer begrenzter Liefergegenstände.

G05.target und G07.admission: P19/W02; G02.core: P02/W03; G03, G07.startup und G09.receipt: P03/W04; G04 und G09.capture: P04/P05/P21/W05; G05.module und G08.input: P06–P08/W06; G05.app: P22/W07; G02.assets, G07.game und G08.app: P09/P13/P22/W07; G09.full: P12/W08; G10: P14/W09; G01.direct/G08.direct/G09.direct und G11: P15/P16/P18/W10; G06 und G07.profiles: P17/W11. G01.protocol gehört W01; der direkte Performanceeinstieg muss denselben Vertrag zusätzlich in W10 nachweisen, ohne W01 rückwirkend von W10 abhängig zu machen. G12/P20 bleibt außerhalb aller verpflichtenden lokalen Paketabnahmen.

Ein frühes Teilgate schließt **nicht** das gesamte übergeordnete Gate. Alle zwölf Gates bleiben in dieser Planungsphase offen. Insbesondere P19 behauptet keine UI-, Software- oder Vollspielparität. Jeder spätere Pflichtnachweis bleibt im vollständigen Abnahmeumfang.

Frühe Module werden in einer **neu geplanten** kleinen echten Renderfixture geprüft (beispielsweise `tests/fixtures/headless_render/`), die die realen Modulinterfaces und Bevy-Renderer/UI verwendet, keine nachgebaute Produktlogik. W02 liefert deren direkte Device-/Targetadmission, W03 den Runner-Core, W04 Ready, W05 Capture, W06 virtuelle Geräte/UI. Diese Fixture ist ausdrücklich kein Original-Acceptancefall und keine Signalstation-Ersatzabnahme; sämtliche Original-Appowner werden in W07 oder ihren späteren isolierten Szenarien auf dem vollständigen Appprofil ausgeführt.

### 11.2 Migration und Rücknahme

1. Nativeverhalten/Fixtures einfrieren, neue Contracts/Profiles additiv definieren (W00/W01).
2. Native Renderzieladapter zuerst ohne Verhaltensänderung extrahieren, dessen originale Surface-/Startup-/Captureprüfungen erhalten (W02).
3. Offscreen hinter explizitem Opt-in aktivieren; keine Erkennung anhand Umgebung. Gateversagen vor Freigabe sichtbar machen, kein stilles Abweichen (W03–W05).
4. Headless-Varianten der echten Testowner additiv anschließen; alte Grafikregeln erst durch **separat vereinbarte** neue Abnahme ergänzen. Alte Teilauswahl weiterhin Exit2 (W06–W10).
5. Profilqualifikation versioniert dokumentieren. Software nicht als kostenloser gleichwertiger Hardwareersatz bezeichnen (W11).
6. Rücknahme deaktiviert neue Opt-in-Startpfade, behält gespeicherte Artefakte/Fehlbelege und alte Decoder/Recordings. Laufende neue Sessions müssen über ihren ursprünglichen Owner geordnet beendet werden; kein Moduswechsel live. Neue Dateien nicht als alte Version uminterpretieren; keine bereits erzeugten Recordings/Reports automatisch löschen.

## 12. Offene Proof-/Entscheidungsgates

Alle Gates sind **offen**. „Owner“ ist Verantwortungsvorschlag für spätere Arbeit, kein bereits beauftragter weiterer Agent. Diese Planung erlaubt keine der beschriebenen Runtimeaktionen.

| Gate | Frage / erforderliche Evidenz / Entscheidungseigner | Wirkung bei Nein |
|---|---|---|
| G01 Kompatibilität | Modulowner Config/Protocol + Projektverantwortlicher: exakte v2/v4-Schemas, Fehlervorrang, public Rust-Exports/source compatibility und alter Client/neue App-Matrix durch P01 belegen | keine neue Version veröffentlichen; native Defaults behalten |
| G02 Runner | Session-/Bevyowner: P02/P13 mit Message-/Changed-/Removed-/Asset-/SubApp-/Pluginlifecycle und pipelined/single-thread Pump; öffentliche Bevyintegration hinreichend? | echter Headlessgesamtpfad blockiert; nicht durch idle App.update oder versteckte Ticks ersetzen |
| G03 Ready/Graph | Renderowner: P03 belegt tatsächlichen Zielwrite und zugehörige GPUsubmission, Freeze und zwei vorbereitete Iterationen | kein EngineReady/keine Vollintegration |
| G04 Readback | Captureowner: finaler Graphhook, Copy/Map/Token-/Timeout-/Format-/Fehlerpfad auf Bevy0.20/wgpu30 mit P04/P05 | kein Capture-/Grafikerfolg; alternatives enges Adapterdesign entscheiden |
| G05 UI/Geometrie | Input-/UI-/Appowner: P06–P08 plus keine nativen Fenster, Image-Legacy-Interaction, Focus/Key/Viewport/Scale; logische Windowentity wirklich ohne OSbedarf | UI-/Inputgesamtparität blockiert, keine Reducedsuite als Ersatz |
| G06 Adapter/Plattform | Renderowner + Produktentscheidung: P17 auf konkreten macMetal/LinuxVulkan-Hardware-/Linuxsoftwareprofilen; Lizenz-/Installations-/Display-/Limit-/Performanceevidenz | jeweiliges Profil unsupported/unqualifiziert, keine universelle CPUbehauptung |
| G07 GPUfeatures/Assets | App-/Renderowner: tatsächliche PBR/UI/2D/Text/Shadow/Overlay/Gizmo/Zoo-Shader/Material-/GPUasset-Scope und Limits mit P03/P09/P13/P17 | betroffener Zielumfang blockiert, Features nicht still abschalten |
| G08 Native Dienste | App-/Buildowner: separater Featuregraph ohne Systemclipboard/Winitzugriff, Focus/Cursor/IMEgrenzen und Settingsisolation mit P07/P08/P09 | kein zugesicherter Betrieb ohne Desktopzugriff |
| G09 Abschluss/Device-loss | Session-/Capture-/Serverowner: P05/P12 für Deviceverlust, Cancel/Timeout, offene Writer und ursprüngliche gemeinsame Frist/Drop | keine Ressourcen-/Shutdownfreigabe; Fehler/unvollständig statt Grün |
| G10 Zoo | Zoo-/Renderowner + Produktentscheidung: kontrollierter Export- versus Sessionadapter, alle 67 Motive ohne Window und stabile Ackorakel P14 | Zoo nicht als vollständig integriert ausgeben |
| G11 Abnahmevertrag | Testowner + Projektverantwortlicher: P15/P16/P18, Originalassertions und Native-Restaussagen akzeptieren, neue Umfangs-/Exitachsen benennen | alte Abnahme bleibt ausschließlich gültig, Headless nur unvollständige Zusatzspur |
| G12 Externe Veröffentlichung | ausschließlich separate spätere Nutzerfreigabe für echten GitHub-/Providerfall mit benanntem Target, Scope und Cleanup | bleibt verboten/nicht ausgeführt; kein Headless-/GPUblocker |

### Späterer CI-Zielentwurf, nicht heutige Arbeit

Erst nach lokaler Headlessqualifikation und neuer ausdrücklicher Freigabe könnte ein separat versioniertes Profil auf geeigneten Runnern laufen. Hardware, Treiber, Softwareadapterinstallation, Secrets, Kosten und Artefaktvertraulichkeit müssten vorher entschieden werden. In diesem Arbeits-PR wird ausschließlich der bestehende Workflow im Quellcode [pausiert](../../.github/disabled-workflows/README.md); kein Runner wird eingerichtet, keine CI aktiviert oder ausgeführt. Lokaler Entwicklerstart/Build/Test muss unabhängig von CI spezifiziert und benutzbar sein. Die bewusste spätere Reaktivierung ist ein eigener Schritt, keine automatische Folge der Implementierung.

## 13. Quellregister und Reproduzierbarkeit

| ID | Tatsächliche Quelle / Anker | Belegt |
|---|---|---|
| S01 | `README.md` — `Current implementation` | v3-Umfang, Unix, native Abnahmegrenzen |
| S02 | `Cargo.toml` — `[features]`; `Cargo.lock` | Bevy0.20.0, Rust≥1.97.1; wgpu/core/hal/Naga30.0.1, winit0.30.13 |
| S03 | `tests/acceptance.py` — `def inventory` / `if args.list` | tatsächliche Discovery, 96 Einträge inkl Control, ehrliche Teilverdicts |
| S04 | `tests/README.md` — `Voraussetzungen und Sicherheitsgrenzen` | Rechte, sichtbare Fenster, keine Headlessersatzabnahme |
| S05 | `docs/api/target.md` — `Technischer Startup` | maßgeblicher Session-/Zeit-/Input-/Screenshot-/Persistenzvertrag |
| S06 | `src/session/plugin.rs` — `fn run_tick` | Control, begrenzte Batches, Inputs/Clock/Schedules, Renderstart Tick1 |
| S07 | `src/session/plugin/render.rs` — `FrozenShadows` | Extractiongate und weiterlaufender Renderer ohne Gameplayticks |
| S08 | `src/session/startup.rs` — `fn prerequisites` | CPU-/Scenescope, Stabilität und Freeze |
| S09 | `src/session/startup/render.rs` — `struct Ticket` / `fn submitted` | Window/Surface/Kamera, GPUassets/Pipelines, zwei Iterationen, Queuecallback |
| S10 | `src/session/screenshot/capture.rs` — `READBACK_TIMEOUT` | serielle Queue, RawHandle, asynchroner Worker und Deadline |
| S11 | `src/session/screenshot/capture/surface.rs` — `fn check` | framegebundener erster Surfacebeweis gegen ungefüllte Buffer |
| S12 | `src/session/screenshot/destination.rs` — `pub(super) fn write` | cap-std-Confinement, erlaubte interne Links, Temp/sync/Rename/Overwrite |
| S13 | `src/session/input/pointer/cursor.rs` — `pub(super) fn location` | Window-Pointertarget, Geometrie/Clamp |
| S14 | `src/session/throughput.rs` — `fn begin_extract` | getrennte App-/Renderassetmessages |
| S15 | `docs/performance.md` — `Wiederholbar messen` | Zeit-/FPS-/TPS-/Messgrenzen und explizite Ticks |
| S16 | `src/session/input/ui.rs` — `fn focus` | heutiger Legacy-Windowcursoradapter |
| S17 | `src/session/input/text.rs` — `MAX_BYTES` | Textfokusbindung/UTF-8/Flushing |
| S18 | `src/session/protocol.rs` — `pub struct Capabilities` | strict v3-Handshake, unbekannte Felder inkompatibel |
| S19 | `src/session/host.rs` — `pub struct Config` | öffentliche Typen, SDKbesitz, unveränderliche Capabilities |
| S20 | `src/session/process.rs` — `pub fn terminate` | Unixgruppen-/Prozessabschluss |
| S21 | `test_app/src/woodpecker.rs` — `pub fn install` | reale Startup-/Input-/Reflection-/Throughputanbindung |
| S22 | `test_app/src/input/devices.rs` — `pub fn position` | echte appseitige Windowtargetabhängigkeit |
| S23 | `test_zoo/src/app.rs` — `visible: !offscreen` | vorhandenes Offscreenimage, dennoch natives hidden Window |
| S24 | `tests/_builds.py` — `def commands`; `tests/_artifacts.py` | Featuregraph und vorbereitete Programme |
| S25 | `crates/host/src/lib.rs` — `../../../src/session/host_facade.rs` | gemeinsame rendererfreie Hostquellen ohne zweiten Codec |

Zusätzlich gelesen: AGENTS/Brief, tests/README vollständig, testing-plan und geschützte Fenstergruppen vollständig, target.md vollständig, performance.md; relevante ADRs 0003/0004/0005/0010/0011/0015/0022/0025; konkrete Clock-/Keyboard-/Text-/Pointer-/Capture-/Throughputquellen, CLI-/Servertypen, Cargo-Pakete, Signalstations App-/Input-/Font-/Settingsadapter und Zooquellen. Historische ADRangaben werden nicht über den aktuellen target.md-/Quellstand gestellt. Konkrete Einzeltestquellen und Originalfunktionen stehen im statischen Inventar, Detailprüfung durch Testreview.

Die Planungsanalyse verwendete fünfzehn lokal gepinnte Dependency-Primärquellen: Bevy0.20.0 `examples/app/headless_renderer.rs`, bevy_app `app.rs/sub_app.rs/schedule_runner.rs`, bevy_render `settings.rs/renderer/mod.rs/pipelined_rendering.rs/view/window/screenshot.rs`, bevy_camera `camera.rs`, bevy_ui `focus.rs`, bevy_picking `pointer.rs`, bevy_clipboard `lib.rs` sowie ergänzend bevy_render `error_handler.rs/lib.rs` und wgpu-core30.0.1 `device/queue.rs`. Versionsbasis ist der oben genannte Repository-Commit samt Cargo.lock. Die in den technischen Abschnitten genannten BASELINE-/SOURCE_ADDENDUM-/OWNER_REQUIREMENTS-Register gehören zu den lokalen historischen Prüfbelegen und werden nicht als ausführbare PR-Prüfkette mitveröffentlicht. Sie belegen vorhandene Interfaces und Risiken, **nicht** erfolgreiche Ausführung auf einer Maschine. W00 muss eine neue implementierungsbezogene Quell-/Testbaseline bereitstellen; historische Planungsrechte gelten nicht automatisch fort.

## 14. Status und Handoff

Dieser Arbeits-PR liefert Plan, Inventar und reversible CI-Pause, keine Headlessimplementierung und keinen Runtimeproof. Die fünf vollständigen Council-Runden mit insgesamt fünfzehn Einzelreviews endeten mit einer bedingten Zustimmung zum Plan; Einzelheiten und Nachweisgrenzen stehen im [Reviewstatus](README.md#reviewstatus). Alle zwölf Runtime-/Entscheidungsgates bleiben offen.

Nächster Produktschritt wäre nach gesonderter Freigabe W00: aktuelle Quell-/Testbaseline, additive Verträge und Proofspecs festlegen. Vor einer Ready-/Readbackfreigabe muss insbesondere der öffentliche, nebenwirkungsfreie GPU-Fehlerbeobachtungshook aus G02/G03/G04/G09 tatsächlich belegt sein. Danach folgt der begrenzte W01–W05-Vertikalslice; UI, Gesamtspiel, Persistenz, Zoo und Plattformqualifikation bleiben spätere eigenständige Pflichtpakete.

## 15. Ergänzung: gemeinsame Testausführung

### 15.1 Zwei getrennte Fragen, eine Testintegration

Auf den nachträglichen dokumentarischen Folgeauftrag hin werden die
[Fensterbefunde und thematischen Testgruppen](../testing-window-groups.md) in die
Planung aufgenommen. Die Bestandsaufnahme enthält alle 96 Originaleinträge mit
Quellverweisen, außerdem die separat gezählten Performance-Messblöcke und
Anwender-Beispiele. Sie ergänzt die **Ausführungsplanung**, nicht die fachliche
Disposition aus COVERAGE.json.

Zwei Entscheidungen müssen pro Test getrennt festgehalten werden:

- **Renderanforderung:** rendererfreie Logik, echte Offscreen-Ausgabe oder echter
  nativer Fenster-/OS-Nachweis. Hierfür bleiben die Dispositionen und
  Native-Restaussagen aus Abschnitt 10.1/COVERAGE.json maßgeblich.
- **App-Lebensdauer:** kompatible laufende App weiterverwenden, mehrere Sessions
  parallel benötigen, einen echten frischen Start verlangen oder eine App
  endgültig beenden. Headless allein beseitigt weder Startkosten noch
  Zustandskontamination; ein gemeinsames natives Fenster beweist umgekehrt keine
  Offscreenfähigkeit.

Ziel ist **möglichst eine App pro kompatiblem Profil und benötigter Lebensdauer**,
kein neuer Spielprozess pro Test und kein einziges globales Fenster um jeden
Preis. Das gilt für Native und Offscreen jeweils mit eigenem Ergebnisnachweis.
Die bisherige R05-Aussage in Abschnitt 10.1 bleibt als Istgrenze richtig: Heute
teilen nur zehn Eingabetests ihre App. Die zusätzliche Gruppierung wird jetzt
bei W00/W10 mitgeplant, bleibt aber eine gesondert zu beweisende
Fixture-/Harness-Änderung und keine Voraussetzung für den frühen W01–W05-Slice.

### 15.2 Quellenbasierte Wiederverwendungskandidaten

Die folgende Kurzzuordnung ist keine Startreihenfolge und keine zugesagte
Fensterzahl. Die vollständige Zuordnung je Test steht ausschließlich in der
[Bestandsaufnahme](../testing-window-groups.md); ein Python-Modul bezeichnet dort
jeweils seinen einzelnen Originalowner.

- **Haupt-App:** 35 Kandidaten einschließlich Nulltick-Startprüfungen, Inspect,
  Keyboard/Pointer/Text, Ticks/Pace, Screenshots, Recording/Replay, CLI/REPL,
  Clienttrennung, nichtfatalem Report und typisierter Kontrolle. Heute sind nur
  die zehn Eingabeowner angeschlossen. Start-, Live- und Abschlussphasen müssen
  separat erhalten bleiben; diese 35 Fälle sind noch kein bestandener Verbund.
- **Direktes SDK:** vier Kandidaten für einen gemeinsamen echten Session-Owner.
  Pending-/History-Verträge nicht durch CLI-Aufrufe ersetzen. Der aktuelle
  Rust-Prozessisolationsmechanismus muss ausdrücklich berücksichtigt werden;
  ein anderes Wrapper-Binary allein teilt keine App.
- **Abweichende Profile:** zehn Einträge mit Engine-/Faultstartup,
  Applicationclock, Diagnosemodi, Activity-/Dateilimits, Fakeprovider oder eigener
  Snapshot-Binary. Erfolgreiche Apps innerhalb eines kompatiblen Profils
  weiterverwenden; echte Startvarianten nicht durch Warmreset ersetzen.
- **Mehrsession:** acht Einträge. Ein kompatibles Paar darf mehrere
  nichtterminale Fälle tragen; Datei-/Input-/Requestisolation und ein frischer
  Replay-Zielkontext bleiben echte Nachweise.
- **Terminale Fälle:** elf Einträge. Passende zuvor genutzte Apps möglichst hier
  abschließen; verschiedene Shutdown-/Panic-/Exit-/Drop-Pfade brauchen jedoch
  verschiedene Lebensdauern. Ein benötigter tick-/commandfreier Start-/Endfall
  darf nicht unbemerkt zu einer warmen Gameplayprüfung werden.
- **Ohne natives Spielfenster:** 27 Einträge nicht in die Grafikfixture zwingen.
  Server-, Peer- und World-Prozessisolation kann trotzdem sinnvoll bleiben.
- **Echter GitHubfall:** ein separat freizugebender externer Eintrag, weiterhin
  P20/G12. Weder automatischer Bestandteil einer gemeinsamen App-Gruppe noch
  Headless-/Grafiknachweis.

### 15.3 Anschluss an Arbeitspakete und Abnahme

Die folgenden Liefergegenstände ergänzen die spätere Ausführung von W00/W10;
COVERAGE.json, die 22 geplanten Proofs und die zwölf offenen Gates werden dafür
nicht rückwirkend geändert oder als bestanden markiert.

- **W00 — Testausführungsvertrag:** Jeden Originalowner aus COVERAGE.json mit
  seiner Ausführungsgruppe verbinden. Bei Python den eindeutigen Modulnamen auf
  die vollständige Original-ID auflösen; Rust-/Control-IDs exakt übernehmen.
  Pro Fall Render-/Binary-/Clock-/Providerprofil, direkten SDK- oder Serverowner,
  benötigte Startphase, explizite Vorbereitung, Reset-/Dateivorbedingungen,
  terminale Aktion und eventuell verschobene Endassertions festhalten.
  Noch keine Reihenfolge aus den Tabellenpositionen ableiten.
- **W07/W08 — Zustands- und Abschlussbeweise:** Gemeinsame Signalstation-Fixture,
  Mehrsession- und Owner-Lebensdauern bleiben den ursprünglichen Tests zugeordnet.
  Der heutige F9-Reset stellt weder Zeit/History noch Recording/Replay/Pending,
  Reportjobs oder alle Dateizustände zurück. F9 kollidiert außerdem mit der
  ReportProbe. Erweiterungen brauchen explizite Teststeuerung und unabhängige
  Vor-/Nachbedingungen; kein breiterer Reset darf Fehler verdecken. P10/P11/P12/P22
  und ihre ursprünglichen Orakel bleiben vollständig.
- **W10 / P15 / G11 — Runnerintegration:** Einzeltest, thematische Gruppe und
  Gesamtlauf führen dieselben Originalowner aus. Ein Einzeltest erhält nur seine
  nötige Vorbereitung, keine ungefragten Vorgängertests. Profile/Phasen bestimmen
  die Wiederverwendung; ein unverträglicher persistenter Kontext wird sichtbar
  abgelehnt, nicht still ersetzt. Native-/Offscreen-Ergebnisse und die
  Prozess-/Fensterzahl des tatsächlich gewählten Laufs werden getrennt belegt.
- **W10 / P18 — Performance und Beispiele:** Kontrollierte Warps, Pause,
  Inspect-/Stop-Latenzen und Verify nutzen bereits ein Game je Messaufruf.
  Native Messung und verschiedene Binaries behalten getrennte Lebensdauern.
  Eine gemeinsame App mit dem normalen Runner braucht zuerst einen passenden
  gemeinsamen Owner/Messadapter; niemals zwei Leser derselben Spiel-Pipes oder
  einen Vergleich von CLI-Gesamtlaufzeit mit bisherigen direkten Warp-Zeiten.
  Headless-Messadapter, OS-Aufruf-Negativguards und eigene Baselines aus P18
  bleiben unverändert Pflicht. Beispiele bleiben ohne Suite-Imports nutzbar.

Für die Gruppierung sind bei P15 insbesondere zusätzliche Nachweise einzuplanen:

- Gleiche PID/Session und geprüftes Profil innerhalb einer geteilten Gruppe;
  zusätzliche Starts nur mit ausgewiesenem Grund. Das gilt für alle geplanten
  Repeats einer Messung ebenso wie für funktionale Fälle.
- Zustandsabhängige Fälle nach dokumentierter Vorbereitung auch einzeln und in
  anderen zulässigen Reihenfolgen; absichtlich stehengebliebene Inputs, Pace,
  Dateien, Symlinks oder offene Arbeit müssen vor dem nächsten Fall auffallen.
- Manual-Nulltick- und Before-First-Tick-Capture vor jedem Warm-up/Reset;
  Engine-Startup mit seinem echten separaten Startvertrag. Keine Zählernullung
  als Ersatz für eine neue Lebensdauer, wo Frische Teil des Vertrags ist.
- Activity-/Request-/Dateibelege je Fall abgrenzen; gleiche Reportsignatur darf
  Created/Existing nicht zufällig von der Testreihenfolge abhängig machen.
- Dauerhafte Report-/Recording-Dateien und Nicht-Duplizierung erst nach echtem
  Owner-/Server-Join final bewerten. Ein Test mit aufgeschobener Endassertion ist
  vorher noch nicht bestanden; sein Fehlschlag wird weiterhin ihm zugeordnet.
- Unerwarteter Fehler, kontaminierter Zustand oder unbestätigter Cleanup sperrt
  weitere Nutzung, ohne Retry/Ersatzprozess. Nur die genau erwartete typisierte
  Negativkontrolle ist kein Appfehler. Exit- und Teilnachweisregeln erhalten.

### 15.4 Planstatus und nächster Entscheidungspunkt

Diese Ergänzung integriert Befunde, **keinen bereits ausgeführten Testumbau**.
Die fünf historischen Council-Runden gelten dem R05-Kern, nicht diesem neuen
Abschnitt. R05-Coverage, Review-Rohbelege und frühere Publikationschecks bleiben
historisch unverändert; eine spätere Prüfung des erweiterten PR-Umfangs muss die
zusätzlich aufgenommene Bestandsaufnahme ausdrücklich einschließen.

Als Nächstes ist die konkrete Reihenfolge innerhalb der kompatiblen Profile und
zwischen Start-/Live-/Endphasen gemeinsam festzulegen. Erst anschließend nach
passender Freigabe die Fixture-/Runneränderungen umsetzen und ihre neuen
Nachweise ausführen. CI bleibt pausiert; kein Build, App-/GPU-/Providerlauf und
keine Implementierungsfreigabe entsteht aus dieser Dokumentergänzung.
