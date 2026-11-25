# Projektregeln

## Abschluss mit Codewort „Feierabend“

Wenn der Nutzer am Ende einer Session **„Feierabend“** (oder „Feier abend“) schreibt, ist das die ausdrückliche Freigabe, den aktuellen Stand zu veröffentlichen:

1. `cargo fmt`, `cargo clippy --all-targets` und `cargo build --release` ausführen.
2. Alle beabsichtigten Änderungen committen und nach `origin/main` pushen.
3. Version in `Cargo.toml` und `packaging/PKGBUILD` gemeinsam erhöhen.
4. Tag `vX.Y.Z` setzen und pushen.
5. Source-Tarball und `PKGBUILD`/`.SRCINFO` mit korrekter `sha256sums` zum GitHub-Release hochladen bzw. aktualisieren, damit immer die neueste Version heruntergeladen werden kann.
6. Falls ein AUR-Account mit hinterlegtem SSH-Key vorhanden ist: `PKGBUILD` und `.SRCINFO` nach `lumen-browser` pushen.

Ohne das Codewort nicht automatisch committen oder veröffentlichen.
