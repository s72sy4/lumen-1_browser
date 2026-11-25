# lumen-1_browser

Ein schlanker Browser für Arch Linux und Wayland mit GTK3/WebKitGTK und einer Oberfläche im Ryoku-Stil.

## Funktionen

- Startseite mit Uhr, Datum und zentrierter Suchleiste
- Farben passen sich dem gewählten Wallpaper an
- Vertikale Tab-Liste (`Strg+L`), smooth ein-/ausgeblendet
- Werbeblocker inklusive YouTube-Ad-Cleaner
- Transparenz-Modus: nur der Desktop scheint durch, das Browser-Wallpaper verschwindet
- Passwort-Tresor und Profile
- Dark-Mode (Light-Mode wurde bewusst entfernt)

## Installation

### AUR (sobald die AUR-Registrierung wieder offen ist)

```bash
yay -S lumen-browser
# oder
paru -S lumen-browser
```

### Manuell über das Release

```bash
mkdir lumen-browser && cd lumen-browser
curl -LO https://github.com/s72sy4/lumen-1_browser/releases/download/v0.1.0/aur-submission.tar.gz
tar -xzf aur-submission.tar.gz
makepkg -si
```

`aur-submission.tar.gz` enthält `PKGBUILD` und `.SRCINFO`. Der Source-Tarball liegt ebenfalls im Release.

### Aus dem Quellcode

```bash
git clone https://github.com/s72sy4/lumen-1_browser.git
cd lumen-1_browser
cargo build --release
./target/release/lumen-1_browser
```

Benötigt: `rust`/`cargo`, `gtk3`, `webkit2gtk-4.1`. Für `makepkg` zusätzlich `base-devel`.

## Tastenkürzel

| Kürzel | Funktion |
| --- | --- |
| `Strg+L` | Tab-Liste ein-/ausblenden |
| `Strg+T` | Neuer Tab |
| `Strg+W` | Tab schließen |
| `Strg+K` | Adressleiste fokussieren |
| `Strg+,` | Einstellungen |
| `Esc` | Overlays schließen |

## Wallpapers

Die mitgelieferten Wallpapers liegen unter `assets/wallpapers` und werden bei der AUR-Installation nach `/usr/share/lumen-1_browser/wallpapers` installiert. In den Einstellungen können zusätzlich eigene Bilder aus dem Pictures-Verzeichnis oder per Datei-Dialog gewählt werden; die Akzentfarben passen sich sofort an.

## Lizenz

MIT – siehe `LICENSE`.
