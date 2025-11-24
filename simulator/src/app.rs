use anyhow::{Context, Result};
use bladeink::story::Story;

const STORY_JSON: &str = include_str!("../../assets/story.ink.json");
const RESTART_CHOICE: &str = "-- restart --";

pub struct App {
    story: Story,
    lines: Vec<String>,
    choices: Vec<String>,
    selected_choice: usize,
    finished: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let story = Story::new(STORY_JSON).context("loading Ink story")?;
        let mut app = Self {
            story,
            lines: Vec::new(),
            choices: Vec::new(),
            selected_choice: 0,
            finished: false,
        };
        app.advance_story()?;
        Ok(app)
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn choices(&self) -> &[String] {
        &self.choices
    }

    pub fn has_choices(&self) -> bool {
        !self.choices.is_empty()
    }

    pub fn selected_choice(&self) -> Option<usize> {
        if self.choices.is_empty() {
            None
        } else {
            Some(self.selected_choice)
        }
    }

    pub fn select_next(&mut self) {
        if self.choices.is_empty() || self.selected_choice + 1 >= self.choices.len() {
            return;
        }
        self.selected_choice += 1;
    }

    pub fn select_previous(&mut self) {
        if self.choices.is_empty() || self.selected_choice == 0 {
            return;
        }
        self.selected_choice -= 1;
    }

    pub fn select_choice(&mut self, index: usize) {
        if index < self.choices.len() {
            self.selected_choice = index;
        }
    }

    pub fn choose_selected(&mut self) -> Result<()> {
        if self.choices.is_empty() {
            return Ok(());
        }

        if self.finished {
            self.restart()?;
        } else {
            self.story
                .choose_choice_index(self.selected_choice)
                .context("applying Ink choice")?;
        }

        self.advance_story()
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    fn advance_story(&mut self) -> Result<()> {
        while self.story.can_continue() {
            let next_line = self
                .story
                .cont()
                .context("reading Ink line")?
                .trim_end_matches(&['\r', '\n'][..])
                .to_owned();

            if next_line.is_empty() {
                continue;
            }

            self.lines.push(next_line);
        }

        self.choices = self
            .story
            .get_current_choices()
            .iter()
            .map(|choice| choice.text.clone())
            .collect();

        if self.choices.is_empty() && !self.story.can_continue() {
            self.finished = true;
            self.choices = vec![RESTART_CHOICE.to_owned()];

            if self
                .lines
                .last()
                .map(|line| line.trim().eq_ignore_ascii_case("the end"))
                != Some(true)
            {
                self.lines.push("The End".to_owned());
            }
        } else {
            self.finished = false;
        }

        self.selected_choice = 0;
        Ok(())
    }

    fn restart(&mut self) -> Result<()> {
        self.story.reset_state().context("resetting Ink story")?;
        self.lines.clear();
        self.choices.clear();
        self.selected_choice = 0;
        self.finished = false;
        self.advance_story()
    }
}
