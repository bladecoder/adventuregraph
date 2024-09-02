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
}

impl StoryScreen {
    pub fn new() -> Self {
        Self {
            scrolled_text: ScrolledText::new(),
            state: StoryScreenState::Text,
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
            println!("Rotary Button pressed");

            if !self.scrolled_text.is_at_end(&mut ag.agdisplay) {
                self.scrolled_text.goto_end(&mut ag.agdisplay);
                return Ok(());
            }

            match self.state {
                StoryScreenState::Choices => {
                    ag.agink.choose(self.scrolled_text.get_selected_choice())?;
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
                println!("Down");

                if self.state != StoryScreenState::Text {
                    self.scrolled_text.down(&mut ag.agdisplay);
                }
            }
            hardware::aginput::Direction::Right => {
                println!("Up");

                if self.state != StoryScreenState::Text {
                    self.scrolled_text.up(&mut ag.agdisplay);
                }
            }
            hardware::aginput::Direction::None => {}
        }

        if ag.aginput.consume_btn1() {
            println!("Button1 pressed");

            ag.agaudio.play_ok();
        }

        Ok(())
    }
}
