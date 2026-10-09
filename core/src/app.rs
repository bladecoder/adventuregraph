use alloc::{
    collections::VecDeque,
    format,
    string::{String, ToString},
    vec::Vec,
};
use bladeink::{story::Story, story_error::StoryError};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
};

use crate::ui;
#[cfg(test)]
use ratatui::text::{Line, Span, Text};
use tui_big_text::PixelSize;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryLine {
    pub text: String,
    pub style: Style,
    pub alignment: Alignment,
    pub banner: Option<PixelSize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Settings {
    style: Style,
    text_animation: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OutputEvent {
    Text(StoryLine),
    TextAnimation(StoryLine, bool),
    Clear,
    Set(Settings),
    Wait(u64),
}

fn parse_color(value: &str, content: &str) -> Result<Color, StoryError> {
    let value = value.trim();
    let color = if value.len() == 6 && value.bytes().all(|b| b.is_ascii_hexdigit()) {
        let rgb = u32::from_str_radix(value, 16).unwrap();
        Some(Color::Rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8))
    } else {
        value
            .parse::<Color>()
            .ok()
            .filter(|c| !matches!(c, Color::Indexed(_) | Color::Rgb(..)))
    };
    color.ok_or_else(|| StoryError::InvalidStoryState(format!("Invalid color: {content}")))
}

fn parse_command(command: &str, content: &str) -> Result<OutputEvent, StoryError> {
    let mut tokens = command.split_whitespace();
    let name = tokens.next().unwrap_or("");
    match name {
        "cls" if tokens.next().is_none() => Ok(OutputEvent::Clear),
        "set" => {
            let mut style = Style::default();
            let mut text_animation = None;
            let mut has_attributes = false;
            for attribute in tokens {
                let (name, value) = attribute
                    .split_once('=')
                    .filter(|(name, value)| !name.is_empty() && !value.is_empty())
                    .ok_or_else(|| {
                        StoryError::InvalidStoryState(format!(
                            "Invalid attribute '{attribute}' in {content}"
                        ))
                    })?;
                match name {
                    "defaultcolor" => style = style.fg(parse_color(value, attribute)?),
                    "defaultbgcolor" => style = style.bg(parse_color(value, attribute)?),
                    "text-animation" => {
                        text_animation = Some(match value {
                            "true" => true,
                            "false" => false,
                            _ => {
                                return Err(StoryError::InvalidStoryState(format!(
                                    "Invalid boolean: {attribute}"
                                )));
                            }
                        });
                    }
                    _ => {
                        return Err(StoryError::InvalidStoryState(format!(
                            "Unknown attribute '{attribute}' in {content}"
                        )));
                    }
                }
                has_attributes = true;
            }
            if !has_attributes {
                return Err(StoryError::InvalidStoryState(format!(
                    "Missing attributes: {content}"
                )));
            }
            Ok(OutputEvent::Set(Settings {
                style,
                text_animation,
            }))
        }
        "wait" => {
            let mut duration = None;
            for attribute in tokens {
                let value = attribute
                    .strip_prefix("time=")
                    .filter(|value| !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()));
                let milliseconds = value
                    .and_then(|value| value.parse::<u64>().ok())
                    .and_then(|seconds| seconds.checked_mul(1000))
                    .ok_or_else(|| {
                        StoryError::InvalidStoryState(format!(
                            "Invalid wait attribute '{attribute}' in {content}"
                        ))
                    })?;
                duration = Some(milliseconds);
            }
            duration.map(OutputEvent::Wait).ok_or_else(|| {
                StoryError::InvalidStoryState(format!("Missing wait time: {content}"))
            })
        }
        _ => Err(StoryError::InvalidStoryState(format!(
            "Unknown command or unsupported attributes: {content}"
        ))),
    }
}

fn parse_output(text: &str, tags: &[String]) -> Result<Option<OutputEvent>, StoryError> {
    let mut style = Style::default();
    let mut alignment = Alignment::Left;
    let mut banner = None;
    let mut text_animation = None;
    for tag in tags {
        let tag = tag.trim();
        let (name, value) = tag.split_once(':').unwrap_or((tag, ""));
        match name.trim() {
            "color" => style = style.fg(parse_color(value, tag)?),
            "bgcolor" => style = style.bg(parse_color(value, tag)?),
            "banner" => {
                banner = Some(match value.trim() {
                    "full" => PixelSize::Full,
                    "half-height" => PixelSize::HalfHeight,
                    "half-width" => PixelSize::HalfWidth,
                    "quadrant" => PixelSize::Quadrant,
                    "third-height" => PixelSize::ThirdHeight,
                    "sextant" => PixelSize::Sextant,
                    "quarter-height" => PixelSize::QuarterHeight,
                    "octant" => PixelSize::Octant,
                    _ => {
                        return Err(StoryError::InvalidStoryState(format!(
                            "Invalid banner tag: {tag}"
                        )));
                    }
                });
            }
            "align" => {
                alignment = match value.trim() {
                    "center" => Alignment::Center,
                    "right" => Alignment::Right,
                    "left" => Alignment::Left,
                    _ => {
                        return Err(StoryError::InvalidStoryState(format!(
                            "Invalid alignment tag: {tag}"
                        )));
                    }
                };
            }
            "text-animation" => {
                text_animation = Some(match value.trim() {
                    "true" => true,
                    "false" => false,
                    _ => {
                        return Err(StoryError::InvalidStoryState(format!(
                            "Invalid text animation tag: {tag}"
                        )));
                    }
                });
            }
            _ => {}
        }
    }
    let text = text.trim_end_matches(['\r', '\n']);
    if let Some(command) = text.strip_prefix('>') {
        return parse_command(command, text).map(Some);
    }
    Ok((!text.is_empty()).then(|| {
        let line = StoryLine {
            text: text.to_string(),
            style,
            alignment,
            banner,
        };
        match text_animation {
            Some(animated) => OutputEvent::TextAnimation(line, animated),
            None => OutputEvent::Text(line),
        }
    }))
}

