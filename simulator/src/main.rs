use std::{cell::RefCell, rc::Rc};

use adventuregraph_core::{
    AppEvent,
    app::App,
    ui::{UiState, draw},
};
use anyhow::Result;
use embedded_graphics_simulator::{
    OutputSettings, SimulatorDisplay, SimulatorEvent, Window, sdl2::Keycode,
};
use mousefood::embedded_graphics::geometry;
use mousefood::prelude::*;
use mousefood::ratatui::Terminal;

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
    let mut app = App::new()?;
    let mut ui_state = UiState::default();
    let output_settings = OutputSettings {
        scale: 2,
        max_fps: 30,
        ..Default::default()
    };

    let simulator_window = Rc::new(RefCell::new(Window::new(
        "Adventuregraph simulator",
        &output_settings,
    )));

    // Define properties of the display which will be shown in the simulator window
    let mut display = SimulatorDisplay::<Bgr565>::new(geometry::Size::new(240, 320));

    let flush_window = Rc::clone(&simulator_window);
    let backend_config = EmbeddedBackendConfig {
        flush_callback: Box::new(move |display| {
            flush_window.borrow_mut().update(display);
        }),
        ..Default::default()
    };
    let backend: EmbeddedBackend<SimulatorDisplay<_>, _> =
        EmbeddedBackend::new(&mut display, backend_config);

    // Start ratatui with our simulator backend
    let mut terminal = Terminal::new(backend)?;

    // Run an infinite loop, where widgets will be rendered
    loop {
        terminal.draw(|frame| draw(frame, &mut app, &mut ui_state))?;

        let mut window = simulator_window.borrow_mut();
        for event in window.events() {
            match event {
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::KeyDown { keycode, .. } => {
                    if let Some(index) = numeric_choice(keycode) {
                        app.handle_event(&mut ui_state, AppEvent::SelectIndex(index))?;
                        app.handle_event(&mut ui_state, AppEvent::ChooseSelected)?;
                        continue;
                    }

                    match keycode {
                        Keycode::Down | Keycode::S | Keycode::J => {
                            app.handle_event(&mut ui_state, AppEvent::SelectNext)?;
                        }
                        Keycode::Up | Keycode::W | Keycode::K => {
                            app.handle_event(&mut ui_state, AppEvent::SelectPrevious)?;
                        }
                        Keycode::Return | Keycode::Space => {
                            app.handle_event(&mut ui_state, AppEvent::ChooseSelected)?;
                        }
                        Keycode::PageDown => {
                            let delta = ui_state.story_viewport_length as i32;
                            app.handle_event(&mut ui_state, AppEvent::ScrollStory(delta))?;
                        }
                        Keycode::PageUp => {
                            let delta = ui_state.story_viewport_length as i32;
                            app.handle_event(&mut ui_state, AppEvent::ScrollStory(-delta))?;
                        }
                        Keycode::Home => {
                            app.handle_event(&mut ui_state, AppEvent::JumpStoryStart)?;
                        }
                        Keycode::End => {
                            app.handle_event(&mut ui_state, AppEvent::JumpStoryEnd)?;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
