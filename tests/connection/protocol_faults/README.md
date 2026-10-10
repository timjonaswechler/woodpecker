# Fehler am Protokoll und Prozessende

`contracts.py` prüft die Reaktion auf ungültige Antworten, EOF, vorzeitiges
Prozessende und blockiertes Shutdown. `peer` ist dafür ein absichtlich
fehlerhafter Protokollpartner ohne Bevy. Auch Server-Shutdown-Verträge verwenden
diesen Prozess, um kontrolliert hängende Sessions zu erzeugen.

Er ersetzt nicht die gemeinsame Bevy-App für normale Command-Verträge.
Der Cargo-Name `process_fixture` bezeichnet nur das Fehlerprogramm. Es gibt
hier keinen Cargo-Test-Target mehr. Der Suite-Runner baut den Peer vor den
Verträgen und führt diese über echte Sessions und JSON/REPL aus.
`received.jsonl` enthält die tatsächlich empfangenen Commands und dient zur
Synchronisation mit intern gestarteter Replay-Arbeit.

```sh
python3 -m tests.run --feature connection --layer system --faults-only
```
