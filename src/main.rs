mod agdisplay;
mod agink;
mod aginput;

use esp_idf_hal::{
    delay::FreeRtos,
    peripherals::Peripherals,
    sys::{
        esp_task_wdt_config_t, esp_task_wdt_deinit, esp_task_wdt_init, heap_caps_get_free_size,
        CONFIG_ESP_TASK_WDT_TIMEOUT_S, MALLOC_CAP_8BIT,
    },
    task::{self},
};

use crate::agdisplay::AGDisplay;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
    println!("Free memory: {} kb", free_mem / 1024);

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
    println!("Free memory: {} kb", free_mem / 1024);

    loop {
        if input.consume_sw() {
            println!("Button pressed");
        }

        if input.consume_left() {
            println!("Left");
        }

        if input.consume_right() {
            println!("Right");
        }

        FreeRtos::delay_ms(100u32);
    }
}
