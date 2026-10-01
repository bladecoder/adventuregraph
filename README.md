# Adventuregraph v2

Adventuregraph plays *The Intercept* on an ESP32-S2 handheld, a terminal, or a 240 × 320 SDL window. All three use the same `no_std + alloc` story and Ratatui drawing code in `core/`.

## Build and run

The firmware uses Espressif's `esp-hal` bare-metal stack. Install the Xtensa toolchain with `espup install -t esp32s2`, source its export file, and install `espflash`. The project skeleton was generated with `esp-generate 1.4.0 -o esp32s2 -o alloc`; dependencies are pinned in `Cargo.toml` and `Cargo.lock`.

```sh
cargo test --workspace --exclude adventuregraph-esp
cargo run -p adventuregraph-tui
cargo run -p adventuregraph-simulator
scripts/build.sh release
scripts/flash.sh release
```

The simulator builds SDL2 from source and requires CMake and a C/C++ compiler. Its window is 240 × 320 pixels at 2× scale. On both host versions, arrows or `j`/`k` select, Enter confirms or skips the current line, PageUp/PageDown scroll, Home/End jump, digits 1–9 choose, and `q` quits.

`scripts/build.sh` runs Cargo from `esp/`, where `.cargo/config.toml` selects `xtensa-esp32s2-none-elf`. The firmware starts with a 128 KiB internal heap. No PSRAM allocator is configured; any future large allocation should be explicit, keeping SPI transfer buffers in internal RAM. A build alone does not establish actual heap headroom or hardware behavior.

## Story assets

`assets/story.ink.json` is the existing compiled *The Intercept* story. `core/build.rs` converts it into Blade Ink `.inkb` bytes at build time. All platforms call `Story::new_from_image_with_seed`; the firmware embeds the bytes as a read-only static. The Blade Ink dependency is pinned to commit `007e99cf649f34ced27744067f565bd359c7c466` on `feat/flash-story-image`, with `binary-image` and without JSON story parsers in the target dependency. `raw/ink-src/TheIntercept.ink` remains the editable source; run `scripts/compileink.sh` with `inklecate` on `PATH` to regenerate the JSON deliberately.

## Board profiles

GPIO assignments and assembly settings live in `esp/boards/*.toml`. The default is `v1.1`; select a profile for both building and flashing with:

```sh
ADVENTUREGRAPH_BOARD=v1.0 scripts/build.sh release
ADVENTUREGRAPH_BOARD=v1.0 scripts/flash.sh release
```

Direct Cargo builds from `esp/` accept the same environment variable. `build.rs` generates typed GPIO bindings from the selected profile, rebuilding when its file or selection changes. Cargo prints the selected profile. Builds share the usual ELF path, so `flash.sh` always builds the selected profile before flashing it.

To add an assembly, copy a profile to `esp/boards/my-assembly.toml`, edit the TOML, and select `ADVENTUREGRAPH_BOARD=my-assembly`. Rebuild and flash after changing a profile; there is no runtime pin configuration.

- `[display]`: required `cs`, `reset`, `dc`, `mosi`, `sck`, `backlight`; optional `spi_mhz` (1–80, default 40), `rotation` (0/90/180/270, default 180), and `backlight_active_high` (default true). These profiles use the ST7789 240 × 320 driver.
- `[encoder]`: required `a`, `b`, `button`; optional `reverse` (default false), `pull`, `button_pull` (both default `"up"`), and `button_active_low` (default true).
- `[button]`: optional additional button; when present, `pin` is required, with optional `active_low` (default true) and `pull` (default `"up"`). Omit the entire section when absent.
- `[buzzer]`: optional piezo buzzer with required `pin`. Omit the entire section to disable feedback.

Pull values are `"up"`, `"down"`, or `"none"`. Active-high buttons generally need `"down"`; `"none"` requires appropriate external resistors. All pin numbers are ESP32-S2 GPIO numbers, not connector positions.

The build rejects unknown fields, duplicate GPIOs, nonexistent pins, GPIO46 as an output, and invalid settings. Boot strapping (0/45/46), USB (19/20), and module memory (26–32) pins require an explicit top-level `allow_reserved_pins = true` before any section. Only opt in after checking the module schematic and boot/USB requirements; validation cannot determine which pins your module actually exposes. The SPI setting must also suit the display and wiring.

## Hardware v1.0 and v1.1

| Function | v1.0 GPIO | v1.1 GPIO |
| --- | ---: | ---: |
| ST7789 CS / RST / D/C | 3 / 5 / 7 | 3 / 5 / 7 |
| ST7789 MOSI / SCK / backlight | 9 / 11 / 12 | 9 / 11 / 12 |
| Encoder A / B / switch | 38 / 40 / 36 | 38 / 40 / 36 |
| User button | Not fitted | 37 |
| Buzzer | 18 | 1 |

The encoder moves through choices after text appears and scrolls during animation. Its switch confirms a choice, skips the current line, or restarts at the end. The user button jumps to the start of the visible story; both buttons sound a short buzzer tone when fitted. PCB and enclosure references are in `docs/`. The removed Wokwi diagrams represented a different display controller and cannot verify this ST7789 firmware.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --exclude adventuregraph-esp --all-targets -- -D warnings
cargo test --workspace --exclude adventuregraph-esp
ADVENTUREGRAPH_BOARD=v1.0 scripts/build.sh release
ADVENTUREGRAPH_BOARD=v1.1 scripts/build.sh release
cargo tree -p adventuregraph-esp -e features
```

After an ESP build, inspect the ELF and linker map with the Xtensa `size`, `nm`, and `objdump` tools. Confirm `BLINKIMG`/`STORY_IMAGE` are in a flash-mapped `.rodata` section. Flash and test the display, encoder direction and switch, GPIO37 button, and buzzer on the physical board before accepting hardware behavior.
