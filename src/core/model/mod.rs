// pub mod models;
pub mod component;
pub mod processor;
pub mod selector;

pub trait Model {
    type Cmd;
    type Msg;
    type Context;

    fn update(&mut self, msg: &Self::Msg, ctx: &Self::Context) -> Self::Cmd;
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()>;
}
