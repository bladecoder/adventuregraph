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
}
