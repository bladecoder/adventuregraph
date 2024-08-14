use crate::adventuregraph::Adventuregraph;

pub trait Screen {
    fn draw(&self, ag: &mut Adventuregraph) -> anyhow::Result<()>;
    fn update(&mut self, ag: &mut Adventuregraph) -> anyhow::Result<()>;
}
