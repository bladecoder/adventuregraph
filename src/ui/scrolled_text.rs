use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{Rgb565, RgbColor},
    primitives::Rectangle,
    text::{Baseline, Text},
};

use embedded_graphics::geometry::*;
use embedded_graphics::prelude::*;

use crate::hardware::agdisplay::AGDisplay;

use super::theme::{BG_COLOR, FG_COLOR, FONT, SEL_BG_COLOR, SEL_FG_COLOR};

const CHOICE_MARGIN: i32 = 2;

struct Line {
    start_col: i32,
    text: String,
}

pub struct ScrolledText {
    lines: Vec<Line>,
    position: i32,    // position in the vector of the first line shown in the display
    scroll_line: i32, // this is the line in memory showed as last line in the display
    text_style: MonoTextStyle<'static, Rgb565>,
    selected_style: MonoTextStyle<'static, Rgb565>,

    lines_added_since_last_lock: i32,

    choices_idx: Vec<usize>,
    selected: usize,
}

impl ScrolledText {
    pub fn new() -> Self {
        let text_style = MonoTextStyleBuilder::new()
            .font(FONT)
            .text_color(FG_COLOR)
            .background_color(BG_COLOR)
            .build();

        let selected_style = MonoTextStyleBuilder::new()
            .font(FONT)
            .text_color(SEL_FG_COLOR)
            .background_color(SEL_BG_COLOR)
            .build();

        Self {
            lines: Vec::new(),
            position: 0,
            scroll_line: 0,
            text_style,
            selected_style,
            lines_added_since_last_lock: 0,
            choices_idx: Vec::new(),
            selected: 0,
        }
    }

    pub fn is_scroll_locked(&self, display: &mut AGDisplay<'_>) -> bool {
        self.lines_added_since_last_lock >= self.char_rows(display) - 2
    }

    pub fn add_text(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        let prev_lines = self.lines.len() as i32;
        self.add_string(display, text, 0);
        self.lines_added_since_last_lock += self.lines.len() as i32 - prev_lines;

        println!(
            "Lines added since last lock: {}. Scroll locked: {}",
            self.lines_added_since_last_lock,
            self.is_scroll_locked(display)
        );

        if self.is_scroll_locked(display) {
            // draw --more text in the last line as a choice
            let choice = vec!["--more--".to_owned()];
            self.add_choices(display, &choice);
        }
    }

    fn add_string(&mut self, display: &mut AGDisplay<'_>, text: &str, start_row: i32) {
        // split the text in multiple lines if it is longer than the display width
        let chars_per_line = (self.char_columns(display) - start_row) as u32;

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
                    self.add_row(
                        display,
                        &current_str[0..last_space_pos_bytes as usize - 1],
                        start_row,
                    );
                    // remove the start of the string until the last space found
                    current_str.replace_range(0..last_space_pos_bytes as usize, "");
                    current_line_chars = chars_after_last_space;
                } else {
                    self.add_row(display, &current_str[0..chars_per_line as usize], start_row);
                    current_str.clear();
                    current_line_chars = 0;
                }

