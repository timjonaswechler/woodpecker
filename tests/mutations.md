# Begrenzte Mutationsprüfung — Schritt 7

`python3 -m tests.mutations` prüft ausschließlich die Fehlersensitivität von vier
bestehenden fensterlosen R0/F0-Verträgen. Kein SDK-Ausbau, keine neue Produktsuite,
keine universelle Sensitivitätsaussage oder Aufwertung historischer Qualitätsnoten.
Das [Abnahmeinventar](README.md#inventar-und-zuständigkeiten) bleibt bei 95 Einträgen;
`tests.runner_checks.mutations` gehört nur zum Offline-Harness.

## Umfang

| Mutation | Bestehender Owner | Erwarteter fachlicher Fehlschlag |
| --- | --- | --- |
| Oberstes Generation-Bit verlieren | 4.69 `handle_encoding_contract` | rekonstruierte Generation `0` statt `2147483648` |
| Native `KeyboardInput`-Messages behalten | 4.71 `native_input_cannot_enter_controlled_ticks_or_release_virtual_holds` | `empty!` meldet `KeyboardInput` nach Capture |
| Text beim Flush zum aktuellen Fokus umleiten | 4.72 `accepted_text_stays_bound_across_focus_despawn_and_editability_changes` | B enthält `beta:Grüße 🦉` statt `beta:` |
| Normalen Request-Zähler nicht erhöhen | 4.77 `exhausted_ids_never_admit_work_or_lose_pending_and_reserved_shutdown` | zweiter Send liefert nicht `RequestIdExhausted`, ID wird wiederverwendet |
| Bekannte Äquivalenz: Generation `\| 0` | derselbe 4.69-Owner | unverändert bestanden; für jedes `u32` gilt `x \| 0 == x`, derselbe Konstruktor erhält denselben Wert |

Die letzte Zeile ist eine vorab begründete Kontrolle, kein nachträglich zum
„äquivalenten“ Ergebnis umgedeuteter Überlebender. Assertions/Produktquellen im
Authoring-Checkout werden nie verändert. Alle Stellen müssen eindeutig passen;
ein anderer Quellstand kann deshalb vor dem Build abbrechen.

## Ausführen

Voraussetzungen: Unix, Python 3.11+, Git, passende Rust-/Cargo-Version und bereits
offline verfügbare Dependencies. Der Authoring-Checkout muss außerhalb des
geschützten Parent-Checkouts liegen. Der Parent wird nur gelesen; einzig das
explizite neue Beweisunterverzeichnis unter `target/test-runs/` erhält Ausgaben.
Vorherige Belege werden nicht überschrieben.

1. Katalog und Offline-Sicherheitsprüfungen ansehen; beide starten kein Cargo:

   ```sh
   python3 -m tests.mutations --help
   python3 -m tests.mutations --list
   python3 -m unittest -v tests.runner_checks.mutations
   ```

2. Parent-Pfade und dessen vorhandene Vorher-Hashdatei angeben. Die Datei enthält
   für `target/debug/woodpecker` und `target/debug/test_app` jeweils `sha256` und
   `size`. Platzhalter durch die eigenen absoluten Pfade ersetzen:

   ```sh
   python3 -m tests.mutations --source-ref HEAD \
     --parent-root /ABSOLUT/PARENT \
     --parent-products /ABSOLUT/PARENT/target/test-runs/KAMPAGNE/original-build-products-before-mutations.json \
     --evidence-dir /ABSOLUT/PARENT/target/test-runs/KAMPAGNE/mutation-evidence \
     --clone-parent-cache
   ```

   `HEAD` ist der dokumentierte Standard und wird einmal zu einem Commit aufgelöst;
   für reproduzierbare Wiederholung dessen vollständige SHA angeben. Ohne
   `--clone-parent-cache` entsteht ein leerer privater Cache. Mit diesem Flag ist
   ausschließlich macOS/APFS `cp -cR` erlaubt: kein stiller Fallback, keine
   Symlinks oder Parent-Hardlinks. Der Parent-Cache darf gleichzeitig nicht gebaut
   werden. Das Werkzeug schreibt dort nicht und ändert keine Berechtigungen.

3. Den gemeldeten `summary.json`-Pfad prüfen. Exit **0** bedeutet vollständige
   begrenzte Kampagne ohne Überlebende, **1** eine vollständige Kampagne mit
   Überlebenden, **2** ungültige/unvollständige Ausführung. Das ist nie eine neue
   Gesamtabnahme. Setup-, Build-, Baseline- und Cleanup-Fehler stoppen ohne Retry.

## Isolation und Belege

Ein Lauf extrahiert den gepinnten Commit per `git archive` in eine neue
**Nicht-Git-Quellkopie** unter dem eigenen ignorierten `target/test-runs/`.
Enthalten sind Cargo-/Lock-Dateien, SDK-Quellen, vorhandene Rust-Tests und der
benötigte interne Wire-Partner sowie Workspace-Manifeste/-Quellen, keine großen
App-Assets. Workspace-Apps werden nicht gebaut oder gestartet.

Jeder Cargo-Aufruf nutzt `--locked`, `CARGO_NET_OFFLINE=true` und ausschließlich
`CARGO_TARGET_DIR` im eigenen Laufordner; `WOODPECKER_TEST_RENDERED` bleibt unset.
Zuerst werden echte Tests mit `--no-run --message-format=json` kompiliert und die
vier Owner einzeln grün ausgeführt. Nach jeder einzelnen Produktmutation werden
die tatsächlichen Compilerartefakte erneut gebaut, deren Testnamen aufgelistet
und genau ein bestehender Test mit `--exact --nocapture --test-threads=1` ausgeführt.
Build und Runtime sind getrennte Nachweise; keine nachgebauten Produktpfade.

Ein Kill benötigt Exit 101, genau einen nicht ignorierten fehlgeschlagenen Test,
die richtige Panic-Funktion, die aus der unveränderten Testquelle ermittelte
Assertion-Zeile und die erwartete fachliche Signatur. Andere Panics, Nulltests,
Crashes, Timeouts und erzwungenes Harness-Cleanup sind keine Erkennung.
`killed`, `survived`, `known-equivalent`, `invalid`, `incomplete` bleiben getrennt.
Überlebende werden weder ersetzt noch durch schwächere Assertions „repariert“.

Pfade und Quellhashes stehen vor der Mutation im Bericht. `finally` restauriert
die Produktdatei und vergleicht alle extrahierten Quellen einschließlich Oracles
zwischen Kandidaten. Vorher-/Nachher-Hashes schützen Parent-Quellen einschließlich
bereits fehlender Dateien und die beiden ursprünglichen Binaries. Die eigenen
Mutationsstellen werden ebenfalls unverändert nachgewiesen. Prozessausgaben,
Build-/Runtime-Status, Patches, Quellhashes, Wire-Artefakte und PID-/Gruppenabwesenheit
werden ohne Cache/Binaries in den gemeldeten dauerhaften Beweisordner kopiert.
Erst nach erfolgreicher Beweiskopie wird der eigene Laufordner einschließlich
Quellkopie und privatem Cargo-Cache entfernt, auch nach fehlgeschlagenen Builds.
`workspace_removed` bestätigt das im dauerhaften Bericht; vorher bleibt `complete`
false. Scheitert die Beweiskopie, bleibt der Laufordner zur Wiederherstellung
bestehen. Scheitert das Entfernen, meldet der Bericht den Cleanup-Fehler und Exit 2.
Parent-Cache, frühere Laufordner und dauerhafte Belege bleiben unangetastet.
Absolute Pfade in den aufgezeichneten Commands, Logs und Patch-Verweisen beschreiben
den ursprünglichen Lauf. Nach Cleanup liegen die Belegdateien mit derselben relativen
Struktur neben der dauerhaften `summary.json`, nicht mehr an diesen temporären Pfaden.
