use crate::aventuregraph::Aventuregraph;

pub trait Screen {
    fn draw(&self, ag: &mut Aventuregraph) -> anyhow::Result<()>;
    fn update(&mut self, ag: &mut Aventuregraph) -> anyhow::Result<()>;
}
