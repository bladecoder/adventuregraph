use crate::hardware::aginput::AGInput;

use crate::hardware::agaudio::AGAudio;
use crate::hardware::agdisplay::AGDisplay;
use esp_idf_hal::peripherals::Peripherals;

// configure Input PINS for pcbv1
#[cfg(feature = "pcbv1")]
pub type PinSw = esp_idf_hal::gpio::Gpio36;
#[cfg(feature = "pcbv1")]
pub type PinA = esp_idf_hal::gpio::Gpio38;
#[cfg(feature = "pcbv1")]
pub type PinB = esp_idf_hal::gpio::Gpio40;

// configure Input PINS for pcbv1.1
#[cfg(not(feature = "pcbv1"))]
pub type PinSw = esp_idf_hal::gpio::Gpio36;
#[cfg(not(feature = "pcbv1"))]
pub type PinA = esp_idf_hal::gpio::Gpio38;
#[cfg(not(feature = "pcbv1"))]
pub type PinB = esp_idf_hal::gpio::Gpio40;

pub fn init_peripherals<'a>(peripherals: Peripherals) -> (AGDisplay<'a>, AGInput<'a>, AGAudio<'a>) {
    println!("Setup display...");

    let agdisplay = AGDisplay::new(
        peripherals.pins.gpio12,
        peripherals.pins.gpio7,
        peripherals.pins.gpio5,
        peripherals.spi2,
        peripherals.pins.gpio11,
        peripherals.pins.gpio9,
        peripherals.pins.gpio3,
    )
    .unwrap();
    println!("Display initialized");

    println!("Setup input...");
    let input = AGInput::new(
        peripherals.pins.gpio36,
        peripherals.pins.gpio38,
        peripherals.pins.gpio40,
        peripherals.timer00,
    )
    .unwrap();

    println!("Setup audio...");
    let agaudio = AGAudio::new(
        peripherals.ledc.channel0,
        peripherals.ledc.timer0,
        peripherals.pins.gpio18,
    )
    .unwrap();

    println!("All peripherals initialized");

    (agdisplay, input, agaudio)
}
