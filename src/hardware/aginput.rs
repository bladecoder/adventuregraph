use esp_idf_hal::{
    gpio::{Input, InterruptType, PinDriver, Pull},
    sys::EspError,
};

use std::sync::atomic::{AtomicBool, Ordering};

use crate::hardware::peripherals_cfg::{PinA, PinB, PinSw};

static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
static LEFT: AtomicBool = AtomicBool::new(false);
static RIGHT: AtomicBool = AtomicBool::new(false);
static ENC_TMP: AtomicBool = AtomicBool::new(false);

pub struct AGInput<'d> {
    enc_sw: PinDriver<'d, PinSw, Input>,
    enc_a: PinDriver<'d, PinA, Input>,
    enc_b: PinDriver<'d, PinB, Input>,
}

impl AGInput<'_> {
    pub fn new(pin_sw: PinSw, pin_a: PinA, pin_b: PinB) -> Result<Self, EspError> {
        let mut enc_sw = PinDriver::input(pin_sw)?;
        enc_sw.set_pull(Pull::Up)?;

        enc_sw.set_interrupt_type(InterruptType::PosEdge)?;
        unsafe {
            enc_sw.subscribe(|| {
                BUTTON_PRESSED.store(true, Ordering::Relaxed);
            })
        }?;

        enc_sw.enable_interrupt()?;

        let mut enc_a = PinDriver::input(pin_a)?;
        enc_a.set_pull(Pull::Up)?;

        enc_a.set_interrupt_type(InterruptType::PosEdge)?;
        unsafe {
            enc_a.subscribe(|| {
                if ENC_TMP.load(Ordering::Relaxed) {
                    RIGHT.store(true, Ordering::Relaxed);
                    ENC_TMP.store(false, Ordering::Relaxed);
                } else {
                    ENC_TMP.store(true, Ordering::Relaxed);
                }
            })
        }?;
        enc_a.enable_interrupt()?;

        let mut enc_b = PinDriver::input(pin_b)?;
        enc_b.set_pull(Pull::Up)?;

        enc_b.set_interrupt_type(InterruptType::PosEdge)?;
        unsafe {
            enc_b.subscribe(|| {
                if ENC_TMP.load(Ordering::Relaxed) {
                    LEFT.store(true, Ordering::Relaxed);
                    ENC_TMP.store(false, Ordering::Relaxed);
                } else {
                    ENC_TMP.store(true, Ordering::Relaxed);
                }
            })
        }?;
        enc_b.enable_interrupt()?;

        Ok(AGInput {
            enc_sw,
            enc_a,
            enc_b,
        })
    }

    pub fn consume_sw(&mut self) -> bool {
        let pressed = BUTTON_PRESSED.load(Ordering::Relaxed);

        if pressed {
            BUTTON_PRESSED.store(false, Ordering::Relaxed);
            self.enc_sw.enable_interrupt().unwrap();
        }

        pressed
    }

    pub fn consume_left(&mut self) -> bool {
        let v = LEFT.load(Ordering::Relaxed);

        if v {
            LEFT.store(false, Ordering::Relaxed);
            self.enc_a.enable_interrupt().unwrap();
            self.enc_b.enable_interrupt().unwrap();
        }

        v
    }

    pub fn consume_right(&mut self) -> bool {
        let v = RIGHT.load(Ordering::Relaxed);

        if v {
            RIGHT.store(false, Ordering::Relaxed);
            self.enc_a.enable_interrupt().unwrap();
            self.enc_b.enable_interrupt().unwrap();
        }

        v
    }
}
