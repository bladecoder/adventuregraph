use embedded_graphics::{
    mono_font::{iso_8859_15::FONT_6X10, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    text::Text,
};

use embedded_graphics::geometry::*;
use embedded_graphics::prelude::*;

use crate::hardware::agdisplay_st7735::AGDisplay;

pub struct ScrolledText {
    text: Vec<String>,
    position: i32,
    character_style: MonoTextStyle<'static, Rgb565>,
}

impl ScrolledText {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            position: 0,
            character_style: MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN),
        }
    }

    pub fn add_text(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        self.text.push(text.to_string());

        let h = self.character_style.font.character_size.height as i32;

        // draw the text in the display at the end of the list
        let display = display.get_display();
        let y = (self.text.len() - 1) as i32 * h + h;
        Text::new(text, Point::new(0, y), self.character_style)
            .draw(display)
            .unwrap();
    }

    pub fn scroll(&mut self) {
        self.position += 1;
        if self.position >= self.text.len() as i32 {
            self.position = 0;
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }
}
