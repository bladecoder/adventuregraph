#![no_std]
#![no_main]

extern crate alloc;

use adventuregraph_core::{App, AppEvent, STORY_IMAGE};
use alloc::boxed::Box;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull},
    main,
    spi::master::{Config as SpiConfig, Spi},
    time::{Duration, Instant, Rate},
};
use mipidsi::{
    Builder,
    interface::SpiInterface,
    models::ST7789,
    options::{Orientation, Rotation},
};
use mousefood::prelude::*;
use ratatui::Terminal;

// The official esp-generate 1.4.0 baseline supplies the bootloader descriptor
// and an internal allocator. No ESP-IDF runtime or HAL is linked.
esp_bootloader_esp_idf::esp_app_desc!();

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

struct Inputs<'d> {
    a: Input<'d>,
    b: Input<'d>,
    encoder_button: Input<'d>,
    user_button: Input<'d>,
    state: u8,
    motion: i8,
    encoder_button_state: DebouncedButton,
    user_button_state: DebouncedButton,
}

struct DebouncedButton {
    raw: bool,
    stable: bool,
    raw_since: Instant,
}

impl DebouncedButton {
    fn new(now: Instant) -> Self {
        Self {
            raw: false,
            stable: false,
            raw_since: now,
        }
    }

    fn update(&mut self, raw: bool, now: Instant) -> bool {
        if raw != self.raw {
            self.raw = raw;
            self.raw_since = now;
        }
        if self.stable != raw && now - self.raw_since >= Duration::from_millis(25) {
            self.stable = raw;
            return raw;
        }
        false
    }
}

impl Inputs<'_> {
    fn poll(&mut self, now: Instant) -> (i8, bool, bool) {
        const TRANSITIONS: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];
        let next = (u8::from(self.a.is_high()) << 1) | u8::from(self.b.is_high());
        self.motion += TRANSITIONS[((self.state << 2) | next) as usize];
        self.state = next;
        let step = if self.motion >= 4 {
            self.motion = 0;
            1
        } else if self.motion <= -4 {
            self.motion = 0;
            -1
        } else {
            0
        };
        let enc_press = self
            .encoder_button_state
            .update(self.encoder_button.is_low(), now);
        let user_press = self
            .user_button_state
            .update(self.user_button.is_low(), now);
        (step, enc_press, user_press)
    }
}

fn chirp(buzzer: &mut Output<'_>, delay: &Delay) {
    for _ in 0..40 {
        buzzer.set_high();
        delay.delay_micros(250);
        buzzer.set_low();
        delay.delay_micros(250);
    }
}

#[main]
fn main() -> ! {
    // The ESP32-S2 has 128 KiB of internal reclaimed RAM available to this
    // allocator. SPI transfer storage is kept in this internal heap.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 131072);

    let delay = Delay::new();
    let out = OutputConfig::default();
    let mut backlight = Output::new(peripherals.GPIO12, Level::Low, out);
    let dc = Output::new(peripherals.GPIO7, Level::Low, out);
    let reset = Output::new(peripherals.GPIO5, Level::High, out);
    let cs = Output::new(peripherals.GPIO3, Level::High, out);
    let mut buzzer = Output::new(peripherals.GPIO1, Level::Low, out);
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(40)),
    )
    .unwrap()
    .with_sck(peripherals.GPIO11)
    .with_mosi(peripherals.GPIO9);
    let device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let buffer: &'static mut [u8; 4096] = Box::leak(Box::new([0; 4096]));
    let interface = SpiInterface::new(device, dc, buffer);
    let mut display = Builder::new(ST7789, interface)
        .reset_pin(reset)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(Rotation::Deg180))
        .init(&mut Delay::new())
        .unwrap();
    backlight.set_high();

    let input_cfg = InputConfig::default().with_pull(Pull::Up);
    let a = Input::new(peripherals.GPIO38, input_cfg);
    let b = Input::new(peripherals.GPIO40, input_cfg);
    let state = (u8::from(a.is_high()) << 1) | u8::from(b.is_high());
    let mut inputs = Inputs {
        a,
        b,
        encoder_button: Input::new(peripherals.GPIO36, input_cfg),
        user_button: Input::new(peripherals.GPIO37, input_cfg),
        state,
        motion: 0,
        encoder_button_state: DebouncedButton::new(Instant::now()),
        user_button_state: DebouncedButton::new(Instant::now()),
    };

    let backend: EmbeddedBackend<_, _> =
        EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new(STORY_IMAGE, 42).unwrap();
    let mut last = Instant::now();
    let mut last_draw = last;
    loop {
        let now = Instant::now();
        app.tick((now - last).as_millis().min(u32::MAX as u64) as u32);
        last = now;
        let (step, select, user) = inputs.poll(now);
        if step != 0 {
            let event = if app.is_animating() || app.choices().is_empty() {
                AppEvent::ScrollStory(step as i32 * 2)
            } else if step > 0 {
                AppEvent::SelectNext
            } else {
                AppEvent::SelectPrevious
            };
            app.handle_event(event).unwrap();
        }
        if select {
            app.handle_event(AppEvent::ChooseSelected).unwrap();
            chirp(&mut buzzer, &delay);
        }
        if user {
            app.handle_event(AppEvent::JumpStoryStart).unwrap();
            chirp(&mut buzzer, &delay);
        }
        if now - last_draw >= Duration::from_millis(50) {
            terminal.draw(|frame| app.draw(frame)).unwrap();
            last_draw = now;
        }
        delay.delay_millis(2);
    }
}
