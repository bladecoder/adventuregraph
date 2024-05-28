use embedded_graphics::{
    mono_font::{iso_8859_15::FONT_6X10, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    text::Text,
};

use embedded_graphics::geometry::*;
use embedded_graphics::prelude::*;

use crate::hardware::agdisplay_st7789::AGDisplay;

pub struct ScrolledText {
    text: Vec<String>,
    position: i32,
    character_style: MonoTextStyle<'static, Rgb565>,

    choices_pos_y: Vec<i32>,
    selected: usize,
}

impl ScrolledText {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            position: 0,
            character_style: MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN),
            choices_pos_y: Vec::new(),
            selected: 0,
        }
    }

    pub fn add_text(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        let h = self.character_style.font.character_size.height as i32;
        let w = self.character_style.font.character_size.width as i32;

        // split the text in multiple lines if it is longer than the display width
        let chars_per_line = display.get_display().bounding_box().size.width / w as u32;

        if text.len() > chars_per_line as usize {
            self.add_text(display, &text[0..chars_per_line as usize]);
            self.add_text(display, &text[chars_per_line as usize..]);
            return;
        }

        self.text.push(text.to_string());
        let display_height = display.get_display().bounding_box().size.height;

        if self.text.len() >= display_height as usize {
            self.scroll(display);
        }

        let d = display.get_display();

        // draw the text in the display at the end of the list
        let y = (self.text.len() - 1) as i32 * h + h;

        Text::new(text, Point::new(0, y), self.character_style)
            .draw(d)
            .unwrap();
    }

    pub fn scroll(&mut self, display: &mut AGDisplay<'_>) {
        self.position += 1;

        display.get_display().clear(Rgb565::BLACK).unwrap();
        let h = self.character_style.font.character_size.height as i32;

        // enumerate vector elements from the position to the end
        for (i, text) in self.text.iter().enumerate().skip(self.position as usize) {
            let y = i as i32 * h + h;

            Text::new(text, Point::new(0, y), self.character_style)
                .draw(display.get_display())
                .unwrap();
        }
    }

    pub fn clear(&mut self, display: &mut AGDisplay<'_>) {
        self.text.clear();
        self.position = 0;
        self.choices_pos_y.clear();
        display.get_display().clear(Rgb565::BLACK).unwrap();
    }

    pub fn add_choices(&mut self, display: &mut AGDisplay<'_>, choices: &Vec<String>) {
        self.choices_pos_y.clear();

        for choice in choices {
            self.add_text(display, &format!("  {}", choice));
            self.choices_pos_y.push((self.text.len() - 1) as i32);
        }

        self.select_choice(display, 0);
    }

    pub fn select_choice(&mut self, display: &mut AGDisplay<'_>, selected: usize) {
        // replace the character 0 from the selected choice with a '>'
        self.text
            .get_mut(self.selected)
            .unwrap()
            .replace_range(0..1, " ");

        let h = self.character_style.font.character_size.height as i32;
        let y = self.choices_pos_y[self.selected] * h + h;

        Text::new(" ", Point::new(0, y), self.character_style)
            .draw(display.get_display())
            .unwrap();

        self.selected = selected;

        self.text
            .get_mut(self.selected)
            .unwrap()
            .replace_range(0..1, ">");

        let y = self.choices_pos_y[self.selected] * h + h;

        Text::new(">", Point::new(0, y), self.character_style)
            .draw(display.get_display())
            .unwrap();
    }
}