pub struct App {
    story: Story,
    default_style: Style,
    text_animation: bool,
    wait_ms: u64,
    lines: Vec<StoryLine>,
    pending: VecDeque<OutputEvent>,
    pending_choices: Vec<String>,
    pending_finished: bool,
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
            default_style: Style::default(),
            text_animation: true,
            wait_ms: 0,
            lines: Vec::new(),
            pending: VecDeque::new(),
            pending_choices: Vec::new(),
            pending_finished: false,
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

    pub fn lines(&self) -> &[StoryLine] {
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
    /// Whether an Ink wait is delaying pending output and choices.
    pub fn is_waiting(&self) -> bool {
        self.wait_ms > 0
    }
    pub fn viewport_rows(&self) -> usize {
        self.viewport_rows
    }
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Advance text revelation and waits using platform-supplied elapsed milliseconds.
    pub fn tick(&mut self, elapsed_ms: u32) {
        let mut available_ms = u64::from(elapsed_ms);
        loop {
            if self.is_waiting() {
                let consumed = available_ms.min(self.wait_ms);
                self.wait_ms -= consumed;
                available_ms -= consumed;
                if self.is_waiting() {
                    return;
                }
                self.present_next();
            } else if self.is_animating() {
                self.animation_ms += available_ms;
                let remaining = self.text_chars() - self.visible_chars;
                let chars = (self.animation_ms / (1000 / CHARS_PER_SECOND)).min(remaining as u64);
                self.visible_chars += chars as usize;
                self.animation_ms -= chars * (1000 / CHARS_PER_SECOND);
                if self.is_animating() {
                    return;
                }
                available_ms = core::mem::take(&mut self.animation_ms);
                self.present_next();
            } else {
                return;
            }
        }
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Result<(), StoryError> {
        if self.is_animating() || self.is_waiting() {
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
                self.default_style = Style::default();
                self.text_animation = true;
                self.wait_ms = 0;
                self.lines.clear();
                self.pending.clear();
                self.pending_choices.clear();
                self.pending_finished = false;
                self.choices.clear();
                self.finished = false;
                self.visible_chars = 0;
                self.animation_ms = 0;
                self.scroll = 0;
                self.total_rows = 0;
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

    #[cfg(test)]
    pub(crate) fn visible_text(&self) -> Text<'static> {
        let mut lines = Vec::new();
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 {
                lines.push(Line::default());
            }
            let count = if index + 1 == self.lines.len() {
                self.visible_chars
            } else {
                line.text.chars().count()
            };
            let end = line
                .text
                .char_indices()
                .nth(count)
                .map_or(line.text.len(), |(i, _)| i);
            for part in line.text[..end].split('\n') {
                lines.push(
                    Line::from(Span::styled(part.to_string(), line.style))
                        .alignment(line.alignment),
                );
            }
        }
        Text::from(lines)
    }

    pub(crate) fn visible_lines(&self) -> Vec<StoryLine> {
        self.lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let mut line = line.clone();
                if index + 1 == self.lines.len() {
                    let end = line
                        .text
                        .char_indices()
                        .nth(self.visible_chars)
                        .map_or(line.text.len(), |(i, _)| i);
                    line.text.truncate(end);
                }
                line
            })
            .collect()
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
            .last()
            .map_or(0, |line| line.text.chars().count())
    }

    fn skip_line(&mut self) {
        self.visible_chars = self.text_chars();
        self.animation_ms = 0;
        self.wait_ms = 0;
        self.present_next();
    }

