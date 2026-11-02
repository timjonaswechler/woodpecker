# Plan der Woodpecker-Abnahmesuite

Der Neuaufbau ab `f9a39ed` einschließlich Bereinigung, Anwender-Beispielen und
Mutationsprüfungen (Schritte 1–7) ist abgeschlossen. Hier bleiben Abnahmestand,
verbindliche Regeln und offene PR-Aufgaben; keine erledigte Einzelarbeitsliste.

Ausführung und Inventar: [Testanleitung](../tests/README.md).
Öffentliche Abläufe: [Spieltest-Beispiele](../examples/README.md).
Fehlersensitivität: [Mutationsanleitung](../tests/mutations.md).

## Aktuelle Erweiterung: vorbereitete Programme und kleinerer Buildgraph

Auf `feat/shared-application-tests` sind rendererfreier CLI-/Host-Build,
zusammengefasste prozessisolierte Rust-Verträge, kleine echte Abort-Fixture und
wiederverwendbare Programme umgesetzt. Kein neues Compilerprofil und keine
zusätzliche Debuginfo-/Compilerflag-Optimierung gehören zu diesem Umbau.

- 76 Harness-, zehn Planungs- und 19 Spieltests sowie Workspace-Clippy mit allen
  Features/Targets und `-D warnings` bestanden.
- Die vollständige grafische lokale Abnahme bestand: **93 Produkttests plus
  typisierte Negativkontrolle**, keine fehlenden lokalen Belege oder Cleanup-Fehler.
  [Gesamtlauf](../target/test-runs/acceptance-udkhqci1/summary.json).
  Das ersetzt den früher fehlgeschlagenen Grafiknachweis für diesen Quellstand,
  ist aber kein Nachweis, dass dessen damalige Surface-Ursache behoben wurde.
- Keyboard, Pointer, Text und ein weiterer Einzeltest bestanden über vier
  Aufrufe mit derselben App **PID 23764**; explizites Stoppen wurde bestätigt.
  [Hoststatus](../target/test-runs/code-size-proof/persistent-status.json) und
  [Shutdown](../target/test-runs/code-size-proof/persistent-stop.log).
- Nach verlustfreiem Wiederherstellen des Compilercaches verwendete Cargo alle
  **1.460 Ausgaben frisch**, ohne erneute Kompilierung.
  [Frischenachweis](../target/test-runs/code-size-proof/freshness.json).
- Die zusätzliche Target-Bereinigung entfernte 30 Incremental-Caches nicht mehr
  benötigter Crates sowie die belegten alten Ausgaben des Vollspiel-Aborts,
  `mein_spiel` und der öffentlichen `test_app`-Bibliothekskopie. Quellen und Belege
  blieben erhalten. Cargo verwendete danach weiterhin alle **1.460 Ausgaben frisch**;
  vorbereitete Programme blieben bytegleich. Acht gemeinsame Eingabetests liefen
  anschließend über zwei Aufrufe mit derselben, offen gebliebenen App **PID 62453**;
  auch der kleine Abort-Fall bestand.
  [Bereinigung und Messwerte](../target/test-runs/target-cleanup-proof/result.json).
- Nach ausdrücklicher Nutzerfreigabe wurden **123 redundante alte Befehlsjournale**
  aus 17 sauber beendeten Wiederholungsläufen gelöscht. Fehlerbelege, aktueller
  Gesamtnachweis, alle Manifeste und vorbereiteten Programme blieben erhalten.
  Nach dieser Bereinigung wurden für das gesamte `target/` rund **4,061 GB** gemessen, einschließlich rund
  **394 MB verbleibender Testbelege/-fixtures**. Das **5-GB-Ruhebudget ist damit
  erfüllt**, mit bereits komprimiertem Compilercache; während Builds benötigt er
  weiterhin seinen größeren entpackten Arbeitsraum. Die ältere
  [Budgetprüfung](../target/test-runs/acceptance-_gczp95w/summary.json) bleibt als
  damaliger Fehlnachweis unverändert erhalten.
  [Löschplan und aktuelle Budgetmessung](../target/test-runs/log-cleanup-proof/result.json).
  Programmprüfsummen, Manifest und CLI-Aufruf wurden nach der Bereinigung validiert.
- Beim Vorabcheck dieser Logbereinigung war App-PID 62453 bereits beendet;
  Host und Server liefen noch. Deren Verzeichnisse und Zustand wurden geschützt,
  keine App neu gestartet oder übernommen. Bei der späteren Mergevorbereitung war
  `target/` bereits geleert; der noch laufende Diagnosehost wurde identitätsgeprüft
  beendet. Sein Serverexit 1 bleibt ein fehlgeschlagener Cleanup-Nachweis und wird
  nicht nachträglich als erfolgreich gewertet.

Ein neuer echter GitHub-Provider-Lauf wurde nicht ausgeführt. Das Ruhebudget und
die lokale Abnahme sind belegt. Der Nutzer hat anschließend auch diese Arbeit
zum Mergen freigegeben; Abschlussreview und CI werden vor dem Merge geprüft.
Auto-Merge und Merge Queue werden nicht verwendet.

