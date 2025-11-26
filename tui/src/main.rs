use std::io;
use std::time::Duration;

use adventuregraph_core::{
    AppEvent,
    app::App,
    ui::{UiState, draw},
};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn numeric_choice(code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Char(c @ '1'..='9') => Some((c as u8 - b'1') as usize),
        _ => None,
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new()?;
    let mut ui_state = UiState::default();

    loop {
        terminal.draw(|frame| draw(frame, &mut app, &mut ui_state))?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && matches!(key_event.code, KeyCode::Char('c'))
                {
                    return Ok(());
                }

                if let Some(index) = numeric_choice(key_event.code) {
                    app.handle_event(&mut ui_state, AppEvent::SelectIndex(index))?;
                    app.handle_event(&mut ui_state, AppEvent::ChooseSelected)?;
                    continue;
                }

                match key_event.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => {
                        app.handle_event(&mut ui_state, AppEvent::SelectNext)?;
                    }
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => {
                        app.handle_event(&mut ui_state, AppEvent::SelectPrevious)?;
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        app.handle_event(&mut ui_state, AppEvent::ChooseSelected)?;
                    }
                    KeyCode::PageDown => {
                        let delta = ui_state.story_viewport_length as i32;
                        app.handle_event(&mut ui_state, AppEvent::ScrollStory(delta))?;
                    }
                    KeyCode::PageUp => {
                        let delta = ui_state.story_viewport_length as i32;
                        app.handle_event(&mut ui_state, AppEvent::ScrollStory(-delta))?;
                    }
                    KeyCode::Home => app.handle_event(&mut ui_state, AppEvent::JumpStoryStart)?,
                    KeyCode::End => app.handle_event(&mut ui_state, AppEvent::JumpStoryEnd)?,
                    _ => {}
                }
            }
            Event::Resize(_, _) => {
                ui_state.clamp_story_scroll();
            }
            _ => {}
        }
    }
}
