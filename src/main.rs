mod agink;
mod hardware;
mod ui;

use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::*;
use esp_idf_hal::sys::{esp_timer_get_time, rand};
use esp_idf_hal::{
    delay::FreeRtos,
    peripherals::Peripherals,
    sys::{heap_caps_get_free_size, MALLOC_CAP_8BIT},
};

use crate::{agink::AGInk, hardware::peripherals_cfg, ui::scrolled_text::ScrolledText};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
    println!("Free memory at start: {} kb", free_mem / 1024);
    let mut agdisplay;
    let mut aginput;
    let mut agaudio;
    (agdisplay, aginput, agaudio) = peripherals_cfg::init_peripherals(peripherals);

    let mut scrolled_text = ScrolledText::new();

    // Clear the screen
    agdisplay
        .get_display()
        .clear(Rgb565::BLACK)
        .map_err(|e| anyhow::anyhow!("Clearing screen"))?;
    agaudio.play_ok();

    scrolled_text.add_text(&mut agdisplay, "Peripherals initialized");
    println!("Peripherals initialized");
    scrolled_text.add_text(
        &mut agdisplay,
        &format!("Free memory: {} kb\n", free_mem / 1024),
    );

    let width = agdisplay.get_display().bounding_box().size.width;
    let height = agdisplay.get_display().bounding_box().size.height;

    scrolled_text.add_text(
        &mut agdisplay,
        &format!("Display dimensions: {}x{}", width, height),
    );

    // test scroll display: write 40 lines
    // for i in 0..40 {
    //     scrolled_text.add_text(&mut agdisplay, &format!("Line {}\n", i));
    // }

    println!("loading Ink story...");
    scrolled_text.add_text(&mut agdisplay, "loading Ink story...");
    let start = unsafe { esp_timer_get_time() };
    let mut agink = AGInk::new()?;
    let end = unsafe { esp_timer_get_time() };
    println!("Ink story loaded in {} ms", (end - start) / 1000);
    scrolled_text.add_text(
        &mut agdisplay,
        &format!("Ink story loaded in {} ms", (end - start) / 1000),
    );

    let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
    println!(
        "Free memory after init and load story: {} kb",
        free_mem / 1024
    );

    scrolled_text.add_text(
        &mut agdisplay,
        &format!("Free memory2: {} kb\n", free_mem / 1024),
    );

    let mut choices_displayed = false;
    let mut selected_choice: u8 = 0;

    loop {
        if agink.can_continue() {
            let line = agink.next_line()?;
            scrolled_text.add_text(&mut agdisplay, &line);
        }

        if agink.has_choices() && !choices_displayed {
            choices_displayed = true;
            let choices = agink.get_choices();

            scrolled_text.add_choices(&mut agdisplay, &choices);

            // choose a choice randomly
            let random_choice = (unsafe { rand() } % agink.get_num_choices() as i32) as usize;
            agink.choose(random_choice)?;
            //scrolled_text.clear(&mut agdisplay);
            scrolled_text.select_choice(&mut agdisplay, random_choice);
            choices_displayed = false;
            FreeRtos::delay_ms(1000u32);
        }

        if !agink.can_continue() && !agink.has_choices() {
            scrolled_text.add_text(&mut agdisplay, "The End");
            println!("The End");
            loop {
                FreeRtos::delay_ms(1000u32);
            }
        }

        if aginput.consume_sw() {
            println!("Button pressed");
            if choices_displayed {
                agink.choose(selected_choice as usize)?;
                choices_displayed = false;
            }
        }

        if aginput.consume_left() {
            println!("Left");
            if choices_displayed {
                selected_choice = (selected_choice + 1) % agink.get_num_choices() as u8;
                scrolled_text.select_choice(&mut agdisplay, selected_choice as usize);
            }
        }

        if aginput.consume_right() {
            println!("Right");
            if choices_displayed {
                selected_choice = (selected_choice + agink.get_num_choices() as u8 - 1)
                    % agink.get_num_choices() as u8;
                scrolled_text.select_choice(&mut agdisplay, selected_choice as usize);
            }
        }

        FreeRtos::delay_ms(100u32);
    }
}
