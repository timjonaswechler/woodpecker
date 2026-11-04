# CI im Quellcode pausiert

Auf ausdrücklichen Nutzerwunsch ist die automatische CI für den Headless-Arbeits-PR
pausiert. `contracts.yml` liegt unverändert außerhalb von `.github/workflows/`.
GitHub Actions erkennt nur dort abgelegte `.yml`-/`.yaml`-Workflowdateien;
Push und Pull Request dieses Branches starten deshalb keinen Repository-Workflow.
Die Repository-/Actions-Einstellungen und der Branchschutz bleiben unverändert.

Die Pause gilt zunächst für diesen Branch. Erst ein Merge übernimmt sie nach
`main`; andere Branches mit aktiven Workflowdateien bleiben davon unberührt.
Fehlende Pflichtchecks können einen Merge blockieren und werden nicht umgangen.

## Bewusst reaktivieren

Erst nach erneuter Freigabe in einem eigenen Commit zurückverschieben:

```sh
mkdir -p .github/workflows
git mv .github/disabled-workflows/contracts.yml .github/workflows/contracts.yml
```

**Achtung:** Der anschließende Push beziehungsweise PR kann sofort wieder die
ursprüngliche Build-/Test-CI starten. Die Pause ist kein bestandener Test und
keine Reparatur der CI. Eine spätere Headless-CI benötigt eigene qualifizierte
Runner-/Adapterprofile.

Quelle: [GitHub Actions: Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#about-yaml-syntax-for-workflows).
