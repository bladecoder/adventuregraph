use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use bladeink::{story::Story, story_error::StoryError};
use ratatui::Frame;

use crate::ui;

const CHARS_PER_SECOND: u64 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    SelectNext,
    SelectPrevious,
    SelectIndex(usize),
    ChooseSelected,
    ScrollStory(i32),
    JumpStoryStart,
    JumpStoryEnd,
}

pub struct App {
    story: Story,
    lines: Vec<String>,
    choices: Vec<String>,
    selected: usize,
    finished: bool,
    visible_chars: usize,
    animation_ms: u64,
    scroll: usize,
    total_rows: usize,
    viewport_rows: usize,
    follow: bool,
}

impl App {
    pub fn new(image: &'static [u8], seed: i32) -> Result<Self, StoryError> {
        let story = Story::new_from_image_with_seed(image, seed)?;
        let mut app = Self {
            story,
            lines: Vec::new(),
            choices: Vec::new(),
            selected: 0,
            finished: false,
            visible_chars: 0,
            animation_ms: 0,
            scroll: 0,
            total_rows: 0,
            viewport_rows: 0,
            follow: true,
        };
        app.advance()?;
        Ok(app)
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
    pub fn choices(&self) -> &[String] {
        &self.choices
    }
    pub fn selected_choice(&self) -> Option<usize> {
        (!self.choices.is_empty()).then_some(self.selected)
    }
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    pub fn is_animating(&self) -> bool {
        self.visible_chars < self.text_chars()
    }
    pub fn viewport_rows(&self) -> usize {
        self.viewport_rows
    }
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Advance animation using elapsed milliseconds supplied by the platform.
    pub fn tick(&mut self, elapsed_ms: u32) {
        if !self.is_animating() {
            return;
        }
        self.animation_ms = self.animation_ms.saturating_add(elapsed_ms as u64);
        let chars = self.animation_ms.saturating_mul(CHARS_PER_SECOND) / 1000;
        self.visible_chars = self
            .visible_chars
            .saturating_add(chars as usize)
            .min(self.text_chars());
        self.animation_ms -= chars * 1000 / CHARS_PER_SECOND;
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Result<(), StoryError> {
        if self.is_animating() {
            match event {
                AppEvent::ChooseSelected => {
                    self.skip_line();
                    return Ok(());
                }
                AppEvent::SelectNext | AppEvent::SelectPrevious | AppEvent::SelectIndex(_) => {
                    return Ok(());
                }
                _ => {}
            }
        }
        match event {
            AppEvent::SelectNext => {
                if self.selected + 1 < self.choices.len() {
                    self.selected += 1;
                }
            }
            AppEvent::SelectPrevious => {
                self.selected = self.selected.saturating_sub(1);
            }
            AppEvent::SelectIndex(index) => {
                if index < self.choices.len() {
                    self.selected = index;
                }
            }
            AppEvent::ChooseSelected if self.finished => {
                self.story.reset_state()?;
                self.lines.clear();
                self.choices.clear();
                self.finished = false;
                self.visible_chars = 0;
                self.animation_ms = 0;
                self.scroll = 0;
                self.follow = true;
                self.advance()?;
            }
            AppEvent::ChooseSelected if !self.choices.is_empty() => {
                self.story.choose_choice_index(self.selected)?;
                self.advance()?;
                self.follow = true;
            }
            AppEvent::ChooseSelected => {}
            AppEvent::ScrollStory(delta) => {
                self.follow = false;
                self.scroll = self
                    .scroll
                    .saturating_add_signed(delta as isize)
                    .min(self.max_scroll());
            }
            AppEvent::JumpStoryStart => {
                self.follow = false;
                self.scroll = 0;
            }
            AppEvent::JumpStoryEnd => {
                self.follow = false;
                self.scroll = self.max_scroll();
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        ui::draw(self, frame);
    }

    pub(crate) fn visible_text(&self) -> String {
        let text = self.lines.join("\n\n");
        if self.visible_chars >= self.text_chars() {
            return text;
        }
        let end = text
            .char_indices()
            .nth(self.visible_chars)
            .map_or(text.len(), |(i, _)| i);
        text[..end].to_string()
    }

    pub(crate) fn set_viewport(&mut self, total_rows: usize, viewport_rows: usize) {
        self.total_rows = total_rows;
        self.viewport_rows = viewport_rows;
        if self.follow {
            self.scroll = self.max_scroll();
        } else {
            self.scroll = self.scroll.min(self.max_scroll());
        }
    }

    fn max_scroll(&self) -> usize {
        self.total_rows.saturating_sub(self.viewport_rows)
    }
    fn text_chars(&self) -> usize {
        self.lines
            .iter()
            .map(|line| line.chars().count())
            .sum::<usize>()
            + self.lines.len().saturating_sub(1) * 2
    }

    fn skip_line(&mut self) {
        let text = self.lines.join("\n\n");
        let start = text
            .char_indices()
            .nth(self.visible_chars)
            .map_or(text.len(), |(i, _)| i);
        let end = text[start..]
            .find('\n')
            .map_or(text.len(), |i| start + i + 1);
        self.visible_chars = text[..end].chars().count();
        self.animation_ms = 0;
    }

    fn advance(&mut self) -> Result<(), StoryError> {
        while self.story.can_continue() {
            let line = self.story.cont()?;
            let line = line.trim_end_matches(['\r', '\n']);
            if !line.is_empty() {
                self.lines.push(line.to_string());
            }
        }
        self.choices = self
            .story
            .get_current_choices()
            .iter()
            .map(|c| c.text.clone())
            .collect();
        self.finished = self.choices.is_empty() && !self.story.can_continue();
        if self.finished {
            self.choices.push("-- restart --".to_string());
            if !self
                .lines
                .last()
                .is_some_and(|line| line.trim().eq_ignore_ascii_case("the end"))
            {
                self.lines.push("The End".to_string());
            }
        }
        self.selected = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn render(app: &mut App, width: u16, height: u16) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
    }

    #[test]
    fn intercept_choices_ending_and_restart() {
        let mut app = App::new(crate::STORY_IMAGE, 42).unwrap();
        assert!(!app.lines().is_empty());
        assert!(app.is_animating());
        let initial_lines = app.lines().to_vec();
        app.tick(50);
        assert_eq!(app.visible_chars, 1);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.visible_chars > 1);
        app.tick(1_000_000);
        assert!(!app.is_animating());
        assert!(!app.choices().is_empty());
        if app.choices().len() > 1 {
            app.handle_event(AppEvent::SelectNext).unwrap();
            assert_eq!(app.selected_choice(), Some(1));
            app.handle_event(AppEvent::SelectPrevious).unwrap();
            assert_eq!(app.selected_choice(), Some(0));
        }
        for _ in 0..100 {
            if app.is_finished() {
                break;
            }
            app.handle_event(AppEvent::SelectIndex(0)).unwrap();
            app.handle_event(AppEvent::ChooseSelected).unwrap();
            app.tick(1_000_000);
        }
        assert!(app.is_finished());
        assert_eq!(app.choices(), ["-- restart --"]);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(!app.is_finished());
        assert_eq!(app.lines(), initial_lines);
    }

    #[test]
    fn selection_animation_scroll_and_small_views() {
        let mut app = App::new(crate::STORY_IMAGE, 7).unwrap();
        let first = app.selected_choice();
        app.handle_event(AppEvent::SelectNext).unwrap();
        assert_eq!(app.selected_choice(), first);
        app.tick(1_000_000);
        render(&mut app, 12, 7);
        assert!(app.scroll() > 0);
        app.handle_event(AppEvent::JumpStoryStart).unwrap();
        assert_eq!(app.scroll(), 0);
        app.handle_event(AppEvent::ScrollStory(i32::MAX)).unwrap();
        assert!(app.scroll() > 0);
        app.handle_event(AppEvent::ScrollStory(i32::MIN)).unwrap();
        assert_eq!(app.scroll(), 0);
        for (width, height) in [(1, 1), (2, 2), (4, 4), (8, 5)] {
            render(&mut app, width, height);
        }
    }
}
