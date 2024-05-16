use std::fmt::Write;

use esp_idf_hal::delay::{FreeRtos, BLOCK};
use esp_idf_hal::gpio::{Gpio4, Gpio5};
use esp_idf_hal::i2c::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;

use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::mode::TerminalMode;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};

const SSD1306_ADDRESS: u8 = 0x3c;

pub struct AGDisplay<'a> {
    display: Ssd1306<I2CInterface<I2cDriver<'a>>, DisplaySize128x64, TerminalMode>,
}

impl AGDisplay<'_> {
    pub fn new(i2c: I2C0, sda: Gpio5, scl: Gpio4) -> anyhow::Result<Self> {
        let config = I2cConfig::new().baudrate(100.kHz().into());
        let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

        let interface = I2CDisplayInterface::new(i2c_driver);

        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_terminal_mode();

        display
            .init()
            .map_err(|e| anyhow::anyhow!("Init error: {:?}", e))?;

        display
            .clear()
            .map_err(|e| anyhow::anyhow!("Clear screen error: {:?}", e))?;

        Ok(Self { display })
    }

    pub fn write_str(&mut self, text: &str) {
        self.display.write_str(text).unwrap()
    }

    pub fn clear(&mut self) {
        self.display.clear().unwrap()
    }
}

pub(crate) fn raw_i2c_test() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio4;

    println!("Starting I2C SSD1306 test");

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    // initialze the display - don't worry about the meaning of these bytes - it's specific to SSD1306
    i2c.write(SSD1306_ADDRESS, &[0, 0xae], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xd4], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x80], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xa8], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x3f], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xd3], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x00], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x40], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x8d], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x14], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xa1], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xc8], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xda], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x12], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x81], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xcf], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xf1], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xdb], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x40], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xa4], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xa6], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0xaf], BLOCK)?;
    i2c.write(SSD1306_ADDRESS, &[0, 0x20, 0x00], BLOCK)?;

    // fill the display
    for _ in 0..64 {
        let data: [u8; 17] = [
            0x40, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff,
        ];
        i2c.write(SSD1306_ADDRESS, &data, BLOCK)?;
    }

    loop {
        // we are sleeping here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(500);
        i2c.write(SSD1306_ADDRESS, &[0, 0xa6], BLOCK)?;
        FreeRtos::delay_ms(500);
        i2c.write(SSD1306_ADDRESS, &[0, 0xa7], BLOCK)?;
    }
}

pub(crate) fn display_test(text: &str) -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio4;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    let interface = I2CDisplayInterface::new(i2c_driver);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display
        .init()
        .map_err(|e| anyhow::anyhow!("Init error: {:?}", e))?;

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Txt error: {:?}", e))?;

    Text::with_baseline(text, Point::new(0, 16), text_style, Baseline::Top)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Txt2 error: {:?}", e))?;

    display
        .flush()
        .map_err(|e| anyhow::anyhow!("Flush error: {:?}", e))?;

    Ok(())
}

pub(crate) fn display_terminal_test(
    i2c: I2C0,
    sda: Gpio5,
    scl: Gpio4,
    text: &str,
) -> anyhow::Result<()> {
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;

    let interface = I2CDisplayInterface::new(i2c_driver);

    let mut display =
        Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0).into_terminal_mode();
    display
        .init()
        .map_err(|e| anyhow::anyhow!("Init error: {:?}", e))?;

    let _ = display.clear();

    display.write_str("0123456789abcde0123456789abcde0123456789abcde0123456789abcde")?;
    display.write_str(text)?;
    display.write_str("Españoláéíóú")?;

    Ok(())
}
