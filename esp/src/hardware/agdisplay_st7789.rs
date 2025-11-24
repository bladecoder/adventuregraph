use mipidsi::interface::SpiInterface;

use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::hal::delay;
use esp_idf_svc::hal::gpio;

use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::spi;

use mipidsi::Builder;
use mipidsi::Display;
use mipidsi::models::ST7789;
use mipidsi::options::Orientation;
use mipidsi::options::Rotation;

const W: u16 = 240;
const H: u16 = 320;

type TAGDisplay<'d> = Display<
    SpiInterface<
        'd,
        spi::SpiDeviceDriver<'d, spi::SpiDriver<'d>>,
        gpio::PinDriver<'d, gpio::Gpio7, gpio::Output>,
    >,
    ST7789,
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
        let mut backlight = gpio::PinDriver::output(backlight)?;

        let buffer = Box::leak(Box::new([0_u8; 4096]));
        let di = SpiInterface::new(
            spi::SpiDeviceDriver::new_single(
                spi,
                sclk,
                sdo,
                Option::<gpio::AnyIOPin>::None,
                Some(cs),
                &spi::SpiDriverConfig::new().dma(spi::Dma::Disabled),
                &spi::SpiConfig::new().baudrate(80.MHz().into()),
            )?,
            gpio::PinDriver::output(dc)?,
            buffer,
        );

        let display = Builder::new(ST7789, di)
            .reset_pin(gpio::PinDriver::output(rst)?)
            .display_size(W, H)
            .orientation(Orientation::new().rotate(Rotation::Deg180))
            .init(&mut delay::Ets)
            .unwrap();

        backlight.set_high()?;

        Ok(Self { display, backlight })
    }

    pub fn get_display(&mut self) -> &mut TAGDisplay<'d> {
        &mut self.display
    }
}