## Historische Abnahme und Mergeentscheidung

[PR #16](https://github.com/timjonaswechler/woodpecker/pull/16) bündelt die Arbeiten
im Hauptrepository `woodpecker` gegen `main`. Auto-Merge und Merge Queue werden
nicht verwendet.

Der Nutzer hat die Zusammenführung nach `main` ausdrücklich beauftragt und auf
weitere CI- und Grafikläufe verzichtet. Für die beiden bereits gestarteten CI-Läufe
auf `1014aaa` wurde der Abbruch angefordert. Der dokumentierte Grafikfehler bleibt
ein fehlgeschlagener Nachweis und wird nicht als bestanden oder behoben gewertet.
Er wird auf diese Nutzerentscheidung hin nicht als Mergeblocker behandelt.
PR-Head, Basis, offene Reviews/Threads und Konfliktfreiheit werden vor dem Merge
geprüft. Branch-Regeln werden nicht umgangen.

**Umfang:** 94 Produkttests: 75 Python, zehn öffentliche und neun interne Rust-Tests.
Davon 93 lokal und einer extern. Mit Negativkontrolle 3.18 enthält das Inventar
95 Einträge; die 49 Harness-Prüfungen zählen separat.

## Frühere Abschlussprüfung auf `55c4036`

Die drei Abschlusscommits enthalten den Text-Editierbarkeitsfix, die Entfernung
ungenutzter Laufzeit-Assets und die überarbeiteten Nutzer-How-tos/API-Verträge.
Der Arbeitsbaum war bei den folgenden Läufen sauber:

- 49 Harness-, zehn Export-/Layout-, 18 Spieltests und 25 fensterlose Produkttests
  mit Feature-Matrix bestanden. Workspace, Format und Clippy bestanden ebenfalls.
  Die bestandene Teilauswahl endet weiterhin korrekt mit Exit 2.
- Das unabhängige Abschlussreview mit GPT-6.1 Sol genehmigt die drei neuen Commits
  mit Stichproben der konsolidierten Integration ohne offene Blocker oder Shoulds.
  Es ersetzt keinen vollständigen Laufzeitnachweis.
- Die grafische Gesamtabnahme **scheiterte** beim Engine-Startup des Bewegungstests:
  `startup_timeout`, `GPU meshes/images ready=true`, `window surface=false`,
  `camera=true`. Der Runner stoppte danach. Drei gezielte Diagnoseläufe blieben
  ebenfalls fehlgeschlagen. Die native Fensteraktivierung wurde im letzten Lauf
  bestätigt, ohne dass eine Surface verfügbar wurde. Es gab keine Größen- oder
  Skalierungsänderung und keine nativen Gameplay-Eingaben.

[Laufprotokoll](../target/pr16-final-proof/checks.json),
[fensterlose Abnahme](../target/test-runs/acceptance-ah32cvp2/summary.json),
[fehlgeschlagene Gesamtabnahme](../target/test-runs/acceptance-foqr8h5y/summary.json)
und [gezielte Diagnose](../target/pr16-final-proof/focused-activated-movement-result.json)
bleiben als getrennte Belege erhalten. Kein erzwungenes Harness-Cleanup wurde
als Erfolg gewertet. Die persönlichen Settings blieben unverändert.

Die früher in diesem Thread bestandenen grafischen Gesamtläufe auf `8e87b1d`
und dem danach lokal korrigierten Quellstand liegen vor der Asset-Bereinigung.
Sie ersetzen den fehlenden grünen Grafiknachweis auf `55c4036` nicht.
Die Ursache des aktuellen Surface-Problems ist noch nicht abschließend geklärt.
Tests und Render-Guards wurden nicht abgeschwächt. Ein neuer echter GitHub-Provider-
Lauf wurde nicht ausgeführt.

## Frühere Abnahmebelege

| Umfang | Geprüftes Ergebnis | Lokaler Beleg |
| --- | --- | --- |
| Lokale Gesamtabnahme 4.80 | 93 Produkttests plus Kontrolle, elf Builds; keine Skips oder erzwungenes Harness-Cleanup. | [Gesamtlauf](../target/test-runs/acceptance-wf1norm_/summary.json) |
| Echter GitHub-Provider 4.81 | `Created → Existing` derselben Referenz, vollständiger Markdown, kein Fallback; eigene Issues verifiziert geschlossen. | [GitHub-Lauf](../target/test-runs/acceptance-_29tva6l/summary.json) |
| Beispiele und Mutationen 6–7 | Drei CLI-Beispiele real bestanden; vier Fehler fachlich erkannt, null Überlebende, eine begründete Äquivalenzkontrolle. | [Abschluss](../target/test-runs/steps-6-7-ab8235by/completion.json) |
| Integration auf `91e2b3a` | 49 Harness-Prüfungen, 25 fensterlose Produkttests, Workspace-Check, Clippy und Format bestanden. | [HEAD-Prüfungen](../target/test-runs/steps-6-7-ab8235by/head-checks.json) |

Die Belege erfassen ihren jeweiligen Quellstand. Lokale Gesamtabnahme und echter
GitHub-Nachweis stammen aus getrennten Läufen; erfolgreiche Teilauswahlen sind
keine neue Gesamtabnahme und können deshalb mit Exit 2 enden. Der echte GitHub-Test
lief gegen das inzwischen auf Nutzerauftrag gelöschte dedizierte Testrepository,
nicht gegen das jetzige Hauptrepository. GitHub Actions prüft ausschließlich den
fensterlosen Umfang, keine Kamera-/GPU-Läufe oder vollständige Headless-Spielversion.

Das frühere unabhängige Gesamt-Review umfasste den damaligen PR-Diff gegen `main`,
einschließlich der 42 Vorgängercommits, nicht automatisch spätere Imports und
Korrekturen. Sein einziges Should-Finding S1 zu entfernten
Testaufrufen und veralteter App-Integration in der Anleitung ist korrigiert und
gegen die tatsächlichen Targets, Features und Links validiert; keine zusätzlichen
Produktintegrations-Blocker oder Should-Findings wurden gemeldet.
[Gesamt-Review und Finding-Abschluss](../target/test-runs/pr-16-ready-9vqdl88p/review-ledger.json)
sowie die [früheren Laufgrenzen](../target/test-runs/steps-6-7-ab8235by/completion.json)
bleiben Teil des Nachweises. Historische Qualitätsnoten werden nicht automatisch
aufgewertet; die Mutationsstichprobe beweist keine universelle Fehlersensitivität.

Artefakte unter `target/` sind lokal und nicht versioniert. Erhaltene Nachweise
umfassen Logs, Fehlversuche, Quellhashes und Cleanup-Audits. Die ausdrücklich
freigegebene Entfernung redundanter Wiederholungsjournale ist oben protokolliert;
die frühere Detailplanung liegt außerdem in der Git-Historie. Diese Dokumentbereinigung
ersetzt keine Laufbelege und wiederholt keine Produktabnahme.

## Verbindliche Testregeln

- Sollwerte am öffentlichen Vertrag prüfen, nicht aus dem Istverhalten abschreiben.
  Ein verantwortlicher Test je Verhalten; interne Rust-Tests nur für begründete,
  öffentlich nicht zuverlässig erzeugbare Risiken. Keine Dummytests.
- Starts, Vorbereitung, Inputs, Ticks, Erwartungen und Cleanup stehen sichtbar im
  Test. Hilfen verwalten Transport, Konfiguration, Prozesse und reine Abfragen,
  keine Spielaktionen. Keine versteckten Resets oder automatischen Wiederholungen,
  auch nicht nach Activity-Verlust. `EngineReady` ersetzt keine Spielbereitschaft.
- Seriell und unabhängig ausführen; mehrere isolierte Sessions pro Fall sind
  erlaubt. Nur eigene Prozesse, Dateien und Testrepositories verändern, keine
  OS-Eingaben oder persönlichen Settings. Produktmutationen nur in isolierten
  Quellkopien mit privatem Buildcache, niemals im Projekt-Checkout.
- Grafikläufe einschließlich Wiederholungen zuerst mit Starts/Fenstern, Varianten,
  Eingriffen und Tickbudgets freigeben. `WOODPECKER_TEST_RENDERED=1` ist nur eine
  technische Sperre. Desktop wach, Fenster teilweise sichtbar, Größe/Skalierung/
  Bildschirmzuordnung während der Session unverändert; kein Headless-Ersatz.
- **Bestanden** erfordert vollständige Pflichtnachweise ohne ungeklärte Outcomes
  oder Prozessreste. Unerwartete Verhaltens-, Build- oder Cleanup-Fehler sind
  **Fehlgeschlagen**; fehlende Voraussetzungen, Skips oder Abbruch **Unvollständig**.
  Erzwungenes Harness-Cleanup bleibt Fehler, auch nach erfolgreichem Aufräumen;
  absichtlich erzwungenes Produktende ist davon zu unterscheiden.
- Dauerhafte Reporteffekte nach Serverende prüfen. `Ended` und eine ruhige
  Poll-Phase beweisen keinen Providerabschluss; dafür den tatsächlichen Owner- und
  Join-Nachweis nutzen.
- Echter GitHub-Nachweis nur separat freigegeben gegen ein benanntes Testrepository
  mit Description-Marker `woodpecker-acceptance-test-repository`. Nur eigene Issues
  nach Prüfung von Body, Run-UUID, Signatur und URL schließen, nichts löschen.
  Bei ungewissem Ausgang keine Publikation wiederholen.
- Pro Lauf Versionen, Commit/Dirty-Zustand, Konfiguration sowie Build-, Test- und
  Cleanup-Belege festhalten. Derselbe Einstieg führt die zuständigen Einzeltests
  aus, keinen zweiten monolithischen Spielablauf.
- Mutationsprüfungen brauchen grüne Baselines und fachliche Assertions: Setup-/
  Buildfehler sind keine Erkennung. Überlebende, bekannte Äquivalenz und ungültige
  oder unvollständige Ausführungen getrennt ausweisen.
