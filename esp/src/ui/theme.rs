use embedded_graphics::{
    mono_font::iso_8859_15::{FONT_6X10, FONT_9X15, FONT_9X18_BOLD},
    pixelcolor::Rgb565,
    prelude::RgbColor,
};

#[cfg(feature = "display-st7735")]
pub const FONT: &embedded_graphics::mono_font::MonoFont<'_> = &FONT_6X10;

#[cfg(not(feature = "display-st7735"))]
pub const FONT: &embedded_graphics::mono_font::MonoFont<'_> = &FONT_9X15;

#[cfg(feature = "display-st7735")]
pub const TITLE_FONT: &embedded_graphics::mono_font::MonoFont<'_> = &FONT_9X18_BOLD;

#[cfg(not(feature = "display-st7735"))]
pub const TITLE_FONT: &embedded_graphics::mono_font::MonoFont<'_> = &FONT_9X18_BOLD;

pub const FG_COLOR: Rgb565 = Rgb565::GREEN;
pub const BG_COLOR: Rgb565 = Rgb565::BLACK;
pub const SEL_FG_COLOR: Rgb565 = Rgb565::BLACK;
pub const SEL_BG_COLOR: Rgb565 = Rgb565::GREEN;
