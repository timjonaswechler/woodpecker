# Aufgezeichnete lokale Evidenz

[recorded.json](recorded.json) ist ein bereinigter Export tatsächlich ausgeführter
Läufe vom2026-11-04, Produktbasis `100cc68395f0f5b6c58e566734d7cef6f060db87`.
Persönliche Maschinenpfade, PIDs, interne Thread-/Handoffkennungen und rohe
Stacks sind nicht enthalten. Ereignisordnung, Fehlertexte, Probenidentitäten,
Quell-/Bild-/Binaryhashes und ursprüngliche Resultate bleiben nachvollziehbar.
`artifacts/` und `boundary-artifacts/` im JSON sind logische Pfadaliase, keine
mitgelieferten Verzeichnisse. Volle Rohlogs/Binaries/Quellsnapshots bleiben lokal.

## Drei getrennte Serien

| Serie | Tatsächliches Resultat |
|---|---|
| Erster Renderproof |9 Prozesse:5 Exit0,4 Exit101. Früher fehlender Zeitkanal-Drain und Fehler vor RenderStartup wurden nicht als Erfolg ausgegeben. A/B und Pause danach echt gerendert. |
| Startup-/Runtimegrenze |16 Prozesse:14 Exit0,1 reproduzierter roter Exit101,1 Instrumentierungs-Timeout. Öffentliches `pre_extract` über `get_added_plugins` tatsächlich nutzbar; Fehlerbeobachtung bei Pause ohne Tick2. Timeout war stdout-Lockkonflikt der Probe, durch stderr korrigiert. |
| Ausgabekette/Publisher |15 Prozesse, alle Exit0 mit ihren Positiv-/Negativorakeln; keine Retries. Gesunder Wrapper, Pipeline/Draw/Copy/Mapfehler, Cancel/Timeout/stale und finaler Publisher geprüft.52 vollständige PNGs unabhängig dekodiert. |

Die aktuelle Empfehlung ist deshalb **kleine Integration angehen**, nicht
„öffentlicher Fehlerhook fehlt“. Keine der Serien ist ein universeller
Headless-/Produkt-PASS. Direkte wgpu-Validation ist echte API-/Corevalidation,
kein behaupteter Hardwarefehler. Cancel/Frist-/Partial-write-Faults sind CPU-
Probenzustände; eigenes Device::destroy ist logischer DeviceLost, kein Treiberreset.

## Enthaltene Originalbytes

- [a.png](a.png): rotes Mesh links, Tick1/Frame2/Request101.
- [b.png](b.png): nach explizitem Tick2 grünes Mesh rechts, Frame3/Request102.
- [b-paused.png](b-paused.png): Frame4/Request103, weiterhin Tick2; bytegleich mit B.
- [early-black.png](early-black.png): erster echter Readback, für diese Szene vom
  Geometrieoracle abgelehnt, trotz Completioncallback.
- [published-frame.png](published-frame.png) und
  [published-receipt.json](published-receipt.json): jüngster Publisherfall07,
  Request800/Generation1/Frame3/Tick1. Eigene echte GPU-Pixel, keine Sollbilder.
- [partial-staging-prefix.bin](partial-staging-prefix.bin): absichtlich nur8
  tatsächlich geschriebene PNG-Signaturbytes aus einem nichtfinalen Stagingpfad.
  Die Erweiterung `.bin` verhindert die Verwechslung mit einer vollständigen PNG.

Datei- und Pixelfingerprints stehen im JSON. Zwischen A und B ändern sich exakt640
Pixel; asymmetrische Eckmarker prüfen Orientierung/Kanäle. Target65×49,
Rgba8UnormSrgb, Nutzzeile260B, Bufferstride512B; exportiert als opakes RGBA8-PNG,
nicht als bereits abgenommener Produkt-RGB-Pfad.

Die mitgelieferten Quellen rekonstruieren die aufgezeichneten Ausführungsbytes:
Nur eine zusätzliche abschließende Leerzeile in `study_base.rs` wurde entfernt;
Original- und Publikationshash sowie diese Normalisierung stehen im JSON. Die
Lockfiles sind bytegleich. Der portable Pythonrunner/CPUAuditor ist eine
nachträgliche Publikationsverpackung und kein behaupteter neuer GPUlauf.
`python3 -B tests/prototypes/headless-render-lab/audit.py` prüft die Paketkonsistenz,
nicht die Gerätegesundheit eines zukünftigen Laufs.
