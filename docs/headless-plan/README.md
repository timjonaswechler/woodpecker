# Headless: Status und Integrationsplan

**Zum Weiterarbeiten zuerst [STATUS.md](STATUS.md) lesen.** Dort stehen der
aktuelle Umsetzungsstand und die verbindliche Regel: **ein vollständiges
Lieferpaket abschließen, dann erst das nächste anfangen**.

| Dokument | Zweck |
| --- | --- |
| **[STATUS.md](STATUS.md)** | Vollständige Lieferpakete L1–L5 mit Abschlusskriterien, maximal ein aktives Paket; ehrlicher W00–W12-/G01–G12-Bestandsabgleich und Quellbelege. |
| [SLICE.md](SLICE.md) | Verbindlicher Umfang und Betriebsvertrag des implementierten 2D-/Basis-3D-Metal-Captureadapters; Nachweise und Grenzen. |
| [PLAN.md](PLAN.md) | Historischer Gesamtplan R05: Architektur, Paket-/Proof-IDs und ursprüngliche Abnahmeanforderungen. Keine aktuelle Erledigt/offen-Liste. |
| [COVERAGE.json](COVERAGE.json) | Unveränderter historischer R05-Owner-/Proofdatensatz. Seine `planning-only`-/`planned-not-executed`-Felder sind kein heutiger Ausführungsstand. |
| [Gemeinsame Testausführung](PLAN.md#15-ergänzung-gemeinsame-testausführung) / [Fenster-Testgruppen](../testing-window-groups.md) | Historische Originalowner und Kandidaten für Wiederverwendung. Nicht bereits headless ausgeführte oder als Verbund abgenommene Fälle. |
| [CI-Pause](../../.github/disabled-workflows/README.md) | Archivierter Workflow und gesonderte Reaktivierung; keine Repository-/Actions-Einstellungsänderung. |

## Arbeitsumfang

Die Capturebasis `5493eb1` ist um [L1](L1.md) auf
[PR #23](https://github.com/timjonaswechler/woodpecker/pull/23) erweitert: echte
Signalstation mit Image-EngineReady, Keyboard/Pointer/Text/UI und Capture auf
Metal lokal nachgewiesen. Alle zehn ursprünglichen Inputowner und die vollständige
native lokale Regression bestehen. Native bleibt Standard. Unabhängiger
Abschlussreview und finaler PR-Preflight stehen noch aus; L2–L5 bleiben offen.

Das einzige aktive Lieferpaket L1 ist der **Signalstation-Basisbetrieb auf
Metal**: Start/Assets/Image-Ready, Capture, der vereinbarte vollständige
Keyboard-/Pointer-/Text-/UIumfang und sauberer Einzel-Session-Abschluss. Alle zehn
ursprünglichen Inputowner gehören zur Headless-Abnahme dieses Pakets.

Der Ablauf **Start → Ready → realer UI-Klick → Zustand und PNG → Shutdown** bleibt
ein früher Integrationscheck, aber kein Paketabschluss. Notwendige Fehlerpfade,
Regressionen, Anleitung und Cleanup werden nicht auf später verschoben. Erst
wenn die [L1-Abschlusskriterien](STATUS.md#abschlusskriterien-von-l1--alle-erforderlich)
erfüllt sind, beginnt L2. „Teilweise“ bleibt lediglich der ehrliche Status des
historischen Bestands, kein künftiger Liefermodus. Kein Scope-Verkleinern oder
Testlockern zum Grünmachen und keine erneute Render-Machbarkeitsrunde.

Der öffentliche `pre_extract`-Hook ist im gepinnten Bevy nutzbar; der frühere
vermutete fehlende öffentliche Fehlerdrain ist kein fortbestehender Blocker.
Eine zulässige Captureantwort bestätigt einen bezeichneten abgeschlossenen Frame
und den benannten letzten beobachteten Fatalcheck, keine zukünftige Gerätegesundheit.

CI bleibt auf diesem Branch pausiert. Ein Statusdokument erteilt keine neue
App-/GPU-/Installations-/Providerfreigabe. Echter GitHub-/Providerlauf bleibt ein
separater Auftrag und kein lokaler Headlessblocker. Ein Merge, Release oder eine
CI-Reaktivierung folgt nicht automatisch aus Implementierung oder Teilnachweisen.

## Herkunft und historische Aussagen

Die technische R05-Basis ist
[`5d6ccd67321aacf69a3ee6ec9527fe7290af1ed4`](https://github.com/timjonaswechler/woodpecker/tree/5d6ccd67321aacf69a3ee6ec9527fe7290af1ed4).
Der technische Inhalt von `PLAN.md` Abschnitten 3–11 bleibt unverändert; zwei
überzählige Markdown-Codezäune wurden beim ursprünglichen Export entfernt.
Abschnitt 15 ergänzt die anschließend eingebrachten Fenster-/Gruppenbefunde und
ist nicht Teil der historischen R05-Reviewzustimmung.

Ursprüngliche Angaben wie „nur Planung“, „keine Produktimplementierung“, „alle
Gates offen“ oder damalige fehlende Ausführungsrechte beschreiben diese
Planungsphase. Sie überschreiben weder die späteren expliziten Aufträge noch die
in `SLICE.md` und `STATUS.md` benannten Implementierungen und echten Nachweise.
R05-Typnamen, neue Pfade und v2/v4-Versionsvorschläge bleiben Vorschläge; tatsächliche
heutige Interfaces sind im [Produktvertrag](../api/target.md) und Quellcode beschrieben.

`COVERAGE.json` bleibt der historische Datensatz einschließlich seiner Felder
`head`, `permissions`, `rubric` und Referenzen auf lokale Evaluator-/Pinregister.
Aktuelle Testdiscovery und Ausführung kommen aus der [Abnahmesuite](../../tests/README.md),
nicht aus rückwirkend auf grün gesetzten R05-Zeilen. Alte Fehlbelege, Quellstände
und ursprüngliche fachliche Testorakel werden nicht überschrieben.

Absolute lokale Pfade, interne Thread-/Taskkennungen und Review-Rohbelege werden
nicht mitveröffentlicht. Die ursprünglichen Rubrik-/Owner-/Evaluatorbelege bleiben
lokal; der Export ist keine eigenständig reproduzierbare historische Evaluator-
Prüfkette. Frühere Fünf-Dateien-Publikationschecks decken spätere Ergänzungen nicht
rückwirkend ab.

## Reviewstatus

**Historischer R05-Review, nicht heutige Laufzeitfreigabe:** fünf vollständige
Council-Runden mit jeweils drei unabhängig ausgeführten Architektur-/Sicherheits-/
Testreviews. Alle verwendeten GPT-6.1 Sol in Pi mit `thinking=xhigh`. Die letzten
subjektiven Bewertungen 9,2 / 9,3 / 9,1 galten dem bedingten Plan. Damals waren alle
zwölf Gates offen und noch kein Runtime-Proof ausgeführt.

Der damalige Offline-Evaluator v5 bestand seine Positivfixture und 30 negative
Fixtures. Das ist kein aktueller Produkt-/GPU-Test oder vollständiges finales
Packaging-Audit. Die heutige Evidenz steht beim jeweiligen
[Slice](SLICE.md#prüfung-und-wiederholung) und [Statusbeitrag](STATUS.md#bereits-erledigt-im-gelieferten-slice).

## Sichere Dokumentchecks

Ohne Builds, Appstart oder GPU-Ausführung:

```sh
python3 -B -m json.tool docs/headless-plan/COVERAGE.json >/dev/null
git diff --check
test -z "$(git ls-files '.github/workflows/*.yml' '.github/workflows/*.yaml')"
```

Diese Checks bestätigen Dokument-/Workflowstruktur, keine neue Runtime-Abnahme.
Bei einer weiteren Etappe `STATUS.md` fortschreiben und neue Evidenz mit ihrem
wirklichen Source-/Profilbezug verlinken; historische Dokumente nicht umetikettieren.
