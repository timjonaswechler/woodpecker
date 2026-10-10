# Reports bei echtem Panic-Abort

`contracts.py` prüft den Report nach einem Panic mit `panic=abort` über einen
echten Server und JSON/REPL.
`program` besitzt dafür ein eigenes Cargo-Profil und prüft die Panic-Strategie
beim Kompilieren. Es verwendet die Fehlerquelle aus `../failure_source.rs`.
Ein Aufruf von `process::abort()` würde diesen Panic-Fall nicht nachweisen.

Der Cargo-Name `panic_abort_fixture` bezeichnet nur das Abort-Programm.
Es gibt hier keinen Cargo-Test-Target mehr.
Der Suite-Runner baut das Programm vor den Report-Verträgen.
Sobald die gemeinsame Bevy-App dieselben Fehler erzeugen kann, soll dieser
Nachweis deren Quellcode mit einem Abort-Build verwenden.

```sh
python3 -m tests.run --scenario reports.panic_abort_reports_once_before_end --layer system
```
