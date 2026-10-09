# SDL simulator

Run `cargo run -p adventuregraph-simulator` from the repository root. The simulator draws the same Ratatui UI and binary Ink image as the firmware on a 240 × 320 pixel window. SDL2 is built from source and needs CMake and a C/C++ compiler.

Generate screenshots of all eight banner modes without opening an SDL window:

```sh
cargo run -p adventuregraph-simulator --example banner_gallery -- /tmp/adventuregraph-banners
```

This renders the actual shared story through Mousefood using the same static
6×10 font as ESP. Each PNG captures a banner during its following Ink wait.
