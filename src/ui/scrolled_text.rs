use embedded_graphics::{
    mono_font::{iso_8859_15::FONT_6X10, MonoTextStyle},
    pixelcolor::{Rgb565, RgbColor},
    primitives::Rectangle,
    text::{Baseline, Text},
};

use embedded_graphics::geometry::*;
use embedded_graphics::prelude::*;

use crate::hardware::agdisplay_st7789::AGDisplay;

pub struct ScrolledText {
    text: Vec<String>,
    position: i32,    // position in the vector of the first line shown in the display
    scroll_line: i32, // this is the line in memory showed as last line in the display
    character_style: MonoTextStyle<'static, Rgb565>,

    choices_idx: Vec<i32>,
    selected: usize,
}

impl ScrolledText {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            position: 0,
            scroll_line: 0,
            character_style: MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN),
            choices_idx: Vec::new(),
            selected: 0,
        }
    }

    pub fn add_text(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        let h = self.character_style.font.character_size.height as i32;
        let w = self.character_style.font.character_size.width as i32;

        // split the text in multiple lines if it is longer than the display width
        let chars_per_line = display.get_display().bounding_box().size.width / w as u32;

        if text.chars().count() > chars_per_line as usize {
            // split the text in two parts. Be careful with the utf8 boundaries
            let s1 = text
                .chars()
                .take(chars_per_line as usize)
                .collect::<String>();
            let s2 = text
                .chars()
                .skip(chars_per_line as usize)
                .collect::<String>();

            self.add_text(display, &s1);
            self.add_text(display, &s2);

            return;
        }

        self.text.push(text.to_string());
        let display_height = display.get_display().bounding_box().size.height / h as u32;

        if self.text.len() - self.position as usize > display_height as usize {
            self.scroll(display);
        }

        // draw the text in the display at the end of the list
        Text::with_baseline(
            text,
            Point::new(
                0,
                self.get_pos_y(display, self.text.len() as i32 - self.position - 1),
            ),
            self.character_style,
            Baseline::Top,
        )
        .draw(display.get_display())
        .unwrap();
    }

    pub fn scroll(&mut self, display: &mut AGDisplay<'_>) {
        println!("scrolling...");
        self.position += 1;
        self.scroll_line += 1;

        // SOFTWARE SCROLLING
        // display.get_display().clear(Rgb565::BLACK).unwrap();

        // // enumerate vector elements from the position to the end
        // for (i, text) in self.text.iter().enumerate().skip(self.position as usize) {
        //     Text::with_baseline(
        //         text,
        //         Point::new(0, self.get_pos_y(i as i32 - self.position)),
        //         self.character_style,
        //         Baseline::Top,
        //     )
        //     .draw(display.get_display())
        //     .unwrap();
        // }

        // HARDWARE SCROLLING
        let h = self.character_style.font.character_size.height as i32;
        let offset = (h as u16 * self.position as u16)
            % display.get_display().bounding_box().size.height as u16;

        display
            .get_display()
            .set_vertical_scroll_offset(offset)
            .unwrap();

        // clear last line
        let last_screen_row = display.get_display().bounding_box().size.height as i32 / h;
        let y = self.get_pos_y(display, last_screen_row - 1);
        let area = Rectangle::new(
            Point::new(0, y),
            Size::new(display.get_display().bounding_box().size.width, h as u32),
        );

        display
            .get_display()
            .fill_solid(&area, Rgb565::BLACK)
            .unwrap();
    }

    fn get_pos_y(&self, display: &mut AGDisplay<'_>, row: i32) -> i32 {
        let h = self.character_style.font.character_size.height as i32;
        let max_display_rows = display.get_display().bounding_box().size.height as i32 / h;
        let row_with_scroll = (row + self.scroll_line) % max_display_rows;

        row_with_scroll * h
    }

    pub fn clear(&mut self, display: &mut AGDisplay<'_>) {
        self.text.clear();
        self.position = 0;
        self.scroll_line = 0;
        self.choices_idx.clear();
        self.selected = 0;
        display.get_display().clear(Rgb565::BLACK).unwrap();

        // reset the vertical scroll offset
        display.get_display().set_vertical_scroll_offset(0).unwrap();
    }

    pub fn add_choices(&mut self, display: &mut AGDisplay<'_>, choices: &Vec<String>) {
        self.choices_idx.clear();
        self.selected = 0;

        for choice in choices {
            self.choices_idx.push((self.text.len()) as i32);
            self.add_text(display, &format!("  {}", choice));
        }

        self.select_choice(display, 0);
    }

    pub fn select_choice(&mut self, display: &mut AGDisplay<'_>, to_select: usize) {
        // replace the character 0 from the selected choice with a '>'
        self.text
            .get_mut(self.selected)
            .unwrap()
            .replace_range(0..1, " ");

        let w = self.character_style.font.character_size.width as i32;
        let h = self.character_style.font.character_size.height as i32;

        let y = self.get_pos_y(display, self.choices_idx[self.selected] - self.position);

        let area = Rectangle::new(Point::new(0, y), Size::new(w as u32, h as u32));

        display
            .get_display()
            .fill_solid(&area, Rgb565::BLACK)
            .unwrap();

        self.selected = to_select;

        self.text
            .get_mut(self.selected)
            .unwrap()
            .replace_range(0..1, ">");

        let y = self.get_pos_y(display, self.choices_idx[self.selected] - self.position);

        Text::with_baseline(">", Point::new(0, y), self.character_style, Baseline::Top)
            .draw(display.get_display())
            .unwrap();
    }
}
