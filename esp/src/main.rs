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
    pcnt::{
        Pcnt,
        channel::{CtrlMode, EdgeMode},
    },
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

include!(concat!(env!("OUT_DIR"), "/board.rs"));

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

struct Inputs<'d> {
    _a: Input<'d>,
    _b: Input<'d>,
    pcnt: Pcnt<'d>,
    encoder_button: Input<'d>,
    user_button: Option<Input<'d>>,
    last_count: i16,
    motion: i32,
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
    fn poll(&mut self, now: Instant) -> (i32, bool, bool) {
        // PCNT keeps counting quadrature edges while SPI drawing blocks the CPU.
        let count = self.pcnt.unit0.counter.get();
        self.motion += count.wrapping_sub(self.last_count) as i32;
        self.last_count = count;
        let step = self.motion / 4;
        self.motion %= 4;
        let enc_press = self.encoder_button_state.update(
            self.encoder_button.is_low() == ENCODER_BUTTON_ACTIVE_LOW,
            now,
        );
        let user_raw = self
            .user_button
            .as_ref()
            .is_some_and(|button| button.is_low() == USER_BUTTON_ACTIVE_LOW);
        let user_press = self.user_button_state.update(user_raw, now);
        (
            if ENCODER_REVERSE { -step } else { step },
            enc_press,
            user_press,
        )
    }
}

fn chirp(buzzer: &mut Option<Output<'_>>, delay: &Delay) {
    let Some(buzzer) = buzzer else { return };
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
    let (
        bl_pin,
        dc_pin,
        reset_pin,
        cs_pin,
        mut buzzer,
        sck_pin,
        mosi_pin,
        a,
        b,
        encoder_button,
        user_button,
    ) = board_pins!(peripherals);
    let out = OutputConfig::default();
    let mut backlight = Output::new(
        bl_pin,
        if BACKLIGHT_ON == Level::High {
            Level::Low
        } else {
            Level::High
        },
        out,
    );
    let dc = Output::new(dc_pin, Level::Low, out);
    let reset = Output::new(reset_pin, Level::High, out);
    let cs = Output::new(cs_pin, Level::High, out);
    let spi = Spi::new(
        peripherals.SPI2,
        SpiConfig::default().with_frequency(Rate::from_mhz(SPI_MHZ)),
    )
    .unwrap()
    .with_sck(sck_pin)
    .with_mosi(mosi_pin);
    let device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();
    let buffer: &'static mut [u8; 4096] = Box::leak(Box::new([0; 4096]));
    let interface = SpiInterface::new(device, dc, buffer);
    let mut display = Builder::new(ST7789, interface)
        .reset_pin(reset)
        .display_size(240, 320)
        .orientation(Orientation::new().rotate(ROTATION))
        .init(&mut Delay::new())
        .unwrap();
    backlight.set_level(BACKLIGHT_ON);

    let pcnt = Pcnt::new(peripherals.PCNT);
    pcnt.unit0.set_filter(Some(800)).unwrap();
    let signal_a = a.peripheral_input();
    let signal_b = b.peripheral_input();
    pcnt.unit0.channel0.set_ctrl_signal(signal_a.clone());
    pcnt.unit0.channel0.set_edge_signal(signal_b.clone());
    pcnt.unit0
        .channel0
        .set_ctrl_mode(CtrlMode::Reverse, CtrlMode::Keep);
    pcnt.unit0
        .channel0
        .set_input_mode(EdgeMode::Decrement, EdgeMode::Increment);
    pcnt.unit0.channel1.set_ctrl_signal(signal_b);
    pcnt.unit0.channel1.set_edge_signal(signal_a);
    pcnt.unit0
        .channel1
        .set_ctrl_mode(CtrlMode::Reverse, CtrlMode::Keep);
    pcnt.unit0
        .channel1
        .set_input_mode(EdgeMode::Increment, EdgeMode::Decrement);
    pcnt.unit0.clear();
    pcnt.unit0.resume();
    let mut inputs = Inputs {
        _a: a,
        _b: b,
        pcnt,
        encoder_button,
        user_button,
        last_count: 0,
        motion: 0,
        encoder_button_state: DebouncedButton::new(Instant::now()),
        user_button_state: DebouncedButton::new(Instant::now()),
    };

    let backend: EmbeddedBackend<_, _> =
        EmbeddedBackend::new(&mut display, EmbeddedBackendConfig::default());
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    let mut app = App::new(STORY_IMAGE, 42).unwrap();
    let mut last = Instant::now();
    let mut last_draw = last;
    loop {
        let now = Instant::now();
        app.tick((now - last).as_millis().min(u32::MAX as u64) as u32);
        last = now;
        let (step, select, user) = inputs.poll(now);
        if step != 0 {
            if app.is_animating() || app.choices().is_empty() {
                app.handle_event(AppEvent::ScrollStory(step * 2)).unwrap();
            } else if step > 0 {
                for _ in 0..step {
                    app.handle_event(AppEvent::SelectNext).unwrap();
                }
            } else {
                for _ in step..0 {
                    app.handle_event(AppEvent::SelectPrevious).unwrap();
                }
            }
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
