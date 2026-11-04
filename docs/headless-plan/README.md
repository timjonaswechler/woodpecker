# Arbeitsrahmen für echte Headlessintegration

Dieser PR hält den Integrationsplan bereit, auf dessen Branch später implementiert
werden kann. Er enthält **noch keine Produktimplementierung**. Native bleibt der
Standard; geplant ist ein expliziter Offscreenpfad mit echtem Bevy/wgpu-Rendering,
Readback und den bestehenden Session-, Input-, Recording- und Replayverträgen.

- [Integrationsplan](PLAN.md): Architektur, Lifecycle, Orakel, Pakete W00–W12 und offene Gates.
- [Coverage](COVERAGE.json): vollständige Zuordnung der Originalowner, zusätzlichen Owner und geplanten Proofs.
- [Gemeinsame Testausführung](PLAN.md#15-ergänzung-gemeinsame-testausführung): Anschluss der Fensterbefunde an W00/W10, P15/P18 und G11.
- [Thematische Testgruppen](../testing-window-groups.md): vollständige Bestandsaufnahme mit Wiederverwendungskandidaten und ihren Voraussetzungen; noch keine Ausführungsreihenfolge.
- [CI-Pause und Reaktivierung](../../.github/disabled-workflows/README.md): unverändert archivierter Workflow; keine Repository-/Actions-Einstellungsänderung.

## Arbeitsumfang

Freigegeben ist jetzt die CI-Pause im Quellcode sowie Branch, Commits, Push und
Arbeits-PR. Die CI ist **auf diesem Branch** pausiert; `main` übernimmt die Änderung
erst durch einen gesonderten Merge. Lokale Builds, App-/GUI-/GPU-Läufe, Installationen,
Produktimplementierung und echte Provider-/GitHubtests sind damit nicht freigegeben.
Insbesondere bleibt der Realprovider-Proof P20/G12 ein separater Auftrag.
Der spätere dokumentarische Folgeauftrag ergänzt die Befunde zur gemeinsamen
App-Nutzung in der Testplanung. Er erlaubt weder den Runner-Umbau noch neue
Builds, App-/GPU-Läufe, CI-Ausführung oder Provideraktionen.

Die technische Basis ist Repository-Commit
[`5d6ccd67321aacf69a3ee6ec9527fe7290af1ed4`](https://github.com/timjonaswechler/woodpecker/tree/5d6ccd67321aacf69a3ee6ec9527fe7290af1ed4).
`PLAN.md` übernimmt den technischen Inhalt der Abschnitte 3–11 unverändert aus R05;
zwei überzählige Markdown-Codezäune wurden entfernt. Arbeitsrechte, CI-Status und
Herkunfts-/Handoffangaben wurden für die Veröffentlichung angepasst.
Der neue Abschnitt 15 verbindet die anschließend eingebrachten Fensterbefunde
mit der Testintegration; er ist nicht Teil der historischen R05-Reviewzustimmung.
`COVERAGE.json` bleibt der unveränderte historische R05-Datensatz. Seine Felder
`head`, `permissions`, `rubric` sowie Referenzen auf lokale Evaluator-/Pinregister
beschreiben die Planungsrunde, nicht den aktuellen PR-HEAD oder neue Rechte.

Absolute lokale Pfade, interne Thread-/Taskkennungen und Review-Rohbelege werden
nicht mitveröffentlicht. Die historischen Rubrik-, Baseline-, Owner-Minimum- und
Evaluatorbelege bleiben lokal; dieser PR enthält deshalb keine eigenständig
reproduzierbare historische Evaluator-Prüfkette. Die zuvor nur lokal gelesene
`docs/testing-window-groups.md` wird auf den dokumentarischen Folgeauftrag hin
inhaltlich unverändert als zugehörige Bestandsaufnahme aufgenommen. Die frühere
Fünf-Dateien-Publikationsprüfung beschreibt nur den ursprünglichen Export;
sie ist kein Nachweis dieser Ergänzung und wird nicht rückwirkend umgeschrieben.

## Reviewstatus

Die Planung durchlief fünf vollständige Council-Runden mit jeweils drei unabhängig
ausgeführten Reviews für Architektur, Sicherheit und Tests. Alle verwendeten
GPT-6.1 Sol in Pi mit `thinking=xhigh`. Die abschließenden subjektiven Bewertungen
waren 9,2 / 9,3 / 9,1 mit Zustimmung **zum bedingten Plan**, nicht zur Laufzeitfähigkeit.
Alle zwölf Gates bleiben offen; kein Runtime-Proof wurde ausgeführt.

Der historische Offline-Evaluator v5 bestand seine Positivfixture und 30 negative
Fixtures. Das ist weder ein vollständiges finales Packaging-Audit noch ein
Produkt-/GPU-Test. Die für die Veröffentlichung neu geprüften Eigenschaften sind
CI-Pause, unveränderte Workflowdaten, Dokumentlinks, JSON-Struktur, Referenzen und
unveränderter technischer R05-Inhalt.

## Nächster Implementierungsschritt

Nach passender Freigabe mit W00 beginnen: neue Quell-/Testbaseline und additive
Verträge festlegen. Früh den öffentlichen GPU-Fehlerbeobachtungs-/Publisher-Hook
für G02/G03/G04/G09 beweisen. Ein Queuecallback allein ist kein GPU-Erfolgsnachweis.
Erst danach den echten W01–W05-Render-/Readbackslice freigeben; die späteren
Vollapp-/UI-/Input-/Recording-/Replay-/Zoo-/Plattformnachweise bleiben Pflicht.

## Sichere Dokumentchecks

Ohne Builds, Appstart oder GPU-Ausführung:

```sh
python3 -B -m json.tool docs/headless-plan/COVERAGE.json >/dev/null
git diff --check upstream/main...HEAD
test -z "$(git ls-files '.github/workflows/*.yml' '.github/workflows/*.yaml')"
```

Diese Checks belegen ausschließlich Dokument-/Workflowstruktur, keine
Implementierungsreife und keine bestandene Gesamtabnahme.
