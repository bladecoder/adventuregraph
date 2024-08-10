use embedded_graphics::{
    mono_font::{iso_8859_15::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{Rgb565, RgbColor},
    primitives::Rectangle,
    text::{Baseline, Text},
};

use embedded_graphics::geometry::*;
use embedded_graphics::prelude::*;

use crate::hardware::agdisplay::AGDisplay;

pub struct ScrolledText {
    text: Vec<String>,
    position: i32,    // position in the vector of the first line shown in the display
    scroll_line: i32, // this is the line in memory showed as last line in the display
    text_style: MonoTextStyle<'static, Rgb565>,
    selected_style: MonoTextStyle<'static, Rgb565>,

    choices_idx: Vec<i32>,
    selected: usize,
}

impl ScrolledText {
    pub fn new() -> Self {
        let selected_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(Rgb565::BLACK)
            .background_color(Rgb565::GREEN)
            .build();

        Self {
            text: Vec::new(),
            position: 0,
            scroll_line: 0,
            text_style: MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN),
            selected_style,
            choices_idx: Vec::new(),
            selected: 0,
        }
    }

    pub fn add_text(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        let w = self.text_style.font.character_size.width as i32;

        // split the text in multiple lines if it is longer than the display width
        let chars_per_line = display.get_display().bounding_box().size.width / w as u32;

        let mut current_line_chars = 0;
        let mut last_space_pos_bytes = -1;
        let mut current_str = String::with_capacity(chars_per_line as usize);
        let mut chars_after_last_space = 0;

        for c in text.chars() {
            current_line_chars += 1;
            current_str.push(c);
            chars_after_last_space += 1;

            if c.is_whitespace() {
                last_space_pos_bytes = current_str.len() as i32;
                chars_after_last_space = 0;
            }

            if current_line_chars == chars_per_line as i32 {
                if last_space_pos_bytes != -1 {
                    self.write_cut_line(
                        display,
                        &current_str[0..last_space_pos_bytes as usize - 1],
                    );
                    // remove the start of the string until the last space found
                    current_str.replace_range(0..last_space_pos_bytes as usize, "");
                    current_line_chars = chars_after_last_space;
                } else {
                    self.write_cut_line(display, &current_str[0..chars_per_line as usize]);
                    current_str.clear();
                    current_line_chars = 0;
                }

                last_space_pos_bytes = -1;
                chars_after_last_space = 0;
            } else if c == '\n' {
                // if the character is a newline, write the line and reset the counter
                self.write_cut_line(display, &current_str[0..current_str.len() - 1]);
                self.write_cut_line(display, "");
                current_str.clear();
                current_line_chars = 0;
                last_space_pos_bytes = -1;
                chars_after_last_space = 0;
            }
        }

        if current_line_chars > 0 {
            self.write_cut_line(display, &current_str);
        }
    }

    fn write_cut_line(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        self.text.push(text.to_string());
        let display_height = self.get_display_rows(display);

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
            self.text_style,
            Baseline::Top,
        )
        .draw(display.get_display())
        .unwrap();
    }

    pub fn scroll(&mut self, display: &mut AGDisplay<'_>) {
        // println!("scrolling...");
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
        let h = self.text_style.font.character_size.height as i32;
        let offset = (h as u16 * self.position as u16)
            % display.get_display().bounding_box().size.height as u16;

        display
            .get_display()
            .set_vertical_scroll_offset(offset)
            .unwrap();

        // clear last line
        let last_screen_row = self.get_display_rows(display);
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
        let h = self.text_style.font.character_size.height as i32;
        let max_display_rows = self.get_display_rows(display);
        let row_with_scroll = (row + self.scroll_line) % max_display_rows;

        row_with_scroll * h
    }

    fn get_display_rows(&self, display: &mut AGDisplay<'_>) -> i32 {
        let h = self.text_style.font.character_size.height as i32;
        display.get_display().bounding_box().size.height as i32 / h
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
            println!("-> {}", choice);
        }

        self.select_choice(display, 0);
    }

    pub fn select_choice(&mut self, display: &mut AGDisplay<'_>, to_select: usize) {
        let selected_idx = self.choices_idx[self.selected as usize];

        // replace the character 0 from the selected choice with a '>'
        self.text
            .get_mut(selected_idx as usize)
            .unwrap()
            .replace_range(0..1, " ");

        let w = self.text_style.font.character_size.width as i32;
        let h = self.text_style.font.character_size.height as i32;

        let y = self.get_pos_y(display, self.choices_idx[self.selected] - self.position);

        let area = Rectangle::new(Point::new(0, y), Size::new(w as u32, h as u32));

        display
            .get_display()
            .fill_solid(&area, Rgb565::BLACK)
            .unwrap();

        self.selected = to_select;

        let selected_idx = self.choices_idx[self.selected as usize];

        // print selected text
        self.text
            .get_mut(selected_idx as usize)
            .unwrap()
            .replace_range(0..1, ">");

        let y = self.get_pos_y(display, self.choices_idx[self.selected] - self.position);

        Text::with_baseline(">", Point::new(0, y), self.text_style, Baseline::Top)
            .draw(display.get_display())
            .unwrap();
    }

    pub fn test_scroll(&mut self, agdisplay: &mut AGDisplay) {
        //test scroll display: write 40 lines
        for i in 0..100 {
            self.add_text(agdisplay, &format!("Line {}\n", i));
        }
    }
}
