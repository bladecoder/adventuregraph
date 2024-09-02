// Audio is output through a simple piezo buzzer connected to one of the GPIO pins.

use esp_idf_hal::{
    delay::FreeRtos,
    gpio::OutputPin,
    ledc::{config::TimerConfig, LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver, Resolution},
    peripheral::Peripheral,
    sys::{ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_set_freq, EspError},
    units::Hertz,
};

pub struct AGAudio<'d> {
    channel: LedcDriver<'d>, // Add named lifetime parameter
    tone_duty: u32,
}

impl<'d> AGAudio<'d> {
    pub fn new<C: LedcChannel, B: LedcTimer + 'd>(
        _channel: impl Peripheral<P = C> + 'd,
        timer: impl Peripheral<P = B> + 'd,
        pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, EspError> {
        let freq = Hertz::from(500);
        let res = Resolution::Bits10;
        let channel = LedcDriver::new(
            _channel,
            LedcTimerDriver::new(timer, &TimerConfig::new().resolution(res).frequency(freq))?,
            pin,
        )?;
        let tone_duty = channel.get_max_duty() / 2;
        Ok(Self { channel, tone_duty })
    }

    pub fn play_ok(&mut self) {
        self.tone(300, 100);
    }

    pub fn play_error(&mut self) {
        self.tone(100, 200);
    }

    pub fn tone(&mut self, frequency: u32, delay: u32) {
        let _ = self.channel.set_duty(self.tone_duty);
        unsafe {
            ledc_set_freq(
                ledc_mode_t_LEDC_LOW_SPEED_MODE,
                self.channel.timer(),
                frequency,
            );
        }
        FreeRtos::delay_ms(delay);
        let _ = self.channel.set_duty(0);
    }

    pub fn melody(&mut self, notes: [u32; 12], cycles: u32) {
        for _ in 0..cycles {
            for note in notes {
                self.tone(note, 50);
            }
        }
    }
}
