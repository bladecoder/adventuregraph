use std::{
    io,
    time::{Duration, Instant},
};

use adventuregraph_core::{App, AppEvent, STORY_IMAGE};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<()> {
    let audio = adventuregraph_desktop_audio::Audio::new();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.hide_cursor()?;
    let result = run(&mut terminal, &audio);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    audio: &adventuregraph_desktop_audio::Audio,
) -> Result<()> {
    let mut app = App::new(STORY_IMAGE, 42)?;
    app.set_buzzer_muted(std::env::var("ADVENTUREGRAPH_MUTE").is_ok_and(|value| value == "1"));
    let mut last = Instant::now();
    loop {
        audio.sync(&mut app);
        let now = Instant::now();
        let elapsed_ms = now.duration_since(last).as_millis().min(u32::MAX as u128) as u32;
        app.tick(elapsed_ms);
        last += std::time::Duration::from_millis(u64::from(elapsed_ms));
        audio.sync(&mut app);
        terminal.draw(|frame| app.draw(frame))?;
        if !event::poll(Duration::from_millis(25))? {
            continue;
        }
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Char('q')
                || (key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))
            {
                return Ok(());
            }
            let action = match key.code {
                KeyCode::Char(c @ '1'..='9') => {
                    let index = (c as u8 - b'1') as usize;
                    if index < app.choices().len() {
                        app.handle_event(AppEvent::SelectIndex(index))?;
                        Some(AppEvent::ChooseSelected)
                    } else {
                        None
                    }
                }
                KeyCode::Char('m') => {
                    app.set_buzzer_muted(!app.buzzer_muted());
                    None
                }
                KeyCode::Down | KeyCode::Char('s' | 'j') => Some(AppEvent::SelectNext),
                KeyCode::Up | KeyCode::Char('w' | 'k') => Some(AppEvent::SelectPrevious),
                KeyCode::Enter | KeyCode::Char(' ') => Some(AppEvent::ChooseSelected),
                KeyCode::PageDown => Some(AppEvent::ScrollStory(app.viewport_rows().max(1) as i32)),
                KeyCode::PageUp => {
                    Some(AppEvent::ScrollStory(-(app.viewport_rows().max(1) as i32)))
                }
                KeyCode::Home => Some(AppEvent::JumpStoryStart),
                KeyCode::End => Some(AppEvent::JumpStoryEnd),
                _ => None,
            };
            if let Some(action) = action {
                app.handle_event(action)?;
            }
        }
    }
}
