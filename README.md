# Adventuregraph v2

Adventuregraph plays Ink stories on an ESP32-S2 handheld, a terminal, or a 240 × 320 SDL window. All three use the same `no_std + alloc` story and Ratatui drawing code in `core/`.

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

`ink/story.ink` is the editable command and color example, with `->start` as its entry point. Run `scripts/compileink.sh` with `rinklecate` on `PATH` to regenerate `assets/story.ink.json` (override the compiler with `INKLECATE` if needed). `core/build.rs` converts the JSON into Blade Ink `.inkb` bytes. All platforms open the same flash-compatible image, using the pinned Blade Ink runtime without target-side JSON parsers.

The shared core captures tags immediately after each Ink continuation and validates the whole block before presentation. Results beginning with `>` are commands, with lowercase names. Attributes use `name=value`, separated by spaces (no spaces inside an attribute). `>cls` takes no attributes. Clear runs at its place in the animation without consuming time, resets the visible history and scroll, and preserves pending output and Ink state. Enter completes the current animated Ink result or skips the current wait, executes following commands, and starts the next output; that keypress never selects a choice. Choices appear after all pending text and waits finish.

Use `#color:red` or `#color: ff0000` to set a line's foreground. Ratatui color names and exactly six hexadecimal digits without a prefix are supported; palette indices are not. Use `#bgcolor: red` to set a line's background with the same color formats. Each line inherits the current defaults; tags override only that line, with the last valid tag for each color winning. Unknown tags are ignored. Invalid color tags and unknown commands return `StoryError::InvalidStoryState`, including the offending content. Command color tags are validated but do not render.

`>set defaultcolor=red` changes the default foreground; `>set defaultbgcolor=green` changes the default background. Set both with `>set defaultcolor=red defaultbgcolor=green`. Changes execute at their position in the animation and affect subsequent lines without changing earlier text. Defaults persist through clear and choices; restart restores the initial style. `reset` restores the terminal default for either color. Repeated attributes use the last valid value. Missing, malformed, unknown attributes and invalid values return `StoryError::InvalidStoryState` before presenting the block. `App::lines()` exposes presented `StoryLine { text, style, alignment, banner }` values, including the current animated line. Restart clears the history, styles, and pending events.

`>set text-animation=false` makes subsequent text appear immediately; `>set text-animation=true` restores the 20-character-per-second animation. The attribute can be combined with default color attributes. The setting persists through clear and choices, and restart enables animation again. Only lowercase `true` and `false` are valid.

Use `#text-animation:false` or `#text-animation:true` to override animation for only the tagged line. This can disable animation while the default is enabled or animate one line while the default is disabled; following lines continue to use the current setting. Only lowercase `true` and `false` are valid, and invalid values return `StoryError::InvalidStoryState`.

`>wait time=N` pauses pending output for N whole seconds, including when text animation is disabled. N must be a nonnegative integer that fits in milliseconds as a `u64`; zero adds no delay. Waits execute in order, preserve the text already displayed, and delay choices at the end of a block. Platforms keep drawing and handling input; Enter skips the current wait without selecting a choice. `App::is_waiting()` reports a wait separately from `is_animating()`. Ticks carry unused elapsed time across waits, text and immediate commands. Missing or invalid wait attributes return `StoryError::InvalidStoryState` during block preparation.

`#align:center`, `#align:right`, and `#align:left` align only the tagged line, including wrapped rows and partially revealed Unicode text. Untagged lines are left-aligned; the last alignment tag on a line wins. Alignment combines with foreground and background tags. Invalid alignment values return `StoryError::InvalidStoryState` during block preparation; command tags are also validated.

The fixtures in `core/tests/fixtures/` have their compiled JSON checked in. Regenerate a fixture after editing, for example `(cd core/tests/fixtures && rinklecate -o defaults.ink.json defaults.ink)`.

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
| Display SPI speed | 40 MHz | 20 MHz |

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

Use `#banner:MODE` to render a line directly with `tui-big-text = "=0.8.10"`.
Modes: `full`, `half-height`, `half-width`, `quadrant`, `third-height`, `sextant`,
`quarter-height`, and `octant`. Values are trimmed and the last valid tag wins;
empty or unknown values fail block preparation. Original case and accents are
preserved; glyph support follows BigText. Banners use native multiline rendering,
horizontal clipping, colors, and alignment, without word wrapping or extra scaling.
Animation reveals the original characters; `#text-animation:false` shows the whole
banner immediately. History scrolls by terminal rows, including inside a glyph.
Only visible banner lines are rasterized into one reusable viewport-width buffer
of eight rows. Normal text retains wrapping and blank separators.

ESP and simulator enable the core's `embedded-fonts` feature and share a static
6×10 Unicode font with block shapes for every mode. Existing Unicode glyphs are
preserved except block symbols, whose bitmaps use exact Unicode subdivisions.
Physical display verification still requires the board.
