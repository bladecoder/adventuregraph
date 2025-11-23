use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::config::MODE_3;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio;
use mipidsi::interface::SpiInterface;

use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi;

use mipidsi::models::ST7735s;

use mipidsi::Builder;
use mipidsi::Display;

const W: u16 = 128;
const H: u16 = 160;

type TAGDisplay<'d> = Display<
    SpiInterface<
        'd,
        spi::SpiDeviceDriver<'d, spi::SpiDriver<'d>>,
        gpio::PinDriver<'d, gpio::Gpio7, gpio::Output>,
    >,
    ST7735s,
    gpio::PinDriver<'d, gpio::Gpio5, gpio::Output>,
>;

pub struct AGDisplay<'d> {
    display: TAGDisplay<'d>,
    backlight: PinDriver<'d, gpio::Gpio12, gpio::Output>,
}

impl<'d> AGDisplay<'d> {
    pub fn new(
        backlight: gpio::Gpio12,
        dc: gpio::Gpio7,
        rst: gpio::Gpio5,
        spi: spi::SPI2,
        sclk: impl Peripheral<P = impl OutputPin> + 'd,
        sdo: impl Peripheral<P = impl OutputPin> + 'd,
        cs: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> anyhow::Result<Self> {
        let mut backlight = PinDriver::output(backlight)?;

        let buffer = Box::leak(Box::new([0_u8; 512]));
        let di = SpiInterface::new(
            spi::SpiDeviceDriver::new_single(
                spi,
                sclk,
                sdo,
                Option::<gpio::AnyIOPin>::None,
                Some(cs),
                &spi::SpiDriverConfig::new().dma(spi::Dma::Disabled),
                &spi::SpiConfig::new()
                    .baudrate(26.MHz().into())
                    .data_mode(MODE_3),
            )?,
            gpio::PinDriver::output(dc)?,
            buffer,
        );

        let display = Builder::new(ST7735s, di)
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
