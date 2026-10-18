//! Interior mutability lets LEDC keep its shared timer reference while pitch changes.
use core::cell::RefCell;
use esp_hal::{
    ledc::{
        LowSpeed,
        timer::{self, TimerIFace},
    },
    time::Rate,
};

pub struct BuzzerTimer<'d>(RefCell<timer::Timer<'d, LowSpeed>>);

impl<'d> BuzzerTimer<'d> {
    pub fn new(timer: timer::Timer<'d, LowSpeed>) -> Self {
        Self(RefCell::new(timer))
    }
    pub fn set_frequency(&self, frequency: u16) {
        self.0
            .borrow_mut()
            .configure(timer::config::Config {
                // 10 bits covers 100..=10000 Hz on APB without switching clock sources.
                duty: timer::config::Duty::Duty10Bit,
                clock_source: timer::LSClockSource::APBClk,
                frequency: Rate::from_hz(u32::from(frequency)),
            })
            .unwrap();
    }
}

impl TimerIFace<LowSpeed> for BuzzerTimer<'_> {
    fn freq(&self) -> Option<Rate> {
        self.0.borrow().freq()
    }
    fn configure(
        &mut self,
        config: timer::config::Config<timer::LSClockSource>,
    ) -> Result<(), timer::Error> {
        self.0.get_mut().configure(config)
    }
    fn is_configured(&self) -> bool {
        self.0.borrow().is_configured()
    }
    fn duty(&self) -> Option<timer::config::Duty> {
        self.0.borrow().duty()
    }
    fn number(&self) -> timer::Number {
        self.0.borrow().number()
    }
    fn frequency(&self) -> u32 {
        self.0.borrow().frequency()
    }
}