    /// Execute immediate output until an animated line or timed wait needs a tick.
    fn present_next(&mut self) {
        while let Some(event) = self.pending.pop_front() {
            match event {
                OutputEvent::Clear => {
                    self.lines.clear();
                    self.visible_chars = 0;
                    self.scroll = 0;
                    self.total_rows = 0;
                    self.follow = true;
                }
                OutputEvent::Set(settings) => {
                    self.default_style = self.default_style.patch(settings.style);
                    if let Some(enabled) = settings.text_animation {
                        self.text_animation = enabled;
                    }
                }
                OutputEvent::Wait(milliseconds) => {
                    self.wait_ms = milliseconds;
                    if self.is_waiting() {
                        return;
                    }
                }
                OutputEvent::Text(line) => {
                    if self.present_text(line, None) {
                        return;
                    }
                }
                OutputEvent::TextAnimation(line, animated) => {
                    if self.present_text(line, Some(animated)) {
                        return;
                    }
                }
            }
        }
        self.choices = core::mem::take(&mut self.pending_choices);
        self.finished = self.pending_finished;
    }

    fn present_text(&mut self, mut line: StoryLine, animation_override: Option<bool>) -> bool {
        line.style = self.default_style.patch(line.style);
        self.lines.push(line);
        let animated = animation_override.unwrap_or(self.text_animation);
        if animated {
            self.visible_chars = 0;
        } else {
            self.visible_chars = self.text_chars();
        }
        animated
    }

