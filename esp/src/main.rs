mod hardware;

use adventuregraph_core::{
    app::App,
    ui::{UiState, draw},
};
use esp_idf_hal::{delay::FreeRtos, peripherals::Peripherals};
use hardware::aginput::Direction;
use mousefood::prelude::*;
use mousefood::ratatui::Terminal;

use mousefood::fonts::{MONO_7X13, MONO_7X13_BOLD};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let (mut agdisplay, mut aginput, mut agaudio) =
        hardware::peripherals_cfg::init_peripherals(peripherals);

    let mut app = App::new()?;
    let mut ui_state = UiState::default();

    let config = EmbeddedBackendConfig {
        font_regular: MONO_7X13,
        font_bold: MONO_7X13_BOLD,
        //font_italic: Some(fonts::MONO_6X13_ITALIC),
        ..Default::default()
    };

    let backend: EmbeddedBackend<_, _> = EmbeddedBackend::new(agdisplay.get_display(), config);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|frame| draw(frame, &app, &mut ui_state))?;

        let inputs = aginput.get_inputs();
        let scroll_step = usize::max(1, ui_state.story_viewport_length / 2) as i32;

        match inputs.rotary {
            Direction::Left => {
                if app.has_choices() {
                    app.select_previous();
                } else {
                    ui_state.scroll_story(-scroll_step);
                }
            }
            Direction::Right => {
                if app.has_choices() {
                    app.select_next();
                } else {
                    ui_state.scroll_story(scroll_step);
                }
            }
            Direction::None => {}
        }

        if inputs.encsw {
            if app.has_choices() || app.is_finished() {
                app.choose_selected()?;
                ui_state.jump_story_to_end();
            } else {
                ui_state.jump_story_to_end();
            }
        }

        if inputs.btn1 {
            agaudio.play_ok();
            ui_state.jump_story_to_start();
        }

        FreeRtos::delay_ms(50u32);
    }
}
