mod adventuregraph;
mod agink;
mod hardware;
mod ui;

use adventuregraph::Adventuregraph;
use esp_idf_hal::delay::FreeRtos;
use ui::screen::Screen;
use ui::scrolled_text;
use ui::story_screen::StoryScreen;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let mut adventuregraph = Adventuregraph::new()?;
    let mut story_screen = StoryScreen::new();

    story_screen.draw(&mut adventuregraph)?;

    loop {
        story_screen.update(&mut adventuregraph)?;
        FreeRtos::delay_ms(100u32);
    }
}

pub fn test_scroll(agdisplay: &mut hardware::agdisplay::AGDisplay) {
    let mut scrolled_text = scrolled_text::ScrolledText::new();
    //test scroll display: write 40 lines
    for i in 0..100 {
        scrolled_text.add_text(agdisplay, &format!("Line {}\n", i));
    }
}

fn test_rotary_encoder(
    aginput: &mut hardware::aginput::AGInput,
    agdisplay: &mut hardware::agdisplay::AGDisplay,
) {
    loop {
        if aginput.consume_sw() {
            println!("Button pressed");
        }

        match aginput.consume_rotary() {
            hardware::aginput::Direction::Left => {
                println!("Left");
            }
            hardware::aginput::Direction::Right => {
                println!("Right");
            }
            hardware::aginput::Direction::None => {}
        }

        FreeRtos::delay_ms(100u32);
    }
}
