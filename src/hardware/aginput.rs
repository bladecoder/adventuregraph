use esp_idf_hal::{
    gpio::{Input, InterruptType, Pin, PinDriver, Pull},
    peripheral::Peripheral,
    sys::{
        gpio_get_level, gpio_mode_t_GPIO_MODE_INPUT, gpio_pullup_en, gpio_set_direction, EspError,
    },
    timer::{config::Config, Timer, TimerDriver},
};

use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};

use crate::hardware::peripherals_cfg::{PinA, PinB, PinSw};

static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);
const ROT_ENC_TABLE: [u8; 16] = [0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0];
static mut PREV_NEXT_CODE: u8 = 0;
static mut STORE: u16 = 0;

// LEFT = -1, RIGHT = 1, NONE = 0
static VAL: AtomicI8 = AtomicI8::new(0);

pub enum Direction {
    Left,
    Right,
    None,
}

pub struct AGInput<'d> {
    enc_sw: PinDriver<'d, PinSw, Input>,
    _timer: TimerDriver<'d>,
}

impl<'d> AGInput<'d> {
    pub fn new<TIMER: Timer>(
        pin_sw: PinSw,
        pin_a: PinA,
        pin_b: PinB,
        timer: impl Peripheral<P = TIMER> + 'd,
    ) -> Result<Self, EspError> {
        let mut enc_sw = PinDriver::input(pin_sw)?;
        enc_sw.set_pull(Pull::Up)?;

        enc_sw.set_interrupt_type(InterruptType::PosEdge)?;
        unsafe {
            enc_sw.subscribe(|| {
                BUTTON_PRESSED.store(true, Ordering::Relaxed);
            })
        }?;

        enc_sw.enable_interrupt()?;

        let pin_a_n = pin_a.pin();
        let pin_b_n = pin_b.pin();

        unsafe { gpio_pullup_en(pin_a_n) };
        unsafe { gpio_set_direction(pin_a_n, gpio_mode_t_GPIO_MODE_INPUT) };
        unsafe { gpio_pullup_en(pin_b_n) };
        unsafe { gpio_set_direction(pin_b_n, gpio_mode_t_GPIO_MODE_INPUT) };

        let config = Config::new().auto_reload(true);

        let mut timer = TimerDriver::new(timer, &config)?;

        // alarm each 900hz
        let value = 1_000_000 / 200;
        timer.set_alarm(value)?;

        unsafe {
            timer.subscribe(move || {
                let a = gpio_get_level(pin_a_n);
                let b = gpio_get_level(pin_b_n);

                let val = AGInput::read_rotary(a, b);

                if val != 0 {
                    VAL.store(val, Ordering::Relaxed);
                }
            })
        }?;

        timer.enable_alarm(true)?;
        timer.enable_interrupt()?;
        timer.enable(true)?;

        Ok(AGInput {
            enc_sw,
            _timer: timer,
        })
    }

    // A valid CW or  CCW move returns 1/-1, invalid returns 0.
    fn read_rotary(a: i32, b: i32) -> i8 {
        unsafe {
            PREV_NEXT_CODE <<= 2;
            if a == 1 {
                PREV_NEXT_CODE |= 0x02
            };
            if b == 1 {
                PREV_NEXT_CODE |= 0x01
            };
            PREV_NEXT_CODE &= 0x0f;

            // If valid then store as 16 bit data.
            if ROT_ENC_TABLE[PREV_NEXT_CODE as usize] == 1 {
                STORE <<= 4;
                STORE |= PREV_NEXT_CODE as u16;

                if (STORE & 0xff) == 0x2b {
                    return -1;
                };
                if (STORE & 0xff) == 0x17 {
                    return 1;
                };
            }
            0
        }
    }

    pub fn consume_sw(&mut self) -> bool {
        let pressed = BUTTON_PRESSED.load(Ordering::Relaxed);

        if pressed {
            BUTTON_PRESSED.store(false, Ordering::Relaxed);
            self.enc_sw.enable_interrupt().unwrap();
        }

        pressed
    }

    pub fn consume_rotary(&mut self) -> Direction {
        let val = VAL.load(Ordering::Relaxed);

        if val == -1 {
            VAL.store(0, Ordering::Relaxed);
            return Direction::Left;
        } else if val == 1 {
            VAL.store(0, Ordering::Relaxed);
            return Direction::Right;
        }

        Direction::None
    }
}
