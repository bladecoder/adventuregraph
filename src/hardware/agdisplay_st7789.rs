use display_interface_spi::SPIInterface;
use embedded_graphics::mono_font::iso_8859_15::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Text;
use esp_idf_hal::spi::config::MODE_3;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio;

use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi;

use mipidsi::models::ST7789;
use mipidsi::options::ColorInversion;
use mipidsi::Builder;
use mipidsi::Display;

use super::agdisplay::AGDisplay;

const W: u16 = 320;
const H: u16 = 240;

pub struct AGDisplayST7789<'d> {
    display: Display<
        SPIInterface<
            spi::SpiDeviceDriver<'d, spi::SpiDriver<'d>>,
            gpio::PinDriver<'d, gpio::Gpio4, gpio::Output>,
        >,
        ST7789,
        gpio::PinDriver<'d, gpio::Gpio8, gpio::Output>,
    >,
}

impl<'d> AGDisplayST7789<'d> {
    pub fn new(
        backlight: gpio::Gpio9,
        dc: gpio::Gpio4,
        rst: gpio::Gpio8,
        spi: spi::SPI3,
        sclk: gpio::Gpio6,
        sdo: gpio::Gpio7,
        cs: gpio::Gpio5,
        sdi: gpio::Gpio3,
    ) -> anyhow::Result<Self> {
        let mut backlight = gpio::PinDriver::output(backlight)?;
        backlight.set_high()?;

        let di = SPIInterface::new(
            spi::SpiDeviceDriver::new_single(
                spi,
                sclk,
                sdo,
                Some(sdi),
                Some(cs),
                &spi::SpiDriverConfig::new().dma(spi::Dma::Disabled),
                &spi::SpiConfig::new()
                    .baudrate(26.MHz().into())
                    .data_mode(MODE_3),
            )?,
            gpio::PinDriver::output(dc)?,
        );

        let display = Builder::new(ST7789, di)
            .reset_pin(gpio::PinDriver::output(rst)?)
            .display_size(W, H)
            .invert_colors(ColorInversion::Inverted)
            .init(&mut delay::Ets)
            .unwrap();

        Ok(Self { display })
    }
}

impl<'d> AGDisplay for AGDisplayST7789<'d> {
    fn get_display<D: DrawTarget<Color = Rgb565>>(&mut self) -> &mut D {
        &mut self.display
    }
}
