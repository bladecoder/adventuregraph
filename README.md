# Adventuregraph

A handheld, battery-powered text adventure player built around an ESP32-S2 and written in Rust with `esp-idf-hal`, `mipidsi`, and the `bladeink` runtime. It renders Ink stories on a SPI TFT display and is driven with a rotary encoder, optional push button, and a piezo buzzer for feedback.

- Firmware: Rust 2024 edition targeting `xtensa-esp32s2-espidf` (crate `adventuregraph-esp`).
- Story format: Ink, compiled into `assets/story.ink.json` (source under `raw/ink-src/`).
- Desktop builds: TUI preview (`adventuregraph-tui`) and an SDL-based simulator (`adventuregraph-simulator`).
- Displays: ST7789 (default) or ST7735 via the `display-st7735` feature.
- Hardware assets: PCBs and enclosure models in `docs/`, Wokwi wiring in `esp/wokwi-esp32s2-v1*`.

## Repository Layout

- `core/` – Shared story runtime, UI primitives, and game state used by every target.
- `esp/` – ESP32-S2 firmware (features: `pcbv1`, `display-st7735`, `software-scroll`).
- `tui/` – Terminal UI preview using `crossterm`/`ratatui` on the host.
- `simulator/` – Desktop simulator using `embedded-graphics-simulator` (needs SDL2).
- `assets/` – Compiled Ink story (`story.ink.json`) and artwork.
- `raw/ink-src/` – Source Ink scripts.
- `docs/` – PCB exports (1.0, 1.1, 1.2) and enclosure CAD/STL files.
- `scripts/` – Helper scripts for building/flashing the ESP target.
- `esp/wokwi-esp32s2-v1*/` – Wokwi simulations matching hardware revisions.

## Building and Flashing

1. Set up the ESP-IDF toolchain and export the environment (for example `source ../export.sh`).
2. Build firmware for ESP32-S2 from the workspace root:
   ```bash
   cargo build -p adventuregraph-esp --target xtensa-esp32s2-espidf
   ```
3. Flash (one option):
   ```bash
   web-flash --chip esp32s2 target/xtensa-esp32-espidf/debug/adventuregraph-esp
   ```

Host builds:
- Terminal preview: `cargo run -p adventuregraph-tui`
- SDL simulator (requires SDL2): `cargo run -p adventuregraph-simulator`

Feature flags:
- `pcbv1` – Routes the buzzer to GPIO18 (hardware v1.0). Without it, the buzzer uses GPIO1 (hardware v1.1+).
- `display-st7735` – Selects the smaller ST7735 display driver; otherwise the ST7789 is used.
- `software-scroll` – Enables software-based scrolling.

## Hardware

The device uses an ESP32-S2 with 2 MB PSRAM, a SPI TFT (ST7789 by default), a KY-040–style rotary encoder with integrated switch, an optional extra push button, and a piezo buzzer. Charging/power management (TP4056, LDO, 18650 cell) and the 3D-printed enclosure live in `docs/`.

### Pinout – PCB v1.0 (feature `pcbv1`)

| Function                | ESP32-S2 pin |
| ----------------------- | ------------ |
| TFT CS                  | GPIO3        |
| TFT RST                 | GPIO5        |
| TFT D/C                 | GPIO7        |
| TFT MOSI (SDA)          | GPIO9        |
| TFT SCK (SCL)           | GPIO11       |
| TFT backlight (LED)     | GPIO12       |
| Rotary encoder A (CLK)  | GPIO38       |
| Rotary encoder B (DT)   | GPIO40       |
| Rotary encoder switch   | GPIO36       |
| Buzzer (+)              | GPIO18       |
| User button             | — (not fitted) |
| Power                   | 3V3 / GND    |

Reference wiring: `wokwi-esp32s2-v1/diagram.json`.

### Pinout – PCB v1.1 (default build)

| Function                | ESP32-S2 pin |
| ----------------------- | ------------ |
| TFT CS                  | GPIO3        |
| TFT RST                 | GPIO5        |
| TFT D/C                 | GPIO7        |
| TFT MOSI (SDA)          | GPIO9        |
| TFT SCK (SCL)           | GPIO11       |
| TFT backlight (LED)     | GPIO12       |
| Rotary encoder A (CLK)  | GPIO38       |
| Rotary encoder B (DT)   | GPIO40       |
| Rotary encoder switch   | GPIO36       |
| Buzzer (+)              | GPIO1        |
| User button             | GPIO37 → GND |
| Power                   | 3V3 / GND    |

Reference wiring: `wokwi-esp32s2-v1.1/diagram.json`.

### Notes

- Both revisions share the same display and encoder wiring; only the buzzer (GPIO18 → GPIO1) and an added user button on GPIO37 change between v1.0 and v1.1.
- PCB/3D files for v1.0 and v1.1 are in `docs/`; v1.2 PCB artwork is also included for reference.
