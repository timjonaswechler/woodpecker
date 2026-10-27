# Qualität aller bisher erstellten Tests – drei unabhängige Reviews

**Historischer Reviewstand, keine Neubewertung der inzwischen geänderten Tests.**
Die [anschließende Qualitätsnachschärfung](testing-plan.md#qualitätsnachschärfung-nach-dem-drei-reviewer-stand)
ist eingearbeitet und separat verifiziert. Die folgenden 279 Noten und
Originalbefunde bleiben unverändert; neue Assertions erhalten nicht automatisch
zehn Punkte. Zeilenbelege beziehen sich auf den damaligen Snapshot.
Der neue interne Nachweis 4.82 gehört nicht zum damaligen 93er-Reviewumfang.

Bewertet sind alle **93 Produkttests**, deren äußerer Code-Haken im
[Testplan](testing-plan.md) gesetzt ist: **75 Python-Fälle, zehn öffentliche
Rust-Integrationstests und acht interne Rust-Nachweise**.
Stand: `f9a39ed` plus der bei Reviewbeginn festgehaltene lokale Arbeitsstand.

Drei Subagents haben jeden Test unabhängig auf **0–10** bewertet.
**279 Einzelnoten; Durchschnitt 9,8/10.**
Die Noten liegen tatsächlich nur zwischen **8 und 10**. Die Reviewer urteilen
also sehr positiv; die Skala ist am oberen Ende stark gesättigt und liefert
keine fein kalibrierte Aussage über die gesamte mögliche Testqualität.

**72 von 93 Fällen** haben identische Einzelnoten, davon **67 dreimal 10**.
Der Median beträgt bei **78 Fällen 10**, bei **15 Fällen 9**.
Die größte Spannweite ist **8–10**, bei 3.17 und 4.53.
Es wurde kein aktueller zentraler Vertragswiderspruch und kein Cap-Fall gemeldet.
Das ist **keine Aussage über bestandene Läufe oder Produktfehlerfreiheit**.

## Vorgehen und Skala

Alle drei Reviewer bekamen denselben Scope und dieselbe Rubrik, aber getrennte
frische Kontexte und unterschiedliche Prüfschwerpunkte. Sie durften weder diese
frühere Auswertung noch die Ergebnisse der anderen Reviewer lesen.
Verwendet wurde jeweils `openai-codex/gpt-6.1-sol` mit `thinking=xhigh`.
Das sind drei getrennte Reviews desselben Modells, keine Modellvielfalt oder
statistisch unabhängigen Messungen.

| Reviewer | Schwerpunkt | Eigener Mittelwert |
| --- | --- | ---: |
| **R1** | Vertragskorrektheit und unabhängige Sollwerte | 9,8 |
| **R2** | Fehlererkennung und plausible überlebende Regressionen | 9,9 |
| **R3** | Determinismus, Isolation, Lifecycle und Wartbarkeit | 9,8 |

Jeder Reviewer bewertete trotzdem **alle fünf Dimensionen für jeden Test**:

| Dimension | Punkte |
| --- | ---: |
| Vertragskorrekte, unabhängige Sollwerte | 0–3 |
| Fachliche Wirkung und Regressionserkennung im eigenen Scope | 0–3 |
| Determinismus und kontrollierbare Voraussetzungen | 0–2 |
| Isolation und ehrlicher Lifecycle/Cleanup | 0–1 |
| Diagnose und nachvollziehbarer Ablauf | 0–1 |
| **Summe** | **0–10** |

Bei einem belegten falschen zentralen Oracle wäre die Gesamtnote auf 3 begrenzt,
bei fehlendem zentralem Wirkungsnachweis auf 4. Kein Reviewer setzte ein Cap.
Orientierung: 0–3 nicht verlässlich, 4–5 wesentliche Nachweislücken,
6–7 nützlich mit Einschränkungen, 8–9 stark, 10 außergewöhnlich vollständig
im jeweils zugesagten Vertrag. Enge administrative Tests dürfen hoch bewertet
werden; fehlende aktuelle Grafikläufe wurden nicht als Codefehler abgezogen.

**Aggregation:** Ø ist der ungewichtete Mittelwert der drei Originalnoten,
auf eine Nachkommastelle gerundet. Median ist die mittlere der drei Noten;
Spanne ist Minimum–Maximum, nicht ein Konfidenzintervall.
Die Gesamtzahl oben mittelt alle 279 Einzelnoten ohne Featuregewichtung.
Es gab keine Umrechnung alter A/B/C-Noten und keine nachträgliche Notenkurve.
Die Synthese erläutert Minderheitenhinweise, verändert aber keine Einzelnoten.

## Zusammenfassung der verbleibenden Grenzen

Die folgenden Punkte wurden durch Quellenabgleich eingeordnet. Sie belegen
Testlücken oder mögliche Harness-/Umweltrisiken, **keine beobachteten
Produktfehler oder gemessenen Flake-Raten**. Ein hoher Median widerlegt einen
konkreten Minderheitenhinweis nicht.

| Thema / Fälle | Stimmen | Einordnung und Quelle |
| --- | --- | --- |
| **Reportabschluss ist nicht Session-Ende**<br>3.16, 3.17, 4.47, 4.48, 4.49, 4.50, 4.51, 4.53 | R1/R3, zusätzlich R2 bei 3.16 | Nach terminalem Lifecycle darf noch Report-Activity eintreffen. Mengenprüfungen davor schließen späte zusätzliche Outcomes nicht vollständig aus. Das ist eine Nachweislücke, kein beobachtetes Duplikat.<br>[`docs/api/target.md:181`](../docs/api/target.md#L181), [`src/server/entry.rs:209`](../src/server/entry.rs#L209), [`tests/reports/tracing_report.py:123`](../tests/reports/tracing_report.py#L123) |
| **Ephemeral-Handle braucht unterscheidbare Identität**<br>4.19 | alle drei | Ein einzelnes Handle belegt Format, Stabilität und Typed/Untyped-Konsistenz. Ein überall konstantes gültiges Token ist dadurch nicht ausgeschlossen; der Vertrag verlangt tatsächliche Indexbits.<br>[`tests/inspect/reflection_values.py:60`](../tests/inspect/reflection_values.py#L60), [`src/session/inspect/value.rs:92`](../src/session/inspect/value.rs#L92), [`docs/api/target.md:514`](../docs/api/target.md#L514) |
| **Report-Metadaten und Markdown nur teilweise als Werte geprüft**<br>4.56, 4.57 | R1/R2 für 4.56; alle drei für 4.57 | Snapshot-Immutabilität, Titel und Signaturen sind stark. Falsche Kontextwerte oder gekürzte Markdown-Nutzdaten könnten einzelne Assertions trotzdem erfüllen.<br>[`tests/rust/test_report_snapshot.rs:112`](../tests/rust/test_report_snapshot.rs#L112), [`tests/rust/test_report_signature.rs:90`](../tests/rust/test_report_signature.rs#L90) |
| **Zeitfenster sind zusätzliche Umweltvoraussetzungen**<br>4.5, 4.6, 4.10, 4.63, 4.66 | mehrere Reviewer; fallbezogen unterschiedlich | Kurze Fortschritts-/Reaktionsfenster können Hostlast mit Produktfehlern verwechseln. 4.62 wird überwiegend positiv bewertet, weil es überlappende Fristenhypothesen als unvollständig behandelt.<br>[`tests/tick/set_pace.py:116`](../tests/tick/set_pace.py#L116), [`tests/session/disconnect.py:74`](../tests/session/disconnect.py#L74), [`tests/shutdown/deadline.py:100`](../tests/shutdown/deadline.py#L100) |
| **Restprozesse werden nicht überall unabhängig ausgeschlossen**<br>3.2, 3.3, 3.17, 4.58, 4.59 | Minderheitenhinweis R3 | Verwaltungszustand und Serverexit ersetzen nicht die Abwesenheit einer getrennten Spielprozessgruppe. Im Quellenabgleich nachvollziehbar; kein Prozessleck beobachtet. Keine pauschale Abwertung aller Tests.<br>[`tests/_support.py:59`](../tests/_support.py#L59), [`src/session/process.rs:30`](../src/session/process.rs#L30) |
| **Zwei kleine konkrete Gegenbeispiele**<br>4.53, 4.72 | Minderheitenhinweise R3 | entered.json kann vor vollständigem Schreiben sichtbar sein. Im positiven Text-Fokusfall wird Ziel A nach dem zweiten Editpass nicht erneut geprüft. Beide Hinweise bleiben trotz hoher Medianwerte sichtbar.<br>[`tests/faults/gh.py:26`](../tests/faults/gh.py#L26), [`tests/reports/slow_provider.py:53`](../tests/reports/slow_provider.py#L53), [`src/session/input/text/tests/test_target_binding.rs:92`](../src/session/input/text/tests/test_target_binding.rs#L92) |

Die niedrigste gemittelte Bewertung erhält **4.6 mit 8,7/10**
(9/8/9). Die stärksten Unterschiede betreffen **3.17** (9/10/8) und
**4.53** (10/10/8). Dort gelten die Risiken aus der Tabelle weiterhin,
auch wenn ein Reviewer beziehungsweise die Mehrheit 10 vergibt.

Frühere Beanstandungen wurden nicht ungeprüft übernommen: Im aktuellen Code
prüfen 4.44/4.45 die korrekte `unknown_type_path`-Ablehnung und 4.81 die
verschachtelte Issue-Referenz. Auch die vollständige Aktionstick-Zählung in
3.12 ist jetzt vorhanden; R2s verbleibender Hinweis betrifft **partielle
Bewegung pro Hold-Tick**, nicht mehr die Tickanzahl. 3.13 wird von allen drei
Reviewern als legitimer fester Szenen-/Bild-Fixture-Vertrag bewertet, nicht
pauschal wegen fester Pixelregionen abgewertet.

## Laufstatus ist eine getrennte Aussage

**43** Fälle haben laut aktuellem Plan einen Erfolgshaken.
**Acht** haben nur einen älteren grafischen Laufbeleg; **42** sind ohne
vollständigen erfolgreichen Lauf des geplanten Falls.

- **Plan-Haken:** dokumentierte Erfolgsmarkierung übernommen, nicht neu verifiziert.
- **Altstand:** 3.3, 3.12, 3.13, 3.16, 4.6, 4.10, 4.13 und 4.16 sind inzwischen geändert;
  ihre älteren grafischen Erfolge bestätigen nicht die aktuelle Fassung.
- **Offen:** Code erstellt, vollständiger vorgesehener Lauf nicht erfolgreich abgehakt.
  Externe/F0-Teilkontrollen sind kein ersetzender Spielnachweis.

Laut [Testanleitung](../tests/README.md#umfang) sind frühere Artefakte teilweise
nach Verlust von `target/` nicht mehr verfügbar. Die dort dokumentierten
fensterlosen Regressionen und Härtungskontrollen wurden für dieses Review nicht
wiederholt. Besonders 4.81 besitzt hier **keinen echten GitHub-Laufbeleg**.

## Vollständige Bewertung je Test

Alle 93 Code-IDs stehen genau einmal in den folgenden Tabellen.
Die Einzelnoten bleiben unverändert sichtbar; eine alleinige 10 im Median
darf nicht als Einstimmigkeit gelesen werden. In der Synthese sind abweichende
Hinweise mit R1/R2/R3 bezeichnet. Die fünf Einzelkomponenten und Begründungen
je Reviewer sind zusätzlich in der interaktiven Darstellung einsehbar.

### Pilotfälle

| Test / Quelle | R1 | R2 | R3 | Ø / 10 | Median | Spanne | Lauf | Synthese |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| **3.1 · Server-Lebenszyklus**<br>[`tests/session/server_lifecycle.py`](../tests/session/server_lifecycle.py#L8) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Prüft echte Loopback-Bereitschaft, leeres Verzeichnis, Stop-Antwort und Exit 0. |
| **3.2 · Session-Lebenszyklus**<br>[`tests/session/session_lifecycle.py`](../tests/session/session_lifecycle.py#L10) | 10 | 10 | 9 | **9,7** | 10 | 9–10 | Plan-Haken | R3: Ended/Serverexit geprüft, App-PID und -Gruppe danach nicht unabhängig als verschwunden nachgewiesen. |
| **3.3 · Unabhängige Session-Lebensdauer**<br>[`tests/session/session_isolation.py`](../tests/session/session_isolation.py#L10) | 10 | 10 | 9 | **9,7** | 10 | 9–10 | Altstand | B wird durch einen frischen Spiel-Command geprüft. R3 vermisst den unabhängigen PID-/Gruppenabschluss von A. |
| **3.4 · Unbekannte Session**<br>[`tests/session/unknown_session.py`](../tests/session/unknown_session.py#L9) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Syntaktisch gültige unbekannte ID, Exit 1, Fehlercode und weitere Servernutzbarkeit. |
| **3.5 · Ungültige Session-ID**<br>[`tests/session/invalid_session_id.py`](../tests/session/invalid_session_id.py#L9) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Zu kurze, zu lange und nichthexadezimale IDs; anschließende echte Verwaltungsabfrage. |
| **3.6 · Ungültige Startkonfiguration**<br>[`tests/session/invalid_launch_config.py`](../tests/session/invalid_launch_config.py#L9) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Gültiges TOML mit leerem Package wird vor einem Session-Eintrag abgelehnt. |
| **3.7 · Fehlgeschlagener Start**<br>[`tests/session/launch_failure.py`](../tests/session/launch_failure.py#L9) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Fehlendes Manifest wird zunächst angenommen; stabiler Failed-Eintrag und Server-Exit 1. |
| **3.8 · Erster Spiel-Command und Resource-Wert**<br>[`tests/inspect/resource_value.py`](../tests/inspect/resource_value.py#L18) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Vollständige Settings aus unabhängigem Seed, exakte Wire-Form und Request-Zuordnung. |
| **3.9 · Manual-Startup ohne Fortschritt**<br>[`tests/startup/manual_startup.py`](../tests/startup/manual_startup.py#L10) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Unabhängiger Last-Zähler und Bevy-Zeit beginnen null und bleiben über reale Pause gleich. |
| **3.10 · Exakte Ticks und Fixed-Clock**<br>[`tests/tick/exact_ticks.py`](../tests/tick/exact_ticks.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Drei/sieben Ticks mit aus Fixture-Dauer vorgerechneten Zeitdifferenzen und Stillstand. |
| **3.11 · Engine-Startup mit gezählten Ticks**<br>[`tests/startup/engine_startup.py`](../tests/startup/engine_startup.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | SessionReady vor EngineReady, genau ein Event, passender App-Zähler und Idle-Stillstand. |
| **3.12 · Tastatur mit tatsächlicher Bewegung**<br>[`tests/keyboard/movement.py`](../tests/keyboard/movement.py#L10) | 10 | 9 | 10 | **9,7** | 10 | 9–10 | Altstand | Tickzahlen sind jetzt vollständig geprüft. R2: Die zusammengefasste Hold-Bewegung könnte partielle Bewegungsausfälle verdecken. |
| **3.13 · Screenshot ohne Simulationsfortschritt**<br>[`tests/screenshot/capture.py`](../tests/screenshot/capture.py#L20) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Altstand | Echtes PNG, drei explizite Bildregionen sowie unveränderte Ticks, Zeit und Player. |
| **3.14 · Recording und Replay in frischer Session**<br>[`tests/replay/recording_roundtrip.py`](../tests/replay/recording_roundtrip.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Frische Zielsitzung, drei reale Replay-Checkpoints, Nicht-No-op und fünf echte Ticks. |
| **3.16 · Nichtfataler Fehler mit lokalem Report**<br>[`tests/reports/tracing_report.py`](../tests/reports/tracing_report.py#L12) | 9 | 9 | 9 | **9,0** | 9 | 9–9 | Altstand | Alle drei: Reportzählung endet bei Ended, obwohl Provider-Outcomes danach noch eintreffen dürfen. |
| **3.17 · Panic, Report und Prozessende**<br>[`tests/reports/panic_exit.py`](../tests/reports/panic_exit.py#L12) | 9 | 10 | 8 | **9,0** | 9 | 8–10 | Plan-Haken | R1/R3: mögliche Report-Nachläufer; R3 zusätzlich fehlender unabhängiger App-/Gruppenabschluss. Größere Streuung 8–10. |

### Öffentliche Feature- und Grenzfalltests

| Test / Quelle | R1 | R2 | R3 | Ø / 10 | Median | Spanne | Lauf | Synthese |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| **4.2 · Startup-Grenzen und fehlende Voraussetzungen**<br>[`tests/startup/startup_failures.py`](../tests/startup/startup_failures.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Vier getrennte Startup-Ursachen, passende Codes/Budgets und ausdrücklich kein EngineReady. |
| **4.3 · Ungültige Startup-/Tick-Konfiguration**<br>[`tests/startup/invalid_config.py`](../tests/startup/invalid_config.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | 15 intrinsisch ungültige Konfigurationen, leere Sessions und Cargo-Tripwire. |
| **4.4 · Application-Clock beibehalten**<br>[`tests/tick/application_clock.py`](../tests/tick/application_clock.py#L10) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | App-eigene 50-ms-Clock mit steady/double/paused und unabhängigen Zeit-/Policy-Sollwerten. |
| **4.5 · Laufenden Warp stoppen**<br>[`tests/tick/stop_warp.py`](../tests/tick/stop_warp.py#L11) | 10 | 9 | 9 | **9,3** | 9 | 9–10 | Plan-Haken | R2/R3: Der schnelle 50.000er-Warp kann vor dem CLI-Stop natürlich fertig werden; aktive Messlage nicht garantiert. |
| **4.6 · Pace ändern, nicht Simulationszeit**<br>[`tests/tick/set_pace.py`](../tests/tick/set_pace.py#L12) | 9 | 8 | 9 | **8,7** | 9 | 8–9 | Altstand | Alle drei sehen Hostlastabhängigkeit nach Kalibrierung. R2 zusätzlich: breite Ratefenster lassen moderate Fehlskalierung bestehen. |
| **4.7 · Warp-Zustands- und Wertfehler**<br>[`tests/tick/warp_errors.py`](../tests/tick/warp_errors.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Ungültige Werte im Idle und Konflikte aktiv getrennt; bestehender Warp arbeitet danach weiter. |
| **4.8 · Session-Auswahl und Bindung**<br>[`tests/session/session_binding.py`](../tests/session/session_binding.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Voll-ID/Präfix über HTTP und gebundenes WebSocket, Ended-Eintrag mit stabilen Metadaten. |
| **4.9 · Request-Zuordnung über Clients und Sessions**<br>[`tests/session/request_correlation.py`](../tests/session/request_correlation.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Gleiche IDs verschiedener Sessions, verschiedene Outputs und ungeordnete Abschlüsse. |
| **4.10 · Angenommene Arbeit nach Disconnect**<br>[`tests/session/disconnect.py`](../tests/session/disconnect.py#L12) | 9 | 9 | 9 | **9,0** | 9 | 9–9 | Altstand | Clientfreier Fortschritt ist jetzt per App-Datei belegt. Alle drei: zwingender Fortschritt in 0,75 s bleibt lastabhängig. |
| **4.11 · Getrennte virtuelle Eingaben und Spielzustände**<br>[`tests/session/input_isolation.py`](../tests/session/input_isolation.py#L39) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | B tickt zuerst bei A-Queue; Keyboard, Maus, Pointer und Player beider Worlds verglichen. |
| **4.12 · Getrennte Settings und Dateien**<br>[`tests/session/file_isolation.py`](../tests/session/file_isolation.py#L20) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Getrennte Dateien/Inodes, echte UI-Änderung nur in A, Persistenz und unveränderte B-/Seed-Werte. |
| **4.13 · Activity-Lücke und Wiederaufnahme**<br>[`tests/session/activity_gap.py`](../tests/session/activity_gap.py#L29) | 9 | 10 | 10 | **9,7** | 10 | 9–10 | Altstand | Eviction/Snapshot/Resume präzise geprüft. R1: maximal 64 Queries in 20 s setzt ausreichend schnellen Beobachter voraus. |
| **4.14 · Nur lokaler Zugriff**<br>[`tests/session/local_access.py`](../tests/session/local_access.py#L14) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Wildcard-Bindung abgelehnt; drei Origins auf Lesen/Stop geprüft, lokale CLI bleibt nutzbar. |
| **4.15 · Handshake und beschädigte Spielantworten**<br>[`tests/session/protocol_faults.py`](../tests/session/protocol_faults.py#L42) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Zehn literale externe Störungen; korrelierte/unkorrelierte Fehler und weitere Nutzbarkeit getrennt. |
| **4.16 · Resource-Metadaten und fehlende Typen**<br>[`tests/inspect/resource_metadata.py`](../tests/inspect/resource_metadata.py#L16) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Altstand | Unabhängige vollständige Fixture-Mengen für All, exakte Metadata-Wireform, Missing/Unknown getrennt. |
| **4.17 · Entity-Filter und Component-Projektionen**<br>[`tests/inspect/entity_query.py`](../tests/inspect/entity_query.py#L17) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Bekannte Vier-Component-Entity, positive/negative Filter, sortiertes All und explizites Listed. |
| **4.18 · Hierarchie und veraltete Handles**<br>[`tests/inspect/hierarchy_handles.py`](../tests/inspect/hierarchy_handles.py#L26) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Unabhängig konstruierte Bäume, absichtlich umgekehrte Kindreihenfolge und echte Entity-Ersetzung. |
| **4.19 · Reflection-Werte und Unavailable-Diagnosen**<br>[`tests/inspect/reflection_values.py`](../tests/inspect/reflection_values.py#L22) | 9 | 9 | 9 | **9,0** | 9 | 9–9 | Plan-Haken | Alle drei: Ein konstantes 16-Hex-Token könnte den einzigen Ephemeral-Handle trotz falscher Identität bestehen lassen. |
| **4.20 · Einmalige Aktion trotz Hold**<br>[`tests/keyboard/single_action.py`](../tests/keyboard/single_action.py#L10) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Jeder gehaltene Tick einzeln, echte Map-Aktion, ButtonInput-Flanken und erneute Press. |
| **4.21 · Ungültige Tastenwechsel**<br>[`tests/keyboard/invalid_transitions.py`](../tests/keyboard/invalid_transitions.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Neun Ablehnungen bei vorgemerkter und angewandter Taste; korrekte Hold-/Release-/Idle-Flanken. |
| **4.22 · Absolute Position und echtes UI-Picking**<br>[`tests/pointer/ui_click.py`](../tests/pointer/ui_click.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Echter Button über Text/Layout, Vorab-No-op, Pressed-Component und Map-Wirkung. |
| **4.23 · Relative Bewegung und FPV**<br>[`tests/pointer/relative_motion.py`](../tests/pointer/relative_motion.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Vorgegebene Deltas/Yaw/Pitch, Akkumulation, Null, beide Ränder und MoveTo ohne MouseMotion. |
| **4.24 · Button-Zustände**<br>[`tests/pointer/buttons.py`](../tests/pointer/buttons.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Jede Button-Press/Release getrennt gegen Picking und Bevy-State inklusive Flanken. |
| **4.25 · Scroll-Deltas**<br>[`tests/pointer/scroll.py`](../tests/pointer/scroll.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Explizite Summen und Line-Einheiten, Vorab-No-op, Null/Idle-Löschung und konstante Buttons. |
| **4.26 · Ungültige Pointer-Eingaben**<br>[`tests/pointer/invalid_input.py`](../tests/pointer/invalid_input.py#L12) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | 19 präzise Rejections; akzeptierte Queue und angewandter Pointer-State überleben Fehler. |
| **4.27 · Unicode in echtem Eingabefeld**<br>[`tests/text/unicode_input.py`](../tests/text/unicode_input.py#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Pointerfokus in echtem Notizfeld, exakter komplexer Unicode-Text, keine Keyboard-Synthese. |
| **4.28 · Fokus und UTF-8-Bytegrenze**<br>[`tests/text/input_limits.py`](../tests/text/input_limits.py#L12) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Echte Übernahme von 16384 UTF-8-Bytes, 16385 abgelehnt; Fokus und Widgetlimit getrennt. |
| **4.29 · Capture vor erstem Tick**<br>[`tests/screenshot/before_first_tick.py`](../tests/screenshot/before_first_tick.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Vor erstem Tick korrekte sofortige Ablehnung, vollständiger Sentinel-/Zeit-/Player-Schutz. |
| **4.30 · Wiederholung und Overwrite**<br>[`tests/screenshot/overwrite.py`](../tests/screenshot/overwrite.py#L14) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Zwei vollständig dekodierte PNGs; Zwischen-Sentinel erzwingt tatsächliches zweites Ersetzen. |
| **4.31 · Screenshot-Pfadsicherheit**<br>[`tests/screenshot/invalid_path.py`](../tests/screenshot/invalid_path.py#L15) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Zwölf Pfadfehler mit eigenen In-/Outside-Sentinels, Link-/Inventar- und Stillstandskontrollen. |
| **4.32 · Dateivertrag und Reihenfolge**<br>[`tests/recording/file_contract.py`](../tests/recording/file_contract.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Exakter v2-Header/Footer, zehn Einträge in Annahmereihenfolge und typgerechte vollständige Outcomes. |
| **4.33 · Recording-Zustände und Barrieren**<br>[`tests/recording/barriers.py`](../tests/recording/barriers.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Zustands-/Pending-Fehler, exakte innere Commands, Header-/Footer-Barrieren und Recovery. |
| **4.34 · Recording-I/O und unerwartetes Ende**<br>[`tests/recording/io_failure.py`](../tests/recording/io_failure.py#L17) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Feste Schreiblast mit echten Rejections, lokales Dateilimit, Recovery und beide End-Footerpfade. |
| **4.35 · Ungültige Datei vollständig vorvalidieren**<br>[`tests/replay/invalid_recording.py`](../tests/replay/invalid_recording.py#L23) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | 33 Datei-/Pfad-/Clockfehler vor Präfixwirkung, unveränderter World und frischer Queue. |
| **4.36 · Replay stoppen**<br>[`tests/replay/stop.py`](../tests/replay/stop.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Echter Präfixfortschritt, kontrollierter Stop, Recording ohne Tail und erhaltene Press-Wirkung. |
| **4.37 · Replay-Exklusivität**<br>[`tests/replay/exclusivity.py`](../tests/replay/exclusivity.py#L14) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Neun externe Konflikte, tatsächlicher aktiver Replay-Warp und nur interne Commands in Recording. |
| **4.38 · Technische Blockierung des Plans**<br>[`tests/replay/blocked.py`](../tests/replay/blocked.py#L14) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Neue Protokollstörung blockiert ohne Tail; historische Fehl-Outcomes werden nicht zu Sollwerten. |
| **4.39 · TOML-/Dateifehler**<br>[`tests/cli/config_errors.py`](../tests/cli/config_errors.py#L12) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Acht Datei-/TOML-/Versionsfehler, leeres Verzeichnis und kindlokaler Cargo-Tripwire. |
| **4.40 · Command-JSON vor dem Senden**<br>[`tests/cli/command_errors.py`](../tests/cli/command_errors.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | 16 JSON-/UTF-8-/Strukturfehler einschließlich echter argv-Bytes; eigener Listener erkennt Sendeversuch. |
| **4.41 · Script vollständig vorvalidieren**<br>[`tests/cli/script_validation.py`](../tests/cli/script_validation.py#L20) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Neun fehlerhafte Gesamtscripts hinter gültigem Präfix; Index, leere Activity und World unverändert. |
| **4.42 · Script-Pipeline, Barrieren und Indizes**<br>[`tests/cli/script_pipeline.py`](../tests/cli/script_pipeline.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Passed und gemischtes Script, volle Indizes/Outputs, echte ungeordnete Antworten und Activity-Barrieren. |
| **4.43 · Ungewisse Script-Ergebnisse**<br>[`tests/cli/script_unknown.py`](../tests/cli/script_unknown.py#L30) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Fünf unabhängige äußere Wire-Störungen; known/unknown/not_submitted und keine Recovery-RPCs. |
| **4.44 · Eingabeformen und Parsefehler**<br>[`tests/repl/commands.py`](../tests/repl/commands.py#L21) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | 13 konkrete REPL-Übersetzungen in Recording, lokale Parsefehler ohne Activity, korrekte Unknown-Type-Ablehnung. |
| **4.45 · PTY-Reaktionsfähigkeit und Terminal**<br>[`tests/repl/terminal.py`](../tests/repl/terminal.py#L21) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Echte schmale PTY, Unicode-Editierung unter Fremdantwort, exakter abgelehnter Pfad und Terminal-Restaurierung. |
| **4.46 · REPL beenden ist kein Session-Shutdown**<br>[`tests/repl/disconnect.py`](../tests/repl/disconnect.py#L22) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | quit/EOF/Ctrl+C beenden nur Client; zwölf echte Ticks je Variante, später explizites Shutdown. |
| **4.47 · Tracing-Opt-in, Filter und Layer-Bestätigung**<br>[`tests/reports/tracing_config.py`](../tests/reports/tracing_config.py#L11) | 9 | 10 | 9 | **9,3** | 9 | 9–10 | Offen | R1/R3: Negativ- und Genau-einmal-Zählung der Reports endet vor nachgewiesenem Providerabschluss. |
| **4.48 · Beobachtete nichtfatale Panics**<br>[`tests/reports/caught_panic.py`](../tests/reports/caught_panic.py#L11) | 9 | 10 | 9 | **9,3** | 9 | 9–10 | Offen | Panic und Weiterleben sind gut belegt. R1/R3: Report-Nachläufer liegen außerhalb des Zählfensters. |
| **4.49 · Echter panic=abort-Build**<br>[`tests/reports/panic_abort.py`](../tests/reports/panic_abort.py#L21) | 9 | 10 | 9 | **9,3** | 9 | 9–10 | Offen | Tatsächliches SIGABRT ist jetzt geprüft. R1/R3: spätere Provider-Ergebnisse werden nicht vollständig nachgezählt. |
| **4.50 · Unerwarteter Exit ohne Panic**<br>[`tests/reports/process_exit.py`](../tests/reports/process_exit.py#L11) | 9 | 10 | 9 | **9,3** | 9 | 9–10 | Offen | Exakte ProcessExit-Fälle sind geprüft. R1/R3: Reportzählung liegt vor vollständig beendetem Providerstrom. |
| **4.51 · Beschädigte Report-Marker**<br>[`tests/reports/invalid_marker.py`](../tests/reports/invalid_marker.py#L13) | 9 | 10 | 9 | **9,3** | 9 | 9–10 | Plan-Haken | Byte-/Marker-Vertrag stark. R1/R3: Genau-einmal-Audit erfasst mögliche spätere Report-Outcomes nicht vollständig. |
| **4.52 · Fehler vor Ready**<br>[`tests/reports/before_ready.py`](../tests/reports/before_ready.py#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Eigene PID/Mode bestätigt Start; fünf konkrete Vor-Ready-Diagnosen ohne Reports oder Commands. |
| **4.53 · Langsamer Provider blockiert Spiel und Clients nicht**<br>[`tests/reports/slow_provider.py`](../tests/reports/slow_provider.py#L15) | 10 | 10 | 8 | **9,3** | 10 | 8–10 | Offen | R3: entered.json kann nach exists() noch unvollständig sein; endgültige gh-Aufrufzahl wird vor Serverexit gezählt. Streuung 8–10. |
| **4.54 · Lokaler Provider und Konflikte**<br>[`tests/rust/test_local_provider.rs`](../tests/rust/test_local_provider.rs#L14) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Sechs konkurrierende Submits, exakt ein Created/fünf Existing, unveränderte Konfliktbytes und Symlinks. |
| **4.55 · GitHub-Vertrag und lokaler Fallback ohne Netzwerk**<br>[`tests/rust/test_github_provider.rs`](../tests/rust/test_github_provider.rs#L11) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Neun externe REST-Szenarien mit Pagination/Decoys, exakten Aufrufen und beiden Fallback-Ursachen. |
| **4.56 · Unveränderlicher Report-Snapshot**<br>[`tests/rust/test_report_snapshot.rs`](../tests/rust/test_report_snapshot.rs#L9) | 9 | 9 | 10 | **9,3** | 9 | 9–10 | Offen | Unveränderlichkeit stark. R1/R2: Mehrere Kontextwerte werden nur als vorhandene Felder, nicht gegen echte Startwerte geprüft. |
| **4.57 · Titel, Markdown und Fehlersignatur**<br>[`tests/rust/test_report_signature.rs`](../tests/rust/test_report_signature.rs#L7) | 9 | 9 | 9 | **9,0** | 9 | 9–9 | Offen | Alle drei: Marker/Fences/Titel/Hashes stark; vollständiger Markdown-Meldungs-, Backtrace- und Kontextinhalt nicht abgesichert. |
| **4.58 · Direkter Shutdown und Vorbedingungen**<br>[`tests/shutdown/command.py`](../tests/shutdown/command.py#L14) | 10 | 10 | 9 | **9,7** | 10 | 9–10 | Offen | Vorbedingungen und MAX-ID-Abschluss stark. R3: kein unabhängiger Nachweis, dass App-PID/-Gruppe danach weg ist. |
| **4.59 · Verwaltungs-Stopp löst aktive Arbeit auf**<br>[`tests/shutdown/session_stop.py`](../tests/shutdown/session_stop.py#L14) | 10 | 10 | 9 | **9,7** | 10 | 9–10 | Offen | Replay-/Footer-/Shutdownfolge stark. R3: vorhandene Process-Probe wird nicht auf tatsächliches Prozessende geprüft. |
| **4.60 · Stopp während Starting**<br>[`tests/shutdown/starting.py`](../tests/shutdown/starting.py#L15) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Nachweislich lebender Pre-Ready-Peer, keine Annahme, reguläres Ended und PID verschwunden. |
| **4.61 · Serverende mit mehreren Sessions und Reports**<br>[`tests/shutdown/server_stop.py`](../tests/shutdown/server_stop.py#L18) | 9 | 10 | 10 | **9,7** | 10 | 9–10 | Offen | Unabhängige Session-/Provider-Gates stark. R1: viele Beobachtungsschritte müssen in die feste Shutdownfrist passen. |
| **4.62 · Gemeinsame Shutdownfrist und erzwungenes Ende**<br>[`tests/shutdown/deadline.py`](../tests/shutdown/deadline.py#L17) | 9 | 10 | 10 | **9,7** | 10 | 9–10 | Offen | Messhypothesen werden sauber getrennt, Ambiguität ergibt Skip. R1: Vorbeobachtungen können das Fristbudget aufbrauchen. |
| **4.63 · Signale**<br>[`tests/shutdown/signals.py`](../tests/shutdown/signals.py#L17) | 9 | 9 | 9 | **9,0** | 9 | 9–9 | Offen | Alle drei: Zweites SIGINT muss inklusive Beobachterscheduling unter drei Sekunden wirken; Hostpause kann Fehlalarm erzeugen. |
| **4.64 · Pending-Ownership und einmaliger Empfang**<br>[`tests/rust/test_pending_ownership.rs`](../tests/rust/test_pending_ownership.rs#L7) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Gleiche IDs mit verschiedenen Outputtypen fremd empfangen, Eigentümer behält Ergebnis, genau einmal. |
| **4.65 · Pending-Drop ist kein Abbruch**<br>[`tests/rust/test_pending_drop.rs`](../tests/rust/test_pending_drop.rs#L8) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Wirklich offenes Pending droppen, natürlicher App-Fortschritt auf zwölf, History und Recording terminal. |
| **4.66 · Direkte Rust-Session terminal abschließen**<br>[`tests/rust/test_session_shutdown.rs`](../tests/rust/test_session_shutdown.rs#L8) | 10 | 10 | 9 | **9,7** | 10 | 9–10 | Offen | PID und Gruppe explizit geprüft. R3: starre Fünfsekundengrenze nach blockierendem Drop unterscheidet keine Schedulerpause. |
| **4.67 · History begrenzen, aktive Arbeit behalten**<br>[`tests/rust/test_history.rs`](../tests/rust/test_history.rs#L8) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | 55 unterscheidbare Commands, exaktes 50er-Annahmefenster, verdrängtes Pending bleibt empfangbar. |
| **4.68 · Event-Überlauf und reserviertes Endevent**<br>[`tests/rust/test_event_overflow.rs`](../tests/rust/test_event_overflow.rs#L9) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Offen | Ohne Eventabholung Produktüberlauf provoziert; 256 geordnete Failures, reserviertes finales Ended. |
| **4.69 · Verlustfreie Handle-Kodierung**<br>[`tests/rust/test_handle_encoding.rs`](../tests/rust/test_handle_encoding.rs#L7) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | 36 u32-Grenzkombinationen, literales JSON, Command-Roundtrip und echte öffentliche Entity-Konversion. |

### Interne Risikonachweise

| Test / Quelle | R1 | R2 | R3 | Ø / 10 | Median | Spanne | Lauf | Synthese |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| **4.71 · Native Eingaben und Fokusverlust**<br>[`src/session/input/tests/test_native_input.rs`](../src/session/input/tests/test_native_input.rs#L13) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Echte Bevy-Nachrichten vor Gate, virtuelle Holds über zwei Pässe und unabhängige native Positivkontrolle. |
| **4.72 · Textziel bei Fokuswechsel oder Despawn**<br>[`src/session/input/text/tests/test_target_binding.rs`](../src/session/input/text/tests/test_target_binding.rs#L8) | 10 | 10 | 9 | **9,7** | 10 | 9–10 | Plan-Haken | Zielbindung/Despawn gut geprüft. R3: Nach zweitem Flush wird positives Ziel A nicht erneut auf doppelte Zustellung geprüft. |
| **4.73 · Surface, Timeout und verspäteter GPU-Readback**<br>[`src/session/screenshot/capture/tests/test_readback_lifecycle.rs`](../src/session/screenshot/capture/tests/test_readback_lifecycle.rs#L9) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Surface/Timeout trotz gepuffertem Bild, verspätete Lieferung und echte CPU-PNG-Positivkontrolle. |
| **4.74 · Pfadaustausch beim Screenshot-Schreiben**<br>[`src/session/screenshot/destination/tests/test_path_replacement.rs`](../src/session/screenshot/destination/tests/test_path_replacement.rs#L4) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Austausch nach echtem prepare, Outside-/Originalbytes geschützt, legitimer In-Root-Link positiv. |
| **4.75 · Pfadaustausch bei Recording und Replay**<br>[`src/session/recording/file/tests/test_path_replacement.rs`](../src/session/recording/file/tests/test_path_replacement.rs#L4) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Beide realen Öffnungen mit kontrolliertem Parent-/Finaltausch; create_new und FIFO-Nichtblockierung. |
| **4.76 · ID-Kollision und ambiges Präfix**<br>[`src/server/tests/test_session_identity.rs`](../src/server/tests/test_session_identity.rs#L4) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Drei erzwungene ID-Kollisionen, freie vierte ID, erhaltene Bytes und echte ambige Präfixablehnung. |
| **4.77 · Request-ID-Erschöpfung**<br>[`src/session/coordinator/tests/test_request_id_exhaustion.rs`](../src/session/coordinator/tests/test_request_id_exhaustion.rs#L8) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Letzte normale ID bleibt empfangbar, keine weitere Annahme/History/Wire; Replay blockiert ohne Präfix. |
| **4.78 · Recording-/Replay-Übergangsrennen**<br>[`src/session/tests/test_transition_races.rs`](../src/session/tests/test_transition_races.rs#L87) | 10 | 10 | 10 | **10,0** | 10 | 10–10 | Plan-Haken | Echte Dateiworker an Header/Sync/Loader gehalten; Fehler/Stop-Reihenfolge, Acks und Pipe-Fences. |

### Separater externer Nachweis

| Test / Quelle | R1 | R2 | R3 | Ø / 10 | Median | Spanne | Lauf | Synthese |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |
| **4.81 · Echter GitHub-Provider gegen Testrepository**<br>[`tests/providers/github_live.py`](../tests/providers/github_live.py#L57) | 9 | 9 | 10 | **9,3** | 9 | 9–10 | Offen | Referenz-Oracle im aktuellen Code korrekt verschachtelt. R1/R2: echte Netzwerk-/Auth-/Remote-Voraussetzungen bleiben unkontrollierbar. |

## Abgehakte Arbeitspunkte ohne eigene Testnote

Auch diese sieben nummerierten Aufgaben sind berücksichtigt, aber keine sieben
weiteren Produkttests:

| Plan | Aufgabe | Einordnung |
| --- | --- | --- |
| 3.15 | Vorbereitung: App-Diagnoseaktion | Opt-in-Diagnosevorbereitung; Wirkung gehört zu den zugehörigen Fachtests. |
| 3.18 | Kontrolliert falsche Erwartung im Spielpfad | Zusätzlicher Kontrolllauf derselben Resource-Testimplementierung; keine allgemeine Mutationskampagne. |
| 3.19 | Gemeinsamer Einstieg und ehrliche Zusammenfassung | Gemeinsamer Einstieg; heute vollständiges lokales Inventar statt nur Pilot-Auswahl. |
| 3.20 | Pilotabschluss | Historischer Pilotabschluss, kein aktueller Gesamtabnahmelauf. |
| 4.1 | Vorbereitung: fehlende Referenz-App-Fähigkeiten | Referenz-App-Fähigkeiten und Fixtures, keine zusätzlichen Produkttests. |
| 4.70 | Abgleich zusätzlicher Risiken | Begründeter Risikoabgleich für öffentlich nicht zuverlässig erzeugbare Zwischenzustände. |
| 4.79 | Relevante Feature-Builds | Feature-/Target-/Runner-Integration; Buildbelege sind keine grafische Abnahme. |

Checklisten aus Schritt 1/2, die innere Konfigurations-Teilaufgabe 5.1 und
getrennte Offline-Runnerkontrollen erhöhen die 93 ebenfalls nicht.
Der äußere Gesamtabschluss 4.80 bleibt offen; maßgeblicher Arbeitsstand ist
ausschließlich der [Testplan](testing-plan.md).

Beim Review inventarisierte der gemeinsame Einstieg **92 lokale Produkttests plus
Kontrolle 3.18** sowie **4.81 separat**: insgesamt 94 Inventareinträge, aber
93 unterschiedliche Produkttests. Die zuvor dokumentierten CI-/Buildcache-
Themen stehen in der [Testanleitung](../tests/README.md), nicht als zweite
Arbeitsliste in dieser Qualitätsbewertung.

## Verifikation und Grenzen dieser Auswertung

- Drei abgeschlossene Delegationen, 93 unterschiedliche Code-IDs je Reviewer,
  insgesamt **279 vollständige Einzelwertungen**.
- Dimensionen, ganzzahlige Grenzen, Summen, mögliche Caps und vorhandene
  Evidenzdateien/-zeilen maschinell validiert.
- **Alle 277 Quellhashes** stimmen vor/nach den Reviews mit demselben Snapshot
  überein. Es wurden keine Stände oder nachträglichen Änderungen vermischt.
- Quellen für die zusammengefassten Hauptbefunde vom Parent geprüft; abweichende
  Detailbewertungen als solche erhalten. Keine vierte verdeckte Gesamtnote.
- Python-Syntax, Discovery/Inventar, Dokument-/Plan-/Tabellen-IDs und Links
  geprüft. HTML bei Desktop- und schmaler Breite einschließlich Filtern,
  Sortierung, Einzelreviewdetails und Pagination kontrolliert.
- **Keine Produkttests, Builds, App-/Server-/Fehlerprozessstarts, grafischen
  Sessions, Netzwerkanfragen oder gh-Aufrufe** für diese Auswertung.
  Keine neue Mutationstest-Kampagne oder statistische Stabilitätsmessung.

Testcode, Produktcode und Testplan wurden bei dieser Drei-Reviewer-Auswertung
nicht geändert. Die Noten bewerten statisch gelesenes Testdesign im vereinbarten
Scope; sie sind keine Messung erfolgreicher Fehlererkennungsraten.
