use embedded_graphics::{
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::Rgb565,
    primitives::Rectangle,
    text::{Baseline, Text},
};

use embedded_graphics::geometry::*;
use embedded_graphics::prelude::*;

use crate::hardware::agdisplay::AGDisplay;

use super::theme::{BG_COLOR, FG_COLOR, FONT, SEL_BG_COLOR, SEL_FG_COLOR};

const CHOICE_MARGIN: i32 = 2;

struct Line {
    text: String,
    style: Style,
}

impl Line {
    fn new(text: &str, style: Style) -> Self {
        Self {
            text: text.to_owned(),
            style,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Style {
    Text,
    Choice,
    SelectedChoice,
}

pub struct ScrolledText {
    lines: Vec<Line>,
    position: i32, // position in the vector of the first line shown in the display
    text_style: MonoTextStyle<'static, Rgb565>,
    selected_style: MonoTextStyle<'static, Rgb565>,

    queued_lines: Vec<Line>,
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
            text_style,
            selected_style,
            queued_lines: Vec::new(),
            lines_added_since_last_lock: 0,
            choices_idx: Vec::new(),
            selected: 0,
        }
    }

    pub fn get_selected_choice(&self) -> usize {
        self.selected
    }

    pub fn is_scroll_locked(&self, display: &mut AGDisplay<'_>) -> bool {
        self.lines_added_since_last_lock >= self.char_rows(display) - 1
    }

    pub fn is_at_end(&self, display: &mut AGDisplay<'_>) -> bool {
        self.position >= self.lines.len() as i32 - self.char_rows(display)
    }

    pub fn goto_end(&mut self, display: &mut AGDisplay<'_>) {
        let lines = self.lines.len() as i32 - self.position - self.char_rows(display);
        self.scroll(display, lines);
    }

    pub fn add_text(&mut self, display: &mut AGDisplay<'_>, text: &str) {
        self.add_line(display, text, Style::Text);

        println!(
            "Lines added since last lock: {}. Scroll locked: {}. Lines in queue: {}",
            self.lines_added_since_last_lock,
            self.is_scroll_locked(display),
            self.queued_lines.len()
        );

        if self.is_scroll_locked(display) {
            // draw --more text in the last line as a choice
            let choice = vec!["--more--".to_owned()];
            self.add_choices(display, &choice);
        }
    }

    fn add_line(&mut self, display: &mut AGDisplay<'_>, text: &str, style: Style) {
        let margin = match style {
            Style::Text => 0,
            _ => CHOICE_MARGIN,
        };

        // split the text in multiple lines if it is longer than the display width
        let chars_per_line = (self.char_columns(display) - margin) as u32;

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
                        Line::new(&current_str[0..last_space_pos_bytes as usize - 1], style),
                    );
                    // remove the start of the string until the last space found
                    current_str.replace_range(0..last_space_pos_bytes as usize, "");
                    current_line_chars = chars_after_last_space;
                } else {
                    self.add_row(
                        display,
                        Line::new(&current_str[0..chars_per_line as usize], style),
                    );
                    current_str.clear();
                    current_line_chars = 0;
                }

