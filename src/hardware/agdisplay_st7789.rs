use display_interface_spi::SPIInterface;

use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio;

use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi;

use mipidsi::models::ST7789;
use mipidsi::Builder;
use mipidsi::Display;

const W: u16 = 240;
const H: u16 = 320;

type TAGDisplay<'d> = Display<
    SPIInterface<
        spi::SpiDeviceDriver<'d, spi::SpiDriver<'d>>,
        gpio::PinDriver<'d, gpio::Gpio4, gpio::Output>,
    >,
    ST7789,
    gpio::PinDriver<'d, gpio::Gpio8, gpio::Output>,
>;

pub struct AGDisplay<'d> {
    display: TAGDisplay<'d>,
    backlight: gpio::PinDriver<'d, gpio::Gpio9, gpio::Output>,
}

impl<'d> AGDisplay<'d> {
    pub fn new(
        backlight: gpio::Gpio9,
        dc: gpio::Gpio4,
        rst: gpio::Gpio8,
        spi: spi::SPI2,
        sclk: gpio::Gpio6,
        sdo: gpio::Gpio7,
        cs: gpio::Gpio5,
        sdi: gpio::Gpio3,
    ) -> anyhow::Result<Self> {
        let mut backlight = gpio::PinDriver::output(backlight)?;

        let di = SPIInterface::new(
            spi::SpiDeviceDriver::new_single(
                spi,
                sclk,
                sdo,
                Some(sdi),
                Some(cs),
                &spi::SpiDriverConfig::new().dma(spi::Dma::Disabled),
                &spi::SpiConfig::new().baudrate(80.MHz().into()),
            )?,
            gpio::PinDriver::output(dc)?,
        );

        let display = Builder::new(ST7789, di)
            .reset_pin(gpio::PinDriver::output(rst)?)
            .display_size(W, H)
            .init(&mut delay::Ets)
            .unwrap();

        backlight.set_high()?;

        Ok(Self { display, backlight })
    }

    pub fn get_display(&mut self) -> &mut TAGDisplay<'d> {
        &mut self.display
    }
}
