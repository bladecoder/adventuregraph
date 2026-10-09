use super::*;
use embedded_graphics::{mono_font::mapping::GlyphMapping, prelude::*};
#[test]
fn every_native_block_symbol_and_existing_unicode_glyph_is_present() {
    for ch in include_str!("../assets/banner-symbols.txt").chars() {
        let index = MAPPING.index(ch);
        assert!(ch == ' ' || index != 0, "Missing {ch}");
        assert!(index < (FONT.image.size().height / 10 * 16) as usize);
    }
    for ch in "Abáéñ┌┘─│→".chars() {
        assert_ne!(MAPPING.index(ch), 0, "Missing {ch}");
    }
}
