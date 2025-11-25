use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Margin};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{
    Block, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    Wrap,
};

#[derive(Default)]
pub struct UiState {
    story_scroll: u16,
    story_total_lines: usize,
    pub story_viewport_length: usize,
    story_entry_count: usize,
    choices_state: ListState,
    choices_content_length: usize,
    choices_viewport_length: usize,
}

impl UiState {
    pub fn clamp_story_scroll(&mut self) {
        let max_scroll = self
            .story_total_lines
            .saturating_sub(self.story_viewport_length)
            .min(u16::MAX as usize);
        if self.story_scroll as usize > max_scroll {
            self.story_scroll = max_scroll as u16;
        }
    }

    pub fn scroll_story(&mut self, delta: i32) {
        if self.story_total_lines <= self.story_viewport_length {
            self.story_scroll = 0;
            return;
        }

        let max_scroll = self
            .story_total_lines
            .saturating_sub(self.story_viewport_length)
            .min(u16::MAX as usize) as i32;
        let current = self.story_scroll as i32;
        let next = (current + delta).clamp(0, max_scroll);
        self.story_scroll = next as u16;
    }

    pub fn jump_story_to_start(&mut self) {
        self.story_scroll = 0;
    }

    pub fn jump_story_to_end(&mut self) {
        if self.story_total_lines <= self.story_viewport_length {
            self.story_scroll = 0;
            return;
        }

        let max_scroll = self
            .story_total_lines
            .saturating_sub(self.story_viewport_length)
            .min(u16::MAX as usize);
        self.story_scroll = max_scroll as u16;
    }
}

pub fn draw(frame: &mut Frame, app: &App, ui: &mut UiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)])
        .split(frame.area());

    let story_text = if app.lines().is_empty() {
        "Loading story...".to_owned()
    } else {
        app.lines().join("\n\n")
    };

    let story_block = Block::bordered().title("THE INTERCEPT");
    let paragraph = Paragraph::new(story_text)
        .wrap(Wrap { trim: true })
        .block(story_block);

    ui.story_total_lines = usize::max(1, paragraph.line_count(chunks[0].width));
    ui.story_viewport_length = usize::max(1, chunks[0].height.saturating_sub(2) as usize);

    let entry_count = app.lines().len();
    if entry_count > ui.story_entry_count {
        ui.jump_story_to_end();
    }
    ui.story_entry_count = entry_count;
    ui.clamp_story_scroll();

    let story = paragraph.scroll((ui.story_scroll, 0));
    frame.render_widget(story, chunks[0]);
    if ui.story_total_lines > ui.story_viewport_length {
        let story_scroll_positions = ui
            .story_total_lines
            .saturating_sub(ui.story_viewport_length)
            .saturating_add(1);
        let mut story_scrollbar_state = ScrollbarState::new(story_scroll_positions.max(1))
            .position(ui.story_scroll as usize)
            .viewport_content_length(ui.story_viewport_length);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(Style::default().fg(Color::Yellow)),
            chunks[0].inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut story_scrollbar_state,
        );
    }

    if app.has_choices() {
        let items: Vec<ListItem> = app
            .choices()
            .iter()
            .enumerate()
            .map(|(index, choice)| ListItem::new(format!("{}. {}", index + 1, choice)))
            .collect();

        ui.choices_content_length = items.len();
        ui.choices_viewport_length = usize::max(1, chunks[1].height.saturating_sub(2) as usize);

        if let Some(selected) = app.selected_choice() {
            if ui.choices_state.selected() != Some(selected) {
                *ui.choices_state.selected_mut() = Some(selected);
            }
        } else {
            ui.choices_state.select(None);
        }

        let list = List::new(items)
            .block(Block::bordered())
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[1], &mut ui.choices_state);

        if ui.choices_content_length > ui.choices_viewport_length {
            let choices_scroll_position = ui.choices_state.offset();
            let choices_scroll_positions = ui
                .choices_content_length
                .saturating_sub(ui.choices_viewport_length)
                .saturating_add(1);
            let mut scrollbar_state = ScrollbarState::new(choices_scroll_positions.max(1))
                .position(choices_scroll_position)
                .viewport_content_length(ui.choices_viewport_length);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .thumb_style(Style::default().fg(Color::Yellow)),
                chunks[1].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    } else {
        ui.choices_content_length = 0;
        ui.choices_viewport_length = usize::max(1, chunks[1].height.saturating_sub(2) as usize);
        ui.choices_state.select(None);

        let info = if app.is_finished() {
            "Story completed. Press Enter to restart, PageUp/PageDown to scroll the text."
        } else {
            "Showing story... Use PageUp/PageDown/Home/End to scroll."
        };

        let info_paragraph = Paragraph::new(info)
            .wrap(Wrap { trim: true })
            .block(Block::bordered());

        frame.render_widget(info_paragraph, chunks[1]);
    }
}
