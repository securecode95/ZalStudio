# ZalStudio

A terminal-based photo kiosk software written in Rust, designed for printing photos via CUPS/Gutenprint to a Mitsubishi CP-9550DW-S dye-sub printer.

## Architecture

```
┌─────────────────────────┐
│   ZalStudio (Rust TUI)  │
│   - Photo gallery       │
│   - Size selection      │
│   - Print queue         │
│   - Job tracking        │
└───────────┬─────────────┘
            │ lp command
            ▼
┌─────────────────────────┐
│   CUPS (localhost:631)  │
│   - Gutenprint backend  │
│   - CP-9550DW-S driver  │
└───────────┬─────────────┘
            │ USB
            ▼
┌─────────────────────────┐
│   Mitsubishi CP-9550DW-S│
└─────────────────────────┘
```

## Quick Start

```bash
cargo run
```

Place photos in the `./photos` directory (or configure a different path — see below).

## Controls

### Gallery Tab
| Key | Action |
|-----|--------|
| `↑` / `↓` | Select photo |
| `←` / `→` | Change paper size |
| `+` / `-` | Adjust copies |
| `A` | Add selected photo to queue |
| `P` | Print queue (shows confirmation) |
| `R` | Rescan photo directory |
| `Tab` | Switch to Queue tab |
| `?` | Toggle help |
| `Q` | Quit |

### Queue Tab
| Key | Action |
|-----|--------|
| `↑` / `↓` | Select queue item |
| `Delete` | Remove item from queue |
| `P` | Print queue |
| `C` | Clear completed/failed jobs |
| `Tab` | Switch to Gallery tab |
| `?` | Toggle help |
| `Q` | Quit |

## Configuration

On first run, ZalStudio creates a config file at:

- **Linux:** `~/.config/zalstudio/config.toml`
- **Windows:** `%APPDATA%\zalstudio\config.toml`
- **macOS:** `~/Library/Application Support/zalstudio/config.toml`

Example:

```toml
printer_name = "CP-9550DW-S"
photo_directory = "./photos"
paper_sizes = ["4x6", "5x7", "6x8", "10x15cm"]
default_paper_size = 0
copies_default = 1
fit_to_page = true
```

## Dependencies

- [ratatui](https://github.com/ratatui/ratatui) — Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Cross-platform terminal control
- [image](https://github.com/image-rs/image) — Image metadata reading
- [kamadak-exif](https://github.com/kamadak/exif-rs) — EXIF data parsing
- [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml) — Configuration
- [walkdir](https://github.com/BurntSushi/walkdir) — Directory traversal
- [dirs](https://github.com/dirs-dev/dirs-rs) — Config directory resolution
- [chrono](https://github.com/chronotope/chrono) — Date/time handling

## CUPS / Printer Setup

Make sure your printer is installed in CUPS and the Gutenprint driver is loaded:

```bash
# List printers
lpstat -p -d

# Test print
lp -d CP-9550DW-S -o media=4x6 -o fit-to-page test.jpg
```

## License

MIT
