use std::{cell::RefCell, rc::Rc, time::Instant};

use adventuregraph_core::{App, AppEvent, STORY_IMAGE};
use anyhow::Result;
use embedded_graphics_simulator::{
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use mousefood::{embedded_graphics::geometry, prelude::*};
use ratatui::Terminal;

fn numeric_choice(keycode: Keycode) -> Option<usize> {
    match keycode {
        Keycode::Num1 | Keycode::Kp1 => Some(0),
        Keycode::Num2 | Keycode::Kp2 => Some(1),
        Keycode::Num3 | Keycode::Kp3 => Some(2),
        Keycode::Num4 | Keycode::Kp4 => Some(3),
        Keycode::Num5 | Keycode::Kp5 => Some(4),
        Keycode::Num6 | Keycode::Kp6 => Some(5),
        Keycode::Num7 | Keycode::Kp7 => Some(6),
        Keycode::Num8 | Keycode::Kp8 => Some(7),
        Keycode::Num9 | Keycode::Kp9 => Some(8),
        _ => None,
    }
}

fn main() -> Result<()> {
    let audio = adventuregraph_desktop_audio::Audio::new();
    let mut app = App::new(STORY_IMAGE, 42)?;
    let settings = OutputSettings {
        scale: 2,
        max_fps: 30,
        ..Default::default()
    };
    let window = Rc::new(RefCell::new(Window::new(
        "Adventuregraph simulator",
        &settings,
    )));
    let mut display = SimulatorDisplay::<Rgb565>::new(geometry::Size::new(240, 320));
    let flush_window = Rc::clone(&window);
    let config = EmbeddedBackendConfig {
        font_regular: adventuregraph_core::embedded_font::FONT,
        flush_callback: Box::new(move |display| flush_window.borrow_mut().update(display)),
        ..Default::default()
    };
    let backend: EmbeddedBackend<SimulatorDisplay<_>, _> =
        EmbeddedBackend::new(&mut display, config);
    let mut terminal = Terminal::new(backend)?;
    // Blank Reset cells are omitted by Ratatui's first diff; initialize their pixels too.
    terminal.clear()?;
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
        for event in window.borrow_mut().events() {
            match event {
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::KeyDown { keycode, .. } => {
                    if let Some(index) = numeric_choice(keycode) {
                        if index < app.choices().len() {
                            app.handle_event(AppEvent::SelectIndex(index))?;
                            app.handle_event(AppEvent::ChooseSelected)?;
                        }
                        continue;
                    }
                    let action = match keycode {
                        Keycode::Escape | Keycode::Q => return Ok(()),
                        Keycode::M => {
                            app.set_buzzer_muted(!app.buzzer_muted());
                            None
                        }
                        Keycode::Down | Keycode::S | Keycode::J => Some(AppEvent::SelectNext),
                        Keycode::Up | Keycode::W | Keycode::K => Some(AppEvent::SelectPrevious),
                        Keycode::Return | Keycode::Space => Some(AppEvent::ChooseSelected),
                        Keycode::PageDown => {
                            Some(AppEvent::ScrollStory(app.viewport_rows().max(1) as i32))
                        }
                        Keycode::PageUp => {
                            Some(AppEvent::ScrollStory(-(app.viewport_rows().max(1) as i32)))
                        }
                        Keycode::Home => Some(AppEvent::JumpStoryStart),
                        Keycode::End => Some(AppEvent::JumpStoryEnd),
                        _ => None,
                    };
                    if let Some(action) = action {
                        app.handle_event(action)?;
                    }
                }
                _ => {}
            }
        }
    }
}
