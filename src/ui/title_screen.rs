use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};

use crate::{
    adventuregraph::Adventuregraph,
    hardware::{agaudio::AGAudio, agdisplay::AGDisplay},
};

use super::screen::Screen;
use super::theme::{BG_COLOR, FG_COLOR, TITLE_FONT};

pub struct TitleScreen {
    title: String,
    character_style: MonoTextStyle<'static, Rgb565>,
}

impl TitleScreen {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            character_style: MonoTextStyle::new(TITLE_FONT, FG_COLOR),
        }
    }

    pub fn draw(&self, agdisplay: &mut AGDisplay, agaudio: &mut AGAudio) -> anyhow::Result<()> {
        agdisplay
            .get_display()
            .clear(BG_COLOR)
            .map_err(|_| anyhow::anyhow!("Clearing screen"))?;

        let title_width =
            self.title.len() as i32 * self.character_style.font.character_size.width as i32;
        let title_height = self.character_style.font.character_size.height as i32;

        Text::with_baseline(
            &self.title,
            Point::new(
                (agdisplay.get_display().bounding_box().size.width as i32 - title_width) / 2,
                (agdisplay.get_display().bounding_box().size.height as i32 - title_height) / 2,
            ),
            self.character_style,
            Baseline::Top,
        )
        .draw(agdisplay.get_display())
        .unwrap();

        agaudio.play_ok();
        Ok(())
    }
}

impl Screen for TitleScreen {
    fn draw(&self, ag: &mut Adventuregraph) -> anyhow::Result<()> {
        self.draw(&mut ag.agdisplay, &mut ag.agaudio)
    }

    fn update(&mut self, ag: &mut Adventuregraph) -> anyhow::Result<()> {
        Ok(())
    }
}
