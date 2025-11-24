use esp_idf_hal::{
    gpio::{InputPin, OutputPin, Pin},
    peripheral::Peripheral,
    sys::{
        EspError, gpio_get_level, gpio_mode_t_GPIO_MODE_INPUT, gpio_pullup_en, gpio_set_direction,
    },
    timer::{Timer, TimerDriver, config::Config},
};

use std::sync::atomic::{AtomicI8, AtomicI32, Ordering};

use crate::hardware::peripherals_cfg::{PinA, PinB, PinSw};

use super::peripherals_cfg::Btn1;

static ENCSW_PRESSED: AtomicI32 = AtomicI32::new(0);
static BUTTON1_PRESSED: AtomicI32 = AtomicI32::new(0);
const ROT_ENC_TABLE: [u8; 16] = [0, 1, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 1, 0];
static mut PREV_NEXT_CODE: u8 = 0;
static mut STORE: u16 = 0;

// LEFT = -1, RIGHT = 1, NONE = 0
static VAL: AtomicI8 = AtomicI8::new(0);

#[derive(Debug, Clone)]
pub struct Inputs {
    pub rotary: Direction,
    pub encsw: bool,
    pub btn1: bool,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Left,
    Right,
    None,
}

pub struct AGInput<'d> {
    _timer: TimerDriver<'d>,
}

impl<'d> AGInput<'d> {
    pub fn new<TIMER: Timer>(
        pin_sw: PinSw,
        pin_a: PinA,
        pin_b: PinB,
        pin_btn1: impl InputPin,
        timer: impl Peripheral<P = TIMER> + 'd,
    ) -> Result<Self, EspError> {
        let pin_a_n = pin_a.pin();
        let pin_b_n = pin_b.pin();
        let pin_btn1 = pin_btn1.pin();
        let pin_encsw = pin_sw.pin();

        unsafe { gpio_pullup_en(pin_a_n) };
        unsafe { gpio_set_direction(pin_a_n, gpio_mode_t_GPIO_MODE_INPUT) };
        unsafe { gpio_pullup_en(pin_b_n) };
        unsafe { gpio_set_direction(pin_b_n, gpio_mode_t_GPIO_MODE_INPUT) };
        unsafe { gpio_pullup_en(pin_encsw) };
        unsafe { gpio_set_direction(pin_encsw, gpio_mode_t_GPIO_MODE_INPUT) };
        unsafe { gpio_pullup_en(pin_btn1) };
        unsafe { gpio_set_direction(pin_btn1, gpio_mode_t_GPIO_MODE_INPUT) };

        let config = Config::new().auto_reload(true);

        let mut timer = TimerDriver::new(timer, &config)?;

        // alarm each 1Mhz. with this counter we get 200hz
        let value = 1_000_000 / 200;
        timer.set_alarm(value)?;

        unsafe {
            timer.subscribe(move || {
                let a = gpio_get_level(pin_a_n);
                let b = gpio_get_level(pin_b_n);
                let btn1 = gpio_get_level(pin_btn1);
                let encsw = gpio_get_level(pin_encsw);

                let val = AGInput::read_rotary(a, b);

                if val != 0 {
                    VAL.store(val, Ordering::Relaxed);
                }

                // Debounce buttons
                let vbtn1 = BUTTON1_PRESSED.load(Ordering::Relaxed);

                if btn1 == 0 {
                    BUTTON1_PRESSED.store(vbtn1 + 1, Ordering::Relaxed);
                } else if vbtn1 > 10 {
                    BUTTON1_PRESSED.store(-1, Ordering::Relaxed);
                }

                let vencsw = ENCSW_PRESSED.load(Ordering::Relaxed);

                if encsw == 0 {
                    ENCSW_PRESSED.store(vencsw + 1, Ordering::Relaxed);
                } else if vencsw > 10 {
                    ENCSW_PRESSED.store(-1, Ordering::Relaxed);
                }
            })
        }?;

        timer.enable_alarm(true)?;
        timer.enable_interrupt()?;
        timer.enable(true)?;

        Ok(AGInput { _timer: timer })
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

    pub fn get_inputs(&mut self) -> Inputs {
        Inputs {
            rotary: self.consume_rotary(),
            encsw: self.consume_sw(),
            btn1: self.consume_btn1(),
        }
    }

    fn consume_sw(&mut self) -> bool {
        let v = ENCSW_PRESSED.load(Ordering::Relaxed);
        let pressed = v == -1;

        if pressed {
            ENCSW_PRESSED.store(0, Ordering::Relaxed);
        }

        pressed
    }

    fn consume_btn1(&mut self) -> bool {
        let v = BUTTON1_PRESSED.load(Ordering::Relaxed);
        let pressed = v == -1;

        if pressed {
            BUTTON1_PRESSED.store(0, Ordering::Relaxed);
        }

        pressed
    }

    fn consume_rotary(&mut self) -> Direction {
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
