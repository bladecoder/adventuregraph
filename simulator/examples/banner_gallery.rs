//! Headless screenshots of the shared App through the real Mousefood backend.
use adventuregraph_core::{App, AppEvent, STORY_IMAGE};
use embedded_graphics_simulator::{OutputSettings, SimulatorDisplay};
use mousefood::{embedded_graphics::geometry::Size, prelude::*};
use ratatui::Terminal;

fn main() -> anyhow::Result<()> {
    let directory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/adventuregraph-banners".into());
    std::fs::create_dir_all(&directory)?;
    let mut app = App::new(STORY_IMAGE, 42)?;
    let mut captured = Vec::new();
    for _ in 0..200 {
        if app.is_waiting()
            && let Some(mode) = app.lines().last().and_then(|line| line.banner)
            && !captured.contains(&mode)
        {
            let mut display = SimulatorDisplay::<Rgb565>::new(Size::new(240, 320));
            {
                let config = EmbeddedBackendConfig {
                    font_regular: adventuregraph_core::embedded_font::FONT,
                    ..Default::default()
                };
                let backend: EmbeddedBackend<_, _> = EmbeddedBackend::new(&mut display, config);
                let mut terminal = Terminal::new(backend)?;
                terminal.draw(|frame| app.draw(frame))?;
            }
            display
                .to_rgb_output_image(&OutputSettings {
                    scale: 2,
                    ..Default::default()
                })
                .save_png(format!("{directory}/{mode:?}.png"))?;
            captured.push(mode);
        }
        if !app.choices().is_empty() {
            break;
        }
        app.handle_event(AppEvent::ChooseSelected)?;
    }
    anyhow::ensure!(
        captured.len() == 8,
        "Expected eight banner modes, got {}",
        captured.len()
    );
    Ok(())
}
