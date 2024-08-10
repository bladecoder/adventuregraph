use embedded_graphics::prelude::Dimensions;
use esp_idf_hal::{
    prelude::Peripherals,
    sys::{esp_timer_get_time, heap_caps_get_free_size, MALLOC_CAP_8BIT},
};

use crate::{
    agink::AGInk,
    hardware::{self, agaudio::AGAudio, agdisplay::AGDisplay, aginput::AGInput},
    ui::title_screen::TitleScreen,
};

pub struct Aventuregraph<'a> {
    pub agdisplay: AGDisplay<'a>,
    pub aginput: AGInput<'a>,
    pub agaudio: AGAudio<'a>,

    pub agink: AGInk,
}

impl<'a> Aventuregraph<'a> {
    pub fn new() -> anyhow::Result<Self> {
        let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
        println!("Free memory at start: {} kb", free_mem / 1024);
        let peripherals = Peripherals::take()?;
        let (mut agdisplay, aginput, mut agaudio) =
            hardware::peripherals_cfg::init_peripherals(peripherals);

        let width = agdisplay.get_display().bounding_box().size.width;
        let height = agdisplay.get_display().bounding_box().size.height;

        println!("Display dimensions: {}x{}", width, height);

        let load_screen = TitleScreen::new("THE INTERCEPT");

        load_screen.draw(&mut agdisplay, &mut agaudio)?;

        println!("Loading Ink story...");
        let start = unsafe { esp_timer_get_time() };
        let agink = AGInk::new()?;
        let end = unsafe { esp_timer_get_time() };
        println!("Ink story loaded in {} ms", (end - start) / 1000);
        let free_mem = unsafe { heap_caps_get_free_size(MALLOC_CAP_8BIT) };
        println!(
            "Free memory after init and load story: {} kb",
            free_mem / 1024
        );

        Ok(Self {
            agdisplay,
            aginput,
            agaudio,
            agink,
        })
    }
}