                last_space_pos_bytes = -1;
                chars_after_last_space = 0;
            } else if c == '\n' {
                // if the character is a newline, write the line and reset the counter
                self.add_row(
                    display,
                    Line::new(&current_str[0..current_str.len() - 1], style),
                );
                self.add_row(display, Line::new("", style));
                current_str.clear();
                current_line_chars = 0;
                last_space_pos_bytes = -1;
                chars_after_last_space = 0;
            }
        }

        if current_line_chars > 0 {
            self.add_row(display, Line::new(&current_str, style));
        }
    }

    fn add_row(&mut self, display: &mut AGDisplay<'_>, line: Line) {
        self.lines_added_since_last_lock += 1;

        if self.is_scroll_locked(display) && self.choices_idx.is_empty() {
            self.queued_lines.push(line);
            return;
        }

        self.lines.push(line);
        let display_height = self.char_rows(display);

        if self.lines.len() - self.position as usize > display_height as usize {
            self.scroll(display, 1);
        } else {
            let row = self.lines.len() as i32 - self.position - 1;
            self.print_row(display, self.lines.last().unwrap(), row);
        }
    }

    fn print_text(&self, display: &mut AGDisplay<'_>, line: &Line, row: i32) {
        let char_width = self.text_style.font.character_size.width as i32;

        let style = match line.style {
            Style::SelectedChoice => self.selected_style,
            _ => self.text_style,
        };

        let start_col = match line.style {
            Style::Text => 0,
            _ => CHOICE_MARGIN,
        };

        Text::with_baseline(
            &line.text,
            Point::new(start_col * char_width, self.get_pos_y(display, row)),
            style,
            Baseline::Top,
        )
        .draw(display.get_display())
        .unwrap();
    }

    fn print_row(&self, display: &mut AGDisplay<'_>, line: &Line, row: i32) {
        let cw = self.text_style.font.character_size.width as i32;
        let ch = self.text_style.font.character_size.height as i32;
        let y = self.get_pos_y(display, row);

        let start_col = match line.style {
            Style::Text => 0,
            _ => CHOICE_MARGIN,
        };

        // fill the start of the line with black
        if start_col > 0 {
            let area = Rectangle::new(
                Point::new(0, y),
                Size::new(start_col as u32 * cw as u32, ch as u32),
            );

            display.get_display().fill_solid(&area, BG_COLOR).unwrap();
        }

        // print the text
        self.print_text(display, line, row);

        // fill the rest of the line with black
        let h = self.text_style.font.character_size.height as i32;
        let start_x = (line.text.len() as i32 + start_col) * cw;
        let area = Rectangle::new(
            Point::new(start_x, y),
            Size::new(
                display.get_display().bounding_box().size.width - start_x as u32,
                h as u32,
            ),
        );

        display.get_display().fill_solid(&area, BG_COLOR).unwrap();
    }

    /// SOFTWARE SCROLLING
    #[cfg(feature = "software-scroll")]
    pub fn scroll(&mut self, display: &mut AGDisplay<'_>, nlines: i32) {
        // println!("scrolling...");
        self.position += nlines;

        // enumerate vector elements from the position to the screen height
        for (i, line) in self
            .lines
            .iter()
            .enumerate()
            .skip(self.position as usize)
            .take(self.char_rows(display) as usize)
        {
            let row = i as i32 - self.position;
            println!("row: {}", row);
            self.print_row(display, line, row);
        }
    }

    /// HARDWARE SCROLLING
    #[cfg(not(feature = "software-scroll"))]
    pub fn scroll(&mut self, display: &mut AGDisplay<'_>, nlines: i32) {
        // println!("scrolling...");
        self.position += nlines;

        let h = self.text_style.font.character_size.height as i32;
        let char_rows = self.char_rows(display);

        let orientation = display.get_display().orientation();

        let offset = if orientation.rotation == Rotation::Deg0 {
            ((self.position % char_rows) * h) as u16
        } else {
            display.get_display().bounding_box().size.height as u16
                - 1
                - ((self.position % char_rows) * h) as u16
        };

        display
            .get_display()
            .set_vertical_scroll_offset(offset)
            .unwrap();

        // print the new lines
        let (start_row, absnlines) = if nlines > 0 {
            (self.position + self.char_rows(display) - nlines, nlines)
        } else {
            (self.position, -nlines)
        };

        for (i, line) in self
            .lines
            .iter()
            .enumerate()
            .skip(start_row as usize)
            .take(absnlines as usize)
        {
            let row = i as i32 - self.position;
            self.print_row(display, line, row);
        }
    }

    #[cfg(feature = "software-scroll")]
    fn get_pos_y(&self, _display: &mut AGDisplay<'_>, row: i32) -> i32 {
        let h = self.text_style.font.character_size.height as i32;

        row * h
    }

    #[cfg(not(feature = "software-scroll"))]
    fn get_pos_y(&self, display: &mut AGDisplay<'_>, row: i32) -> i32 {
        let h = self.text_style.font.character_size.height as i32;
        let max_display_rows = self.char_rows(display);
        let row_with_scroll = (row + self.position) % max_display_rows;

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
        self.choices_idx.clear();
        self.selected = 0;
        self.lines_added_since_last_lock = 0;
        self.queued_lines.clear();
        display.get_display().clear(BG_COLOR).unwrap();

        // reset the vertical scroll offset
        display.get_display().set_vertical_scroll_offset(0).unwrap();
    }

    pub fn add_choices(&mut self, display: &mut AGDisplay<'_>, choices: &Vec<String>) {
        self.choices_idx.clear();
        self.selected = 0;

        for choice in choices {
            self.choices_idx.push(self.lines.len());
            self.add_line(display, choice, Style::Choice);
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

    pub fn up(&mut self, display: &mut AGDisplay<'_>) {
        // if selected is 0, scroll up
        if self.selected == 0 && self.position > 0 {
            self.scroll(display, -1);
            return;
        }

        if self.selected == 0 {
            return;
        }

        self.select_choice(display, self.selected - 1);
    }

    pub fn down(&mut self, display: &mut AGDisplay<'_>) {
        // if we have scrolled up, scroll down
        if self.position < self.lines.len() as i32 - self.char_rows(display) {
            self.scroll(display, 1);
            return;
        }

        if self.selected == self.choices_idx.len() - 1 {
            return;
        }

        self.select_choice(display, self.selected + 1);
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
            let text = &mut self.lines[selected_idx + i as usize];
            text.style = Style::Choice;
            let text = &self.lines[selected_idx + i as usize];
            self.print_text(display, text, row);
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
            let text = &mut self.lines[selected_idx + i as usize];
            text.style = Style::SelectedChoice;
            let text = &self.lines[selected_idx + i as usize];
            self.print_text(display, text, row);
        }
    }

    #[cfg(feature = "software-scroll")]
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

        display.get_display().fill_solid(&area, BG_COLOR).unwrap();
    }

    #[cfg(not(feature = "software-scroll"))]
    pub fn clear_rows(&self, display: &mut AGDisplay<'_>, row: i32, num_rows: i32) {
        let h = self.text_style.font.character_size.height as i32;

        // clear the rows from row to row + num_rows
        for i in 0..num_rows {
            let y = self.get_pos_y(display, row + i);

            let area = Rectangle::new(
                Point::new(0, y),
                Size::new(display.get_display().bounding_box().size.width, h as u32),
            );

            display.get_display().fill_solid(&area, BG_COLOR).unwrap();
        }
    }

    pub fn more(&mut self, display: &mut AGDisplay<'_>) {
        self.clear_choices(display);

        // extract the queued lines and add them to the display
        let queued_lines = std::mem::take(&mut self.queued_lines);

        for line in queued_lines {
            self.add_row(display, line);
        }
    }
}
