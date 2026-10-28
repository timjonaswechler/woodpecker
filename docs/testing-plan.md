# Plan der Woodpecker-Abnahmesuite

Der Neuaufbau ab `f9a39ed` einschließlich Bereinigung, Anwender-Beispielen und
Mutationsprüfungen (Schritte 1–7) ist abgeschlossen. Hier bleiben Abnahmestand,
verbindliche Regeln und offene PR-Aufgaben; keine erledigte Einzelarbeitsliste.

Ausführung und Inventar: [Testanleitung](../tests/README.md).
Öffentliche Abläufe: [Spieltest-Beispiele](../examples/README.md).
Fehlersensitivität: [Mutationsanleitung](../tests/mutations.md).

## Full-stop-Abnahme und offene Checkpoints

**Noch keine Merge-Freigabe.** Offen sind:

- [ ] Genehmigte Basisbranch auf `origin` bereitstellen und prüfen. Beim letzten
  Abgleich war das Repository ohne Branches; unter `full-stop` pusht der Agent
  nicht auf den Default-Branch.
- [ ] Die 42 zur Aufnahme genehmigten Vorgängercommits unabhängig reviewen.
  Die bisherigen Reviews decken diesen erweiterten Umfang nicht ab.
- [ ] Den gemeinsamen PR veröffentlichen und dessen tatsächlichen Head, Basis,
  Checks, Reviews und Mergeability prüfen. Offen und ungemergt übergeben;
  weder Auto-Merge noch Merge Queue aktivieren.

**Umfang:** 94 Produkttests: 75 Python, zehn öffentliche und neun interne Rust-Tests.
Davon 93 lokal und einer extern. Mit Negativkontrolle 3.18 enthält das Inventar
95 Einträge; die 49 Harness-Prüfungen zählen separat.

## Abnahmebelege

| Umfang | Geprüftes Ergebnis | Lokaler Beleg |
| --- | --- | --- |
| Lokale Gesamtabnahme 4.80 | 93 Produkttests plus Kontrolle, elf Builds; keine Skips oder erzwungenes Harness-Cleanup. | [Gesamtlauf](../target/test-runs/acceptance-wf1norm_/summary.json) |
| Echter GitHub-Provider 4.81 | `Created → Existing` derselben Referenz, vollständiger Markdown, kein Fallback; eigene Issues verifiziert geschlossen. | [GitHub-Lauf](../target/test-runs/acceptance-_29tva6l/summary.json) |
| Beispiele und Mutationen 6–7 | Drei CLI-Beispiele real bestanden; vier Fehler fachlich erkannt, null Überlebende, eine begründete Äquivalenzkontrolle. | [Abschluss](../target/test-runs/steps-6-7-ab8235by/completion.json) |
| Integration auf `91e2b3a` | 49 Harness-Prüfungen, 25 fensterlose Produkttests, Workspace-Check, Clippy und Format bestanden. | [HEAD-Prüfungen](../target/test-runs/steps-6-7-ab8235by/head-checks.json) |

Die Belege erfassen ihren jeweiligen Quellstand. Lokale Gesamtabnahme und echter
GitHub-Nachweis stammen aus getrennten Läufen; erfolgreiche Teilauswahlen sind
keine neue Gesamtabnahme und können deshalb mit Exit 2 enden.

Das unabhängige Review von Schritt 6/7 hat `91e2b3a` gegenüber `08f5b07` für die
14 betroffenen Dateien mit **APPROVE** freigegeben, nicht den gesamten Branch.
[Review- und Laufgrenzen](../target/test-runs/steps-6-7-ab8235by/completion.json)
bleiben Teil des Nachweises. Historische Qualitätsnoten werden nicht automatisch
aufgewertet; die Mutationsstichprobe beweist keine universelle Fehlersensitivität.

Artefakte unter `target/` sind lokal und nicht versioniert. Vollständige Logs,
Fehlversuche, Quellhashes und Cleanup-Audits bleiben dort erhalten; die frühere
Detailplanung liegt außerdem in der Git-Historie. Diese Dokumentbereinigung
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