    fn advance(&mut self) -> Result<(), StoryError> {
        let mut pending = VecDeque::new();
        while self.story.can_continue() {
            let line = self.story.cont()?;
            let tags = self.story.get_current_tags()?;
            if let Some(event) = parse_output(&line, &tags)? {
                pending.push_back(event);
            }
        }
        self.pending_choices = self
            .story
            .get_current_choices()
            .iter()
            .map(|c| c.text.clone())
            .collect();
        self.pending_finished = self.pending_choices.is_empty() && !self.story.can_continue();
        if self.pending_finished {
            self.pending_choices.push("-- restart --".to_string());
            let last_text = pending.iter().fold(
                self.lines.last().map(|line| line.text.as_str()),
                |last, event| match event {
                    OutputEvent::Text(line) | OutputEvent::TextAnimation(line, _) => {
                        Some(line.text.as_str())
                    }
                    OutputEvent::Clear => None,
                    OutputEvent::Set(_) | OutputEvent::Wait(_) => last,
                },
            );
            if !last_text.is_some_and(|text| text.trim().eq_ignore_ascii_case("the end")) {
                pending.push_back(OutputEvent::Text(StoryLine {
                    text: "The End".to_string(),
                    style: Style::default(),
                    alignment: Alignment::Left,
                    banner: None,
                }));
            }
        }
        self.pending = pending;
        self.choices.clear();
        self.finished = false;
        self.animation_ms = 0;
        self.selected = 0;
        self.present_next();
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
    fn example_choices_ending_and_restart() {
        let mut app = App::new(crate::STORY_IMAGE, 42).unwrap();
        assert!(!app.lines().is_empty());
        assert!(app.is_animating());
        let initial_lines = app.lines().to_vec();
        app.tick(50);
        assert_eq!(app.visible_chars, 1);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines().len(), 2);
        assert_eq!(app.visible_chars, 0);
        app.tick(1_000_000);
        assert!(!app.is_animating());
        assert!(!app.choices().is_empty());
        if app.choices().len() > 1 {
            app.handle_event(AppEvent::SelectNext).unwrap();
            assert_eq!(app.selected_choice(), Some(1));
            app.handle_event(AppEvent::SelectPrevious).unwrap();
            assert_eq!(app.selected_choice(), Some(0));
        }
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        app.tick(1_000_000);
        assert!(!app.is_finished());
        app.handle_event(AppEvent::SelectIndex(1)).unwrap();
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        app.tick(1_000_000);
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
        app.tick(3_000);
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
    fn fixture() -> App {
        App::new(include_bytes!(concat!(env!("OUT_DIR"), "/events.inkb")), 42).unwrap()
    }

    fn tag(value: &str) -> Vec<String> {
        alloc::vec![value.to_string()]
    }

    #[test]
    fn parsing_errors_and_color_formats() {
        for invalid in [
            "color",
            "color:",
            "color:garbage",
            "color:#ff0000",
            "color:123",
            "color:fffff",
            "color:fffffff",
            "color:ff00gg",
        ] {
            let error = parse_output("hello", &tag(invalid)).unwrap_err();
            assert!(matches!(error, StoryError::InvalidStoryState(_)));
            assert!(error.to_string().contains(invalid));
        }
        for command in [">CLS", ">clear", ">", ">cls extra"] {
            assert!(matches!(
                parse_output(command, &[]),
                Err(StoryError::InvalidStoryState(_))
            ));
        }
        assert!(parse_output(">cls", &tag("color:bad")).is_err());
        assert_eq!(
            parse_output("> cls  ", &tag("color:red")).unwrap(),
            Some(OutputEvent::Clear)
        );
        for (value, color) in [
            ("red", Color::Red),
            ("ff0000", Color::Rgb(255, 0, 0)),
            ("FF0000", Color::Rgb(255, 0, 0)),
            ("light-blue", Color::LightBlue),
        ] {
            let Some(OutputEvent::Text(line)) =
                parse_output("hello", &tag(&format!("color: {value} "))).unwrap()
            else {
                panic!()
            };
            assert_eq!(line.style.fg, Some(color));
        }
        let Some(OutputEvent::Text(line)) =
            parse_output("literal #color:red", &tag("unknown")).unwrap()
        else {
            panic!()
        };
        assert_eq!(line.style, Style::default());
        assert_eq!(line.text, "literal #color:red");
    }

    #[test]
    fn compiled_tags_unicode_commands_and_enter() {
        let mut app = fixture();
        assert_eq!(app.lines()[0].text, "á🙂界");
        assert_eq!(app.lines()[0].style.fg, Some(Color::Blue));
        app.tick(50);
        assert_eq!(app.visible_text().lines[0].spans[0].content, "á");
        app.tick(50);
        assert_eq!(app.visible_text().lines[0].spans[0].content, "á🙂");
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[1].text, "plain");
        assert_eq!(app.lines()[1].style, Style::default());
        app.set_viewport(100, 5);
        app.handle_event(AppEvent::JumpStoryEnd).unwrap();
        assert_eq!(app.scroll(), 95);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.scroll(), 0);
        assert_eq!(app.lines().len(), 1);
        assert_eq!(app.lines()[0].text, "red");
        assert!(app.choices().is_empty());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[1].text, "rgb");
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.lines().is_empty());
        assert_eq!(
            app.choices(),
            ["Finish", "Invalid command", "Invalid color"]
        );
        assert!(!app.is_finished());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        app.tick(1000);
        assert_eq!(app.lines()[0].text, "The End");
        assert!(app.is_finished());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[0].text, "á🙂界");
        assert_eq!(app.lines()[0].style.fg, Some(Color::Blue));
        assert_eq!(app.visible_chars, 0);
    }

    #[test]
    fn invalid_blocks_fail_before_presenting_any_text() {
        for (choice, content) in [(1, ">unknown"), (2, "color:42")] {
            let mut app = fixture();
            app.tick(1000);
            app.handle_event(AppEvent::SelectIndex(choice)).unwrap();
            let before = app.lines().to_vec();
            let error = app.handle_event(AppEvent::ChooseSelected).unwrap_err();
            assert!(matches!(error, StoryError::InvalidStoryState(_)));
            assert!(error.to_string().contains(content));
            assert_eq!(app.lines(), before);
            assert!(app.pending.is_empty());
        }
    }

    #[test]
    fn ticks_preserve_time_across_clears_and_lines() {
        for elapsed in [1, 49, 50, 149, 150, 151, 449, 450, 451, 749, 750, 1000] {
            let mut large = fixture();
            let mut small = fixture();
            large.tick(elapsed);
            for _ in 0..elapsed {
                small.tick(1);
            }
            assert_eq!(large.lines(), small.lines());
            assert_eq!(large.pending, small.pending);
            assert_eq!(large.visible_chars, small.visible_chars);
            assert_eq!(large.animation_ms, small.animation_ms);
            assert_eq!(large.choices(), small.choices());
        }
    }

    fn assert_rendered_color(app: &mut App, text: &str, color: Color) {
        app.handle_event(AppEvent::JumpStoryStart).unwrap();
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        for y in (1..23).rev() {
            let row: String = (0..100).map(|x| buffer[(x, y)].symbol()).collect();
            if let Some(x) = row.find(text) {
                let column = row[..x].chars().count() as u16;
                assert_eq!(buffer[(column, y)].fg, color);
                return;
            }
        }
        panic!("Missing rendered text: {text}");
    }

    #[test]
    fn real_story_tags_reach_the_renderer_without_leaking() {
        let mut app = App::new(crate::STORY_IMAGE, 42).unwrap();
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[0].style, Style::default());
        app.tick(50);
        assert_rendered_color(&mut app, "T", Color::Red);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        app.tick(50);
        assert_eq!(app.lines()[2].style.fg, Some(Color::Rgb(255, 0, 0)));
        // The previous named-color line is complete; the RGB line is partial.
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(1, 5)].symbol(), "T");
        assert_eq!(buffer[(1, 5)].fg, Color::Rgb(255, 0, 0));
        assert_eq!(buffer[(1, 3)].fg, Color::Red);
        assert_eq!(buffer[(1, 1)].fg, Color::Reset);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        while !app.lines()[0].text.starts_with("After the clear") {
            app.handle_event(AppEvent::ChooseSelected).unwrap();
        }
        assert_eq!(app.lines().len(), 1);
        assert!(app.lines()[0].text.starts_with("After the clear"));
        assert_eq!(app.lines()[0].style, Style::default());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        app.tick(50);
        assert_rendered_color(&mut app, "T", Color::Blue);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines().len(), 1);
        assert_eq!(
            app.lines()[0].text,
            "This line remains after the second clear."
        );
        assert_eq!(app.lines()[0].style, Style::default());
    }
    #[test]
    fn banner_tags_validate_preserve_text_and_last_tag_wins() {
        let event = parse_output("Abá\nñ", &["banner: full".into(), "banner: octant ".into()])
            .unwrap()
            .unwrap();
        let OutputEvent::Text(line) = event else {
            panic!()
        };
        assert_eq!(line.text, "Abá\nñ");
        assert_eq!(line.banner, Some(PixelSize::Octant));
        for tag in ["banner", "banner:", "banner: unknown", "banner:FULL"] {
            assert!(matches!(
                parse_output("text", &[tag.into()]),
                Err(StoryError::InvalidStoryState(_))
            ));
        }
    }

    #[test]
    fn banner_revelation_uses_original_unicode_characters_and_enter_skips_one_line() {
        let mut app = timing_fixture();
        app.pending.clear();
        app.wait_ms = 0;
        app.lines.clear();
        let OutputEvent::Text(line) = parse_output("áBñ", &["banner:octant".into()])
            .unwrap()
            .unwrap()
        else {
            panic!()
        };
        app.pending.push_back(OutputEvent::Wait(1000));
        assert!(app.present_text(line, Some(true)));
        assert_eq!(app.visible_lines()[0].text, "");
        app.tick(50);
        assert_eq!(app.visible_lines()[0].text, "á");
        assert_eq!(app.visible_lines()[0].banner, Some(PixelSize::Octant));
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.visible_lines()[0].text, "áBñ");
        assert_eq!(app.wait_ms, 1000);
    }

    #[test]
    fn banners_preserve_animation_waits_enter_restart_and_tick_time() {
        let mut large = App::new(crate::STORY_IMAGE, 42).unwrap();
        let mut small = App::new(crate::STORY_IMAGE, 42).unwrap();
        large.tick(100_000);
        for _ in 0..1000 {
            small.tick(100);
        }
        assert_eq!(large.lines(), small.lines());
        assert_eq!(large.pending, small.pending);
        assert_eq!(large.visible_chars, small.visible_chars);
        assert_eq!(large.wait_ms, small.wait_ms);
        assert_eq!(large.choices(), small.choices());
        assert!(
            large
                .lines()
                .iter()
                .any(|line| line.banner == Some(PixelSize::Octant))
        );
        large.handle_event(AppEvent::SelectIndex(1)).unwrap();
        large.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(large.is_finished());
        large.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(large.is_animating());
        assert_eq!(large.lines()[0].banner, None);
        assert_eq!(large.visible_chars, 0);
    }

    fn defaults_fixture() -> App {
        App::new(
            include_bytes!(concat!(env!("OUT_DIR"), "/defaults.inkb")),
            42,
        )
        .unwrap()
    }

    #[test]
    fn set_attributes_and_background_validation() {
        assert_eq!(
            parse_output(">set defaultcolor=red defaultbgcolor=green", &[]).unwrap(),
            Some(OutputEvent::Set(Settings {
                style: Style::default().fg(Color::Red).bg(Color::Green),
                text_animation: None
            }))
        );
        assert_eq!(
            parse_output("> set  defaultcolor=red\tdefaultcolor=FF0000  ", &[]).unwrap(),
            Some(OutputEvent::Set(Settings {
                style: Style::default().fg(Color::Rgb(255, 0, 0)),
                text_animation: None
            }))
        );
        for command in [
            ">set",
            ">set defaultcolor",
            ">set =red",
            ">set defaultcolor=",
            ">set defaultcolor=#ff0000",
            ">set defaultbgcolor=42",
            ">set defaultcolor=red=blue",
            ">set unknown=green",
            ">cls name=value",
        ] {
            let error = parse_output(command, &[]).unwrap_err();
            assert!(matches!(error, StoryError::InvalidStoryState(_)));
        }
        for invalid in [
            "bgcolor",
            "bgcolor:",
            "bgcolor:bad",
            "bgcolor:#ff0000",
            "bgcolor:42",
        ] {
            let error = parse_output("text", &tag(invalid)).unwrap_err();
            assert!(error.to_string().contains(invalid));
            assert!(parse_output(">set defaultcolor=red", &tag(invalid)).is_err());
        }
        let Some(OutputEvent::Text(line)) = parse_output("text", &tag("bgcolor: FF0000")).unwrap()
        else {
            panic!()
        };
        assert_eq!(line.style.bg, Some(Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn default_colors_follow_commands_overrides_clear_choices_and_restart() {
        let mut app = defaults_fixture();
        assert_eq!(app.default_style, Style::default());
        app.tick(149);
        assert_eq!(app.lines()[0].style, Style::default());
        assert_eq!(app.default_style, Style::default());
        app.tick(1);
        assert_eq!(app.lines()[0].style, Style::default());
        assert_eq!(app.lines()[1].text, "inherited");
        assert_eq!(app.lines()[1].style, Style::default().fg(Color::Red));
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        let inherited = Style::default().fg(Color::Red).bg(Color::Green);
        assert_eq!(app.lines()[2].style, inherited);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(
            app.lines()[3].style,
            Style::default().fg(Color::Blue).bg(Color::Red)
        );
        app.tick(50);
        let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let cell = &terminal.backend().buffer()[(1, 7)];
        assert_eq!(cell.symbol(), "e");
        assert_eq!(cell.fg, Color::Blue);
        assert_eq!(cell.bg, Color::Red);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[4].style, inherited);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines().len(), 1);
        assert_eq!(app.lines()[0].text, "preserved");
        assert_eq!(app.lines()[0].style, inherited);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        let updated = Style::default().fg(Color::Yellow).bg(Color::Blue);
        assert_eq!(app.lines()[1].style, updated);
        app.tick(50);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_eq!(terminal.backend().buffer()[(1, 1)].bg, Color::Green);
        assert_eq!(terminal.backend().buffer()[(1, 3)].bg, Color::Blue);
        app.tick(1000);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines().last().unwrap().text, "continued");
        assert_eq!(app.lines().last().unwrap().style, updated);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines().last().unwrap().text, "reset");
        assert_eq!(
            app.lines().last().unwrap().style,
            Style::default().fg(Color::Reset).bg(Color::Reset)
        );
        app.tick(1000);
        assert!(app.is_finished());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[0].text, "old");
        assert_eq!(app.lines()[0].style, Style::default());
        assert_eq!(app.default_style, Style::default());
    }

    #[test]
    fn default_commands_preserve_tick_time_and_validate_whole_blocks() {
        for elapsed in [149, 150, 151, 600, 601, 1000, 2000, 3000] {
            let mut large = defaults_fixture();
            let mut small = defaults_fixture();
            large.tick(elapsed);
            for _ in 0..elapsed {
                small.tick(1);
            }
            assert_eq!(large.lines(), small.lines());
            assert_eq!(large.pending, small.pending);
            assert_eq!(large.default_style, small.default_style);
            assert_eq!(large.animation_ms, small.animation_ms);
            assert_eq!(large.visible_chars, small.visible_chars);
        }
        let mut app = defaults_fixture();
        app.tick(10_000);
        app.handle_event(AppEvent::SelectIndex(1)).unwrap();
        let before = app.lines().to_vec();
        let style = app.default_style;
        let error = app.handle_event(AppEvent::ChooseSelected).unwrap_err();
        assert!(error.to_string().contains("unknown=blue"));
        assert_eq!(app.lines(), before);
        assert_eq!(app.default_style, style);
    }
    fn alignment_fixture() -> App {
        App::new(
            include_bytes!(concat!(env!("OUT_DIR"), "/alignment.inkb")),
            42,
        )
        .unwrap()
    }

    #[test]
    fn alignment_tags_validate_and_last_tag_wins() {
        for (value, expected) in [
            ("left", Alignment::Left),
            (" center ", Alignment::Center),
            ("right", Alignment::Right),
        ] {
            let Some(OutputEvent::Text(line)) =
                parse_output("text", &tag(&format!("align:{value}"))).unwrap()
            else {
                panic!()
            };
            assert_eq!(line.alignment, expected);
        }
        let Some(OutputEvent::Text(line)) = parse_output(
            "first\nsecond",
            &alloc::vec!["align:right".to_string(), "align:center".to_string()],
        )
        .unwrap() else {
            panic!()
        };
        assert_eq!(line.alignment, Alignment::Center);
        let mut app = alignment_fixture();
        app.lines = alloc::vec![line];
        app.visible_chars = 100;
        assert!(
            app.visible_text()
                .lines
                .iter()
                .all(|line| line.alignment == Some(Alignment::Center))
        );
        for invalid in [
            "align",
            "align:",
            "align:justify",
            "align:Center",
            "align:[center|right|left]",
        ] {
            let error = parse_output("text", &tag(invalid)).unwrap_err();
            assert!(matches!(error, StoryError::InvalidStoryState(_)));
            assert!(error.to_string().contains(invalid));
            assert!(parse_output(">cls", &tag(invalid)).is_err());
        }
    }

    #[test]
    fn compiled_alignment_renders_partial_unicode_wrapping_and_line_scope() {
        let mut app = alignment_fixture();
        let mut terminal = Terminal::new(TestBackend::new(12, 24)).unwrap();
        app.tick(50);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let cell = &terminal.backend().buffer()[(6, 1)];
        assert_eq!(cell.symbol(), "á");
        assert_eq!(cell.fg, Color::Red);
        assert_eq!(cell.bg, Color::Green);
        app.tick(50);
        assert_eq!(app.lines()[1].alignment, Alignment::Right);
        app.tick(50);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        assert_eq!(terminal.backend().buffer()[(5, 1)].symbol(), "á");
        assert_eq!(terminal.backend().buffer()[(6, 1)].symbol(), "界");
        assert_eq!(terminal.backend().buffer()[(10, 3)].symbol(), "x");
        app.tick(10_000);
        terminal.draw(|frame| app.draw(frame)).unwrap();
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer[(9, 3)].symbol(), "x");
        assert_eq!(buffer[(10, 3)].symbol(), "y");
        assert_eq!(buffer[(1, 5)].symbol(), "l");
        assert_eq!(buffer[(1, 7)].symbol(), "p");
        assert_eq!(buffer[(4, 9)].symbol(), "o");
        assert_eq!(buffer[(6, 10)].symbol(), "t");
        assert_eq!(app.lines()[3].alignment, Alignment::Left);
        assert_eq!(app.lines()[5].alignment, Alignment::Center);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        app.tick(1000);
        assert_eq!(app.lines().last().unwrap().alignment, Alignment::Left);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.lines()[0].alignment, Alignment::Center);
        assert_eq!(app.visible_chars, 0);
    }

    #[test]
    fn invalid_alignment_block_fails_before_presentation() {
        let mut app = alignment_fixture();
        app.tick(10_000);
        let before = app.lines().to_vec();
        app.handle_event(AppEvent::SelectIndex(1)).unwrap();
        let error = app.handle_event(AppEvent::ChooseSelected).unwrap_err();
        assert!(error.to_string().contains("align:justify"));
        assert_eq!(app.lines(), before);
    }
    fn timing_fixture() -> App {
        App::new(include_bytes!(concat!(env!("OUT_DIR"), "/timing.inkb")), 42).unwrap()
    }

    #[test]
    fn timing_commands_validate_attributes() {
        assert_eq!(
            parse_output(">set text-animation=false defaultcolor=red", &[]).unwrap(),
            Some(OutputEvent::Set(Settings {
                style: Style::default().fg(Color::Red),
                text_animation: Some(false)
            }))
        );
        assert_eq!(
            parse_output(">set text-animation=false text-animation=true", &[]).unwrap(),
            Some(OutputEvent::Set(Settings {
                style: Style::default(),
                text_animation: Some(true)
            }))
        );
        for (command, milliseconds) in [
            (">wait time=0", 0),
            ("> wait time=2  ", 2000),
            (">wait time=1 time=3", 3000),
            (">wait time=18446744073709551", 18_446_744_073_709_551_000),
        ] {
            assert_eq!(
                parse_output(command, &[]).unwrap(),
                Some(OutputEvent::Wait(milliseconds))
            );
        }
        for command in [
            ">set text-animation",
            ">set text-animation=",
            ">set text-animation=True",
            ">set text-animation=1",
            ">set text-animation=no",
            ">wait",
            ">wait time",
            ">wait time=",
            ">wait time=-1",
            ">wait time=+1",
            ">wait time=0.5",
            ">wait time=nan",
            ">wait time=18446744073709552",
            ">wait time=18446744073709551616",
            ">wait duration=1",
            ">wait time=1 other=2",
        ] {
            assert!(
                matches!(
                    parse_output(command, &[]),
                    Err(StoryError::InvalidStoryState(_))
                ),
                "{command}"
            );
        }
        assert!(parse_output(">wait time=1", &tag("color:bad")).is_err());
    }

    #[test]
    fn waits_and_animation_switches_execute_in_order_and_reset() {
        let mut app = timing_fixture();
        assert!(app.is_waiting());
        assert!(!app.is_animating());
        assert!(app.lines().is_empty());
        app.tick(999);
        assert_eq!(app.wait_ms, 1);
        assert!(app.lines().is_empty());
        app.tick(1);
        assert!(!app.is_waiting());
        assert!(app.is_animating());
        assert_eq!(app.visible_chars, 0);
        app.tick(50);
        assert_eq!(app.visible_text().lines[0].spans[0].content, "á");
        assert!(app.text_animation);
        app.tick(50);
        assert!(!app.text_animation);
        assert!(app.is_waiting());
        assert!(!app.is_animating());
        assert_eq!(app.lines()[1].text, "instant");
        assert_eq!(app.visible_chars, 7);
        assert_eq!(app.lines()[1].alignment, Alignment::Center);
        assert_eq!(app.lines()[1].style.fg, Some(Color::Red));
        let mut terminal = Terminal::new(TestBackend::new(30, 16)).unwrap();
        terminal.draw(|frame| app.draw(frame)).unwrap();
        for (i, ch) in "instant".chars().enumerate() {
            let cell = &terminal.backend().buffer()[(12 + i as u16, 3)];
            assert_eq!(cell.symbol(), ch.to_string());
            assert_eq!(cell.fg, Color::Red);
        }
        assert!(app.choices().is_empty());
        app.handle_event(AppEvent::SelectNext).unwrap();
        assert_eq!(app.selected_choice(), None);
        app.tick(1999);
        assert_eq!(app.wait_ms, 1);
        assert_eq!(app.lines().len(), 2);
        app.tick(1);
        assert!(app.text_animation);
        assert_eq!(app.lines()[0].text, "after");
        assert_eq!(app.lines()[1].text, "slow");
        assert_eq!(app.visible_chars, 0);
        assert_eq!(app.lines()[1].style.bg, Some(Color::Green));
        app.tick(200);
        assert_eq!(app.lines()[2].text, "last");
        assert!(!app.text_animation);
        assert!(app.is_waiting());
        assert_eq!(app.visible_chars, 4);
        app.tick(999);
        assert!(app.choices().is_empty());
        app.tick(1);
        assert_eq!(app.choices().len(), 3);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.is_waiting());
        assert!(!app.text_animation);
        app.tick(1000);
        assert!(app.is_finished());
        assert!(app.lines().iter().any(|line| line.text == "continued"));
        assert!(!app.is_animating());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.is_waiting());
        assert!(app.text_animation);
        assert!(app.lines().is_empty());
        assert_eq!(app.default_style, Style::default());
        assert_eq!(app.animation_ms, 0);
    }

    #[test]
    fn enter_skips_one_wait_or_animated_line_without_selecting_choices() {
        let mut app = timing_fixture();
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(!app.is_waiting());
        assert!(app.is_animating());
        app.tick(25);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.is_waiting());
        assert_eq!(app.wait_ms, 2000);
        assert_eq!(app.animation_ms, 0);
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.is_animating());
        assert_eq!(app.lines()[0].text, "after");
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert!(app.is_waiting());
        app.handle_event(AppEvent::ChooseSelected).unwrap();
        assert_eq!(app.choices().len(), 3);
        assert!(!app.is_waiting());
        assert!(!app.is_finished());
        assert_eq!(app.lines().last().unwrap().text, "last");
    }

    #[test]
    fn timing_ticks_preserve_all_remaining_time() {
        for elapsed in [
            0, 999, 1000, 1001, 1049, 1050, 1099, 1100, 1101, 3099, 3100, 3101, 3299, 3300, 3301,
            4299, 4300, 6000,
        ] {
            let mut large = timing_fixture();
            let mut small = timing_fixture();
            large.tick(elapsed);
            for _ in 0..elapsed {
                small.tick(1);
            }
            assert_eq!(large.lines(), small.lines(), "elapsed {elapsed}");
            assert_eq!(large.pending, small.pending);
            assert_eq!(large.visible_chars, small.visible_chars);
            assert_eq!(large.wait_ms, small.wait_ms);
            assert_eq!(large.animation_ms, small.animation_ms);
            assert_eq!(large.text_animation, small.text_animation);
            assert_eq!(large.choices(), small.choices());
        }
        let mut app = timing_fixture();
        // Consecutive waits also retain elapsed time, even without any text.
        app.pending.push_front(OutputEvent::Wait(2000));
        app.tick(2999);
        assert_eq!(app.wait_ms, 1);
        assert!(app.lines().is_empty());
        app.tick(51);
        assert_eq!(app.visible_chars, 1);
    }

    #[test]
    fn invalid_timing_blocks_do_not_change_presentation_or_settings() {
        for (index, invalid) in [(1, "text-animation=maybe"), (2, "time=-1")] {
            let mut app = timing_fixture();
            app.tick(10_000);
            let before = app.lines().to_vec();
            app.handle_event(AppEvent::SelectIndex(index)).unwrap();
            let error = app.handle_event(AppEvent::ChooseSelected).unwrap_err();
            assert!(error.to_string().contains(invalid));
            assert_eq!(app.lines(), before);
            assert!(!app.text_animation);
            assert!(!app.is_waiting());
        }
    }
    #[test]
    fn real_example_disables_centered_text_then_waits_and_animates_left_text() {
        let mut app = App::new(crate::STORY_IMAGE, 42).unwrap();
        for _ in 0..100 {
            if app.is_waiting() {
                break;
            }
            assert!(app.is_animating());
            app.handle_event(AppEvent::ChooseSelected).unwrap();
        }
        assert!(app.is_waiting());
        assert!(!app.text_animation);
        assert_eq!(app.wait_ms, 2000);
        let centered = app
            .lines()
            .iter()
            .find(|line| line.text.contains("centered and appears immediately"))
            .unwrap();
        assert_eq!(centered.alignment, Alignment::Center);
        assert_eq!(app.lines().last().unwrap().alignment, Alignment::Right);
        assert_eq!(app.visible_chars, app.text_chars());
        let before = app.lines().len();
        app.tick(1999);
        assert_eq!(app.lines().len(), before);
        app.tick(1);
        assert!(app.text_animation);
        assert!(app.is_animating());
        assert_eq!(app.visible_chars, 0);
        assert_eq!(app.lines().last().unwrap().alignment, Alignment::Left);
        assert!(
            app.lines()
                .last()
                .unwrap()
                .text
                .contains("animated after a two-second wait")
        );
        app.tick(50);
        assert_eq!(app.visible_chars, 1);
    }
}
