// pub mod models;
pub mod processor;
pub mod selector;

pub mod component;

pub trait Model {
    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer);
    fn update(
        &mut self,
        msg: crate::core::msg::Msg,
        ctx: &crate::core::context::Context,
    ) -> crate::core::cmd::Cmd;
}

pub trait AnyModel {
    type Cmd;
    type Msg;
    type Context;

    fn update(&mut self, msg: &Self::Msg, ctx: &Self::Context) -> Option<Self::Cmd>;
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()>;
}
