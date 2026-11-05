# Signalstation L1: echte Appbilder und Nachweise

Lokale Ausführung auf **Apple M1 Pro / Metal, macOS 27.2**, Bevy 0.20.0,
wgpu 30.0.1, Rust/Cargo 1.99.1. 1440×900 physische Pixel, Scale 1,
nicht-pipelined und asynchrone Pipelinekompilierung. Die wirkliche App behält
Assets, Materialien, UI, gebündelte Fonts, MSAA4 und AcesFitted-Tonemapping.
Keine nativen Fenster, kein OS-Input und kein Systemclipboard im Headlessbuild.

## Bilder

- [a.png](a.png): reale Signalstation nach Image-EngineReady; eine zweite Aufnahme
  während Pause war bytegleich bei unveränderten Real-/Virtual-/Fixeduhren und
  Simulationstickzählern.
- [map.png](map.png): tatsächlicher UIbutton „Gesamtkarte [G]“, Overview → Map
  mit genau drei Move-/Press-/Release-Ticks. Das Bild zeigt den Pressframe nach
  dem zweiten Aktionstick; Release folgt im dritten. Widget- und Legacy-
  Interaction reagieren auf denselben Imagepointer.
- [notes-before.png](notes-before.png), [notes.png](notes.png): echtes fokussiertes
  Notizenwidget vor/nach `Grüße 🦉\nCafé – 東京`. Acht explizite Fokus-/Editticks;
  die dokumentierte Glyphenregion verändert 1163 Pixel. Latein, Akzent, CJK und
  monochromes Emoji wurden auch visuell geprüft.
- [overlay.png](overlay.png): vorhandenes GPU-Performanceoverlay auf dem Image,
  mit gezähltem Renderfortschritt. Das Bild folgt dem F8-Presstick, Release folgt
  als zweiter ausdrücklicher Aktionstick. Keine zusätzliche Bild-Settlingrunde.

Die Bilder sind RGB8-PNGs direkt aus dem Produkt-Capture-/Dateiowner, kein
CPUbild, Thumbnail, Browserscreenshot oder Proben-Publisher.

## Ausgeführte Abnahme

- **13/13 Headlessfälle:** alle zehn ursprünglichen Inputowner unverändert plus
  Basis, Wiederverwendung der ursprünglichen Startupowner (Manual, Engine und
  vier Fehlerzustände) und Geräteverlust der echten App.
- **99/99 native lokale Einträge:** 98 Produkttests plus typisierte Negativkontrolle,
  einschließlich Feature-Matrix; separat von der Headlessauswahl ausgeführt.
- 14 interne Owner-, 21 App- und 93 CPU-Harness-Prüfungen bestanden.
- Alle acht bisherigen 2D-/Basis-3D-SDK-GPUfälle samt Licht-/Schattenaudit erneut
  bestanden. Workspace- und Headless-Clippy mit `-D warnings` sowie Format bestanden.
- Der Betriebssystem-Audit lieferte für die lebende testeigene App **null native
  Fenster**. Der Dependencygraph enthielt im Headlessbuild weder Winit noch
  Arboard/X11, im CLIbuild keinen Renderer oder Windowbackend.
- CI blieb pausiert, echter GitHub-/Providerlauf wurde nicht erneut ausgeführt.

`--headless` endet trotz bestandener Auswahl mit **Exit 2**, weil er keine
Gesamtfreigabe für alle Produktfunktionen, nativen Verträge oder externe Provider
beansprucht. Die erfolgreiche native lokale Gesamtabnahme endete mit Exit 0.
Fehlgeschlagene Vorläufe bleiben unter `target/headless-l1-proof/` separat erhalten:
fehlende Konfiguration, UI-Subview-Capturetimeout, altes Window-Pointer-Unreachable,
Legacy-UI-None, fehlende Image-Framemetrik und Geräteverlust ohne Faultaktivierung.

## Prüfen und reproduzieren

[`recorded.json`](recorded.json) enthält die beobachteten Test-IDs/Ergebnisse,
Programm- und Bildhashes sowie alle 266 relevanten Runtimequellen-/Assethashes.
Absolute Rechnerpfade und interne Thread-IDs wurden nicht exportiert. Die
vollständigen Rohläufe bleiben lokal unter den dort genannten relativen Pfaden.

```sh
# Nur CPU: prüft committed Bilder/Quellen und aufgezeichnete Ergebnisse.
python3 -B tests/headless/audit.py
# Planung ohne Builds, Dateien oder Appstarts:
python3 -B -m tests.acceptance --headless --list
# Echte Wiederholung erst nach separater GPUfreigabe:
WOODPECKER_TEST_RENDERED=1 python3 -B -m tests.acceptance --headless --rendered
```

Der Auditor startet **keine neue GPUausführung**. Sein PASS bestätigt das
aufgezeichnete Paket, nicht eine neue Appausführung. Vorbereitung und weitere
Befehle: [Testanleitung](../../README.md#l1-expliziter-headless-teilumfang-mit-gpu).
Umfang und offene Folgepakete: [L1](../../../docs/headless-plan/L1.md).
Keine Software-/Linux-/beliebige-Effekte-/Kaltcache- oder Treiberfehlertoleranzfreigabe.
