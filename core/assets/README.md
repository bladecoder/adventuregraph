The 6×10 bitmap and original glyph order come from
embedded-graphics-unicodefonts 0.2.0 (`src/raw/mono_6x10.data` and
`src/mono_6x10_atlas.rs`), under the accompanying MIT license.
The original font derives from the X.Org misc-fixed fonts.

Block glyphs are replaced or appended using their Unicode subdivisions.
The quadrant, sextant and octant lookup tables are read from the pinned
`tui-big-text` 0.8.10 source; the widget itself is used without modifications.
`banner-symbols.txt` contains every possible emitted block symbol, including
those not reached by the example text. Space is intentionally blank.

Regenerate from the repository root with:

```
python3 scripts/generate-banner-font.py
cargo fmt --all
```

The script needs cached sources for both pinned crates. The Unicode font source
is already a dependency of Mousefood's `fonts` feature. All glyph mappings and
bitmap data in the resulting font are static; initialization allocates nothing.
