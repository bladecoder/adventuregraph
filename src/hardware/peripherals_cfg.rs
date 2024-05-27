use crate::hardware::aginput::AGInput;

use crate::hardware::agaudio::AGAudio;
use crate::hardware::agdisplay_st7735::AGDisplay;
use esp_idf_hal::peripherals::Peripherals;

// configure Input PINS for esp32
#[cfg(esp32)]
pub type PinSw = esp_idf_hal::gpio::Gpio27;
#[cfg(esp32)]
pub type PinA = esp_idf_hal::gpio::Gpio26;
#[cfg(esp32)]
pub type PinB = esp_idf_hal::gpio::Gpio25;

// configure Input PINS for esp32s2
#[cfg(esp32s2)]
pub type PinSw = esp_idf_hal::gpio::Gpio12;
#[cfg(esp32s2)]
pub type PinA = esp_idf_hal::gpio::Gpio10;
#[cfg(esp32s2)]
pub type PinB = esp_idf_hal::gpio::Gpio11;

#[cfg(esp32s2)]
pub(crate) fn init_peripherals<'a>(
    peripherals: Peripherals,
) -> (AGDisplay<'a>, AGInput<'a>, AGAudio<'a>) {
    println!("setup display...");
    // let agdisplay = AGDisplaySSD1306::new(
    //     peripherals.i2c0,
    //     peripherals.pins.gpio33,
    //     peripherals.pins.gpio35,
    // )
    // .unwrap();
    //let agdisplay = NoDisplay::new();

    // let agdisplay = AGDisplayST7789::new(
    //     peripherals.pins.gpio9,
    //     peripherals.pins.gpio4,
    //     peripherals.pins.gpio8,
    //     peripherals.spi3,
    //     peripherals.pins.gpio6,
    //     peripherals.pins.gpio7,
    //     peripherals.pins.gpio5,
    //     peripherals.pins.gpio3,
    // )
    // .unwrap();

    let agdisplay = AGDisplay::new(
        peripherals.pins.gpio9,
        peripherals.pins.gpio4,
        peripherals.pins.gpio8,
        peripherals.spi2,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        peripherals.pins.gpio5,
    )
    .unwrap();
    println!("Display initialized");

    println!("setup input...");
    let input = AGInput::new(
        peripherals.pins.gpio12,
        peripherals.pins.gpio10,
        peripherals.pins.gpio11,
    )
    .unwrap();

    println!("setup audio...");
    let agaudio = AGAudio::new(
        peripherals.ledc.channel0,
        peripherals.ledc.timer0,
        peripherals.pins.gpio18,
    )
    .unwrap();

    (agdisplay, input, agaudio)
}

#[cfg(esp32)]
pub(crate) fn init_peripherals<'a>(
    peripherals: Peripherals,
) -> (AGDisplay<'a>, AGInput<'a>, AGAudio<'a>) {
    println!("setup display...");
    let agdisplay = AGDisplay::new(
        peripherals.i2c0,
        peripherals.pins.gpio5,
        peripherals.pins.gpio4,
    )
    .unwrap();

    println!("Display initialized");

    println!("setup input...");
    let input = AGInput::new(
        peripherals.pins.gpio27,
        peripherals.pins.gpio26,
        peripherals.pins.gpio25,
    )
    .unwrap();

    println!("setup audio...");
    let agaudio = AGAudio::new(
        peripherals.ledc.channel0,
        peripherals.ledc.timer0,
        peripherals.pins.gpio16,
    )
    .unwrap();

    (agdisplay, input, agaudio)
}
