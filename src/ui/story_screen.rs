use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::prelude::*;

use esp_idf_hal::{delay::FreeRtos, sys::rand};

use crate::{
    adventuregraph::Adventuregraph,
    hardware::{self},
};

use super::theme::BG_COLOR;
use super::{screen::Screen, scrolled_text::ScrolledText};

const CHOOSE_RANDOM: bool = false;

#[derive(Debug, PartialEq, Clone, Copy)]
enum StoryScreenState {
    Text,
    Choices,
    ScrollLock,
    End,
}

pub struct StoryScreen {
    scrolled_text: ScrolledText,
    state: StoryScreenState,
    selected_choice: u8,
}

impl StoryScreen {
    pub fn new() -> Self {
        Self {
            scrolled_text: ScrolledText::new(),
            state: StoryScreenState::Text,
            selected_choice: 0,
        }
    }
}

impl Screen for StoryScreen {
    fn draw(&self, ag: &mut Adventuregraph) -> anyhow::Result<()> {
        ag.agdisplay
            .get_display()
            .clear(BG_COLOR)
            .map_err(|_| anyhow::anyhow!("Clearing screen"))?;

        Ok(())
    }

    fn update(&mut self, ag: &mut Adventuregraph) -> anyhow::Result<()> {
        if ag.agink.can_continue() && self.state == StoryScreenState::Text {
            let line = ag.agink.next_line()?;
            self.scrolled_text.add_text(&mut ag.agdisplay, &line);
            if self.scrolled_text.is_scroll_locked(&mut ag.agdisplay) {
                self.state = StoryScreenState::ScrollLock;
                println!("Scroll locked!!");
            }
        }

        if ag.agink.has_choices() && self.state == StoryScreenState::Text {
            self.state = StoryScreenState::Choices;
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
                self.state = StoryScreenState::Text;
                FreeRtos::delay_ms(1000u32);
            }
        }

        if !ag.agink.can_continue()
            && !ag.agink.has_choices()
            && self.state == StoryScreenState::Text
        {
            self.state = StoryScreenState::End;
            self.scrolled_text
                .add_text(&mut ag.agdisplay, "\n    The End\n");
            println!("The End");
            let choice = vec!["--restart--".to_owned()];
            self.scrolled_text.add_choices(&mut ag.agdisplay, &choice);
        }

        if ag.aginput.consume_sw() {
            println!("Button pressed");

            match self.state {
                StoryScreenState::Choices => {
                    ag.agink.choose(self.selected_choice as usize)?;
                    self.scrolled_text.clear_choices(&mut ag.agdisplay);
                }
                StoryScreenState::ScrollLock => {
                    self.scrolled_text.more(&mut ag.agdisplay);
                }
                StoryScreenState::End => {
                    self.scrolled_text.clear(&mut ag.agdisplay);
                    ag.agink.restart()?;
                }

                _ => {}
            }

            self.state = StoryScreenState::Text;
        }

        match ag.aginput.consume_rotary() {
            hardware::aginput::Direction::Left => {
                println!("Left");

                if self.state == StoryScreenState::Choices {
                    self.selected_choice =
                        (self.selected_choice + 1) % ag.agink.get_num_choices() as u8;
                    self.scrolled_text
                        .select_choice(&mut ag.agdisplay, self.selected_choice as usize);
                }
            }
            hardware::aginput::Direction::Right => {
                println!("Right");

                if self.state == StoryScreenState::Choices {
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
