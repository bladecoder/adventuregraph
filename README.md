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

## Hardware v1.1

| Function | GPIO |
| --- | ---: |
| ST7789 CS / RST / D/C | 3 / 5 / 7 |
| ST7789 MOSI / SCK / backlight | 9 / 11 / 12 |
| Encoder A / B / switch | 38 / 40 / 36 |
| User button | 37 |
| Buzzer | 1 |

The encoder moves through choices after text appears and scrolls during animation. Its switch confirms a choice, skips the current line, or restarts at the end. The user button jumps to the start of the visible story; both buttons sound a short buzzer tone. PCB and enclosure references are in `docs/`. The removed Wokwi diagrams represented a different display controller and cannot verify this ST7789 firmware.

## Verification

```sh
cargo fmt --all --check
cargo clippy --workspace --exclude adventuregraph-esp --all-targets -- -D warnings
cargo test --workspace --exclude adventuregraph-esp
scripts/build.sh release
cargo tree -p adventuregraph-esp -e features
```

After an ESP build, inspect the ELF and linker map with the Xtensa `size`, `nm`, and `objdump` tools. Confirm `BLINKIMG`/`STORY_IMAGE` are in a flash-mapped `.rodata` section. Flash and test the display, encoder direction and switch, GPIO37 button, and buzzer on the physical board before accepting hardware behavior.
