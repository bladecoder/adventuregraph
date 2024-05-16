mod agdisplay;
mod agink;
mod aginput;

use esp_idf_hal::{
    delay::FreeRtos,
    peripherals::Peripherals,
    sys::{heap_caps_get_free_size, MALLOC_CAP_8BIT},
};

use crate::agdisplay::AGDisplay;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
    println!("Free memory at start: {} kb", free_mem / 1024);

    println!("setup ink...");
    let mut agink = agink::AGInk::new()?;
    println!("ink file loaded");
    let line = agink.next_line()?;

    println!("setup display...");
    let mut agdisplay = AGDisplay::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    )?;

    println!("Display initialized");

    agdisplay.clear();
    agdisplay.write_str(&line);

    println!("setup input...");
    let mut input = aginput::AGInput::new(
        peripherals.pins.gpio25,
        peripherals.pins.gpio26,
        peripherals.pins.gpio27,
    )?;

    let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
    println!(
        "Free memory after init and load story: {} kb",
        free_mem / 1024
    );

    let mut choices_displayed = false;
    let mut selected_choice: u8 = 0;
    let mut line_cursor = 0;

    loop {
        if agink.can_continue() {
            let line = agink.next_line()?;
            agdisplay.clear();
            agdisplay.write_str(&line);
        }

        if agink.has_choices() && !choices_displayed {
            choices_displayed = true;
            line_cursor = agdisplay.get_row();
            let choices = agink.get_choices();
            for (i, choice) in choices.iter().enumerate() {
                // Display choices
                if i == selected_choice as usize {
                    agdisplay.write_str(&format!(">{}\n", choice));
                } else {
                    agdisplay.write_str(&format!(" {}\n", choice));
                }
            }
        }

        if input.consume_sw() {
            println!("Button pressed");
            if choices_displayed {
                agink.choose(selected_choice as usize)?;
                choices_displayed = false;
            }
        }

        if input.consume_left() {
            println!("Left");
            if choices_displayed {
                agdisplay.set_char_at_posx(line_cursor + selected_choice, ' ');
                selected_choice = (selected_choice + 1) % agink.get_num_choices() as u8;
                agdisplay.set_char_at_posx(line_cursor + selected_choice, '>');
            }
        }

        if input.consume_right() {
            println!("Right");
            if choices_displayed {
                agdisplay.set_char_at_posx(line_cursor + selected_choice, ' ');
                selected_choice = (selected_choice + agink.get_num_choices() as u8 - 1)
                    % agink.get_num_choices() as u8;
                agdisplay.set_char_at_posx(line_cursor + selected_choice, '>');
            }
        }

        FreeRtos::delay_ms(100u32);
    }
}
