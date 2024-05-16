use bladeink::story::Story;

// const JSON_STRING: &str = r##"{"inkVersion":21,"root":[["^Line from Ink.","\n",["done",{"#n":"g-0"}],null],"done",null],"listDefs":{}}"##;

const JSON_STRING: &str = include_str!("../assets/story.ink.json");

pub struct AGInk {
    story: Story,
}

impl AGInk {
    pub fn new() -> anyhow::Result<Self> {
        let story = Story::new(JSON_STRING)?;
        Ok(Self { story })
    }

    pub fn next_line(&mut self) -> anyhow::Result<String> {
        let line = self.story.cont()?;
        print!("{}", line);

        Ok(line)
    }

    pub fn choose(&mut self, choice: usize) -> anyhow::Result<()> {
        self.story.choose_choice_index(choice)?;
        Ok(())
    }

    pub fn has_choices(&self) -> bool {
        !self.story.get_current_choices().is_empty()
    }

    pub fn get_choices(&self) -> Vec<String> {
        let choices = self.story.get_current_choices();
        choices.iter().map(|choice| choice.text.clone()).collect()
    }

    pub fn get_num_choices(&self) -> usize {
        self.story.get_current_choices().len()
    }

    pub fn can_continue(&self) -> bool {
        self.story.can_continue()
    }
}
