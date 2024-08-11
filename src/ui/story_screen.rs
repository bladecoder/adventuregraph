use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::*;

use esp_idf_hal::{delay::FreeRtos, sys::rand};

use crate::{
    aventuregraph::Aventuregraph,
    hardware::{self},
};

use super::{screen::Screen, scrolled_text::ScrolledText};

const CHOOSE_RANDOM: bool = false;

pub struct StoryScreen {
    scrolled_text: ScrolledText,
    choices_displayed: bool,
    selected_choice: u8,
}

impl StoryScreen {
    pub fn new() -> Self {
        Self {
            scrolled_text: ScrolledText::new(),
            choices_displayed: false,
            selected_choice: 0,
        }
    }
}

impl Screen for StoryScreen {
    fn draw(&self, ag: &mut Aventuregraph) -> anyhow::Result<()> {
        ag.agdisplay
            .get_display()
            .clear(Rgb565::BLACK)
            .map_err(|_| anyhow::anyhow!("Clearing screen"))?;

        Ok(())
    }

    fn update(&mut self, ag: &mut Aventuregraph) -> anyhow::Result<()> {
        if ag.agink.can_continue() {
            let line = ag.agink.next_line()?;
            self.scrolled_text.add_text(&mut ag.agdisplay, &line);
        }

        if ag.agink.has_choices() && !self.choices_displayed {
            self.choices_displayed = true;
            self.selected_choice = 0;
            let choices = ag.agink.get_choices();

            self.scrolled_text.add_choices(&mut ag.agdisplay, &choices);

            // choose a choice randomly
            if CHOOSE_RANDOM {
                let random_choice =
                    (unsafe { rand() } % ag.agink.get_num_choices() as i32) as usize;
                ag.agink.choose(random_choice)?;
                self.scrolled_text
                    .select_choice(&mut ag.agdisplay, random_choice);
                self.choices_displayed = false;
                FreeRtos::delay_ms(1000u32);
            }
        }

        if !ag.agink.can_continue() && !ag.agink.has_choices() {
            self.scrolled_text.add_text(&mut ag.agdisplay, "The End");
            println!("The End");
            loop {
                FreeRtos::delay_ms(1000u32);
            }
        }

        if ag.aginput.consume_sw() {
            println!("Button pressed");

            if self.choices_displayed {
                ag.agink.choose(self.selected_choice as usize)?;
                self.choices_displayed = false;
                self.scrolled_text.clear_choices(&mut ag.agdisplay);
            }
        }

        match ag.aginput.consume_rotary() {
            hardware::aginput::Direction::Left => {
                println!("Left");

                if self.choices_displayed {
                    self.selected_choice =
                        (self.selected_choice + 1) % ag.agink.get_num_choices() as u8;
                    self.scrolled_text
                        .select_choice(&mut ag.agdisplay, self.selected_choice as usize);
                }
            }
            hardware::aginput::Direction::Right => {
                println!("Right");

                if self.choices_displayed {
                    self.selected_choice =
                        (self.selected_choice + ag.agink.get_num_choices() as u8 - 1)
                            % ag.agink.get_num_choices() as u8;
                    self.scrolled_text
                        .select_choice(&mut ag.agdisplay, self.selected_choice as usize);
                }
            }
            hardware::aginput::Direction::None => {}
        }
        Ok(())
    }
}