                last_space_pos_bytes = -1;
                chars_after_last_space = 0;
            } else if c == '\n' {
                // if the character is a newline, write the line and reset the counter
                self.add_row(display, &current_str[0..current_str.len() - 1], start_row);
                self.add_row(display, "", start_row);
                current_str.clear();
                current_line_chars = 0;
                last_space_pos_bytes = -1;
                chars_after_last_space = 0;
            }
        }

        if current_line_chars > 0 {
            self.add_row(display, &current_str, start_row);
        }
    }

    fn add_row(&mut self, display: &mut AGDisplay<'_>, text: &str, start_col: i32) {
        self.lines.push(Line {
            start_col,
            text: text.to_owned(),
        });
        let display_height = self.char_rows(display);

        if self.lines.len() - self.position as usize > display_height as usize {
            self.scroll(display);
        }

        // draw the text in the display at the end of the list
        let row = self.lines.len() as i32 - self.position - 1;
        self.print_row(display, self.lines.last().unwrap(), row, false);
    }

    fn print_row(&self, display: &mut AGDisplay<'_>, line: &Line, row: i32, selected: bool) {
        let char_width = self.text_style.font.character_size.width as i32;

        let style = if selected {
            self.selected_style
        } else {
            self.text_style
        };

        Text::with_baseline(
            &line.text,
            Point::new(line.start_col * char_width, self.get_pos_y(display, row)),
            style,
            Baseline::Top,
        )
        .draw(display.get_display())
        .unwrap();
    }

    /// SOFTWARE SCROLLING
    pub fn scroll(&mut self, display: &mut AGDisplay<'_>) {
        // println!("scrolling...");
        self.position += 1;

        // enumerate vector elements from the position to the end
        for (i, line) in self.lines.iter().enumerate().skip(self.position as usize) {
            let row = i as i32 - self.position;

            self.print_row(display, line, row, false);

            // fill the rest of the line with black
            let h = self.text_style.font.character_size.height as i32;
            let y = self.get_pos_y(display, row);
            let start_x = line.text.len() as i32 * self.text_style.font.character_size.width as i32;
            let area = Rectangle::new(
                Point::new(start_x, y),
                Size::new(
                    display.get_display().bounding_box().size.width - start_x as u32,
                    h as u32,
                ),
            );

            display.get_display().fill_solid(&area, BG_COLOR).unwrap();
        }
    }

    /// HARDWARE SCROLLING
    pub fn scroll_hw(&mut self, display: &mut AGDisplay<'_>) {
        // println!("scrolling...");
        self.position += 1;
        self.scroll_line += 1;

        let h = self.text_style.font.character_size.height as i32;
        let offset = (h as u16 * self.scroll_line as u16)
            % display.get_display().bounding_box().size.height as u16;

        display
            .get_display()
            .set_vertical_scroll_offset(offset)
            .unwrap();

        // clear last line
        let last_screen_row = self.char_rows(display);
        self.clear_rows(display, last_screen_row - 1, 1);
    }

    fn get_pos_y(&self, display: &mut AGDisplay<'_>, row: i32) -> i32 {
        let h = self.text_style.font.character_size.height as i32;
        let max_display_rows = self.char_rows(display);
        let row_with_scroll = (row + self.scroll_line) % max_display_rows;

        row_with_scroll * h
    }

    fn char_rows(&self, display: &mut AGDisplay<'_>) -> i32 {
        let h = self.text_style.font.character_size.height as i32;
        display.get_display().bounding_box().size.height as i32 / h
    }

    fn char_columns(&self, display: &mut AGDisplay<'_>) -> i32 {
        let w = self.text_style.font.character_size.width as i32;
        display.get_display().bounding_box().size.width as i32 / w
    }

    pub fn clear(&mut self, display: &mut AGDisplay<'_>) {
        self.lines.clear();
        self.position = 0;
        self.scroll_line = 0;
        self.choices_idx.clear();
        self.selected = 0;
        self.lines_added_since_last_lock = 0;
        display.get_display().clear(BG_COLOR).unwrap();

        // reset the vertical scroll offset
        display.get_display().set_vertical_scroll_offset(0).unwrap();
    }

    pub fn add_choices(&mut self, display: &mut AGDisplay<'_>, choices: &Vec<String>) {
        self.choices_idx.clear();
        self.selected = 0;

        for choice in choices {
            self.choices_idx.push(self.lines.len());
            self.add_string(display, choice, CHOICE_MARGIN);
            println!("-> {}", choice);
        }

        self.select_choice(display, 0);
    }

    pub fn clear_choices(&mut self, display: &mut AGDisplay<'_>) {
        let first_choice_line = self.choices_idx[0] as i32;
        let num_lines = self.lines.len() as i32 - first_choice_line;

        self.clear_rows(display, first_choice_line - self.position, num_lines);

        self.choices_idx.clear();
        self.selected = 0;

        // delete last num_lines from the text vector
        self.lines.truncate(first_choice_line as usize);

        // reset lock counter
        self.lines_added_since_last_lock = 0;
    }

    pub fn select_choice(&mut self, display: &mut AGDisplay<'_>, to_select: usize) {
        // Restore previous selected text style
        let selected_idx = self.choices_idx[self.selected];
        let row = selected_idx as i32 - self.position;
        let num_rows = if self.selected == self.choices_idx.len() - 1 {
            self.lines.len() as i32 - selected_idx as i32
        } else {
            self.choices_idx[self.selected + 1] as i32 - selected_idx as i32
        };

        for i in 0..num_rows {
            let row = row + i;
            let text = &self.lines[selected_idx + i as usize];
            self.print_row(display, text, row, false);
        }

        // print selected text
        self.selected = to_select;
        let selected_idx = self.choices_idx[self.selected];
        let row = selected_idx as i32 - self.position;
        let num_rows = if self.selected == self.choices_idx.len() - 1 {
            self.lines.len() as i32 - selected_idx as i32
        } else {
            self.choices_idx[self.selected + 1] as i32 - selected_idx as i32
        };

        for i in 0..num_rows {
            let row = row + i;
            let text = &self.lines[selected_idx + i as usize];
            self.print_row(display, text, row, true);
        }
    }

    pub fn clear_rows(&self, display: &mut AGDisplay<'_>, row: i32, num_rows: i32) {
        let y = self.get_pos_y(display, row);
        let h = self.text_style.font.character_size.height as i32;

        let area = Rectangle::new(
            Point::new(0, y),
            Size::new(
                display.get_display().bounding_box().size.width,
                (h * num_rows) as u32,
            ),
        );

        display
            .get_display()
            .fill_solid(&area, Rgb565::BLACK)
            .unwrap();
    }

    pub fn more(&mut self, display: &mut AGDisplay<'_>) {
        //self.lines_added_since_last_lock = 0;
        self.clear_choices(display)
    }

    pub fn test_scroll(&mut self, agdisplay: &mut AGDisplay) {
        //test scroll display: write 40 lines
        for i in 0..100 {
            self.add_text(agdisplay, &format!("Line {}\n", i));
        }
    }
}
