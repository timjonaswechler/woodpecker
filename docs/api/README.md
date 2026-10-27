# API-Dokumentation

- [target.md](target.md): maßgeblicher öffentlicher Vertrag und Modulverantwortlichkeiten.
- [goal.rs](goal.rs): Rust-ähnliche Interface-Skizze; muss nicht kompilieren.
- [usage.md](usage.md): aktuelle CLI-Bedienung und Start mit der Referenz-App.
- [Testanleitung](../../tests/README.md): ausführbare Auswahl, Voraussetzungen und Grenzen.
- [Testsuite-Plan](../testing-plan.md): einziger aktueller Arbeitsstand und Abnahmenachweise.

Der vollständige lokale Testumfang und der getrennte echte GitHub-Fall sind
belegt. Sie ersetzen weder künftige Läufe noch eine Prüfung aller denkbaren
Anwendungsszenarien. Stand und genaue Belege werden nur im Testplan fortgeschrieben.

[implementation-plan.md](implementation-plan.md) und [next-steps.md](next-steps.md)
sind **historische** Entwicklungs-/Übergabeaufzeichnungen vor dem Suite-Neuaufbau.
Ihre Checkboxen, Befehle und inzwischen entfernten Szenen sind keine aktuelle
Arbeitsliste oder ausführbare Testanleitung. Historische Pfade bleiben als Zitate,
nicht als Links zu nicht mehr vorhandenen Dateien.

Für Screenshotgrenzen: [Fehler und Guard](diagnostics/black-screenshots.md),
[historische Reproduktion](diagnostics/screenshot-evidence.md),
[datierter Upstream-Stand](diagnostics/bevy-upstream-screenshot-status.md) und
[zurückgestellte Resize-Diagnose](diagnostics/window-schedules.md).
[Research](research/) bewahrt technische Quellenstände,
[ADRs](../adr/README.md) die Entscheidungsherkunft; beide sind keine zweite Zielbeschreibung.

Eine Regel hat eine maßgebliche Stelle. Code ist technische Referenz, keine
automatische Anforderung. Neue Produktentscheidungen brauchen ausdrückliche
Zustimmung; vorhandene Nutzeränderungen bleiben erhalten. Subagenten werden
nur nach ausdrücklicher Zustimmung eingesetzt.
