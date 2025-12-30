use crate::core::{cmd::Cmd, context::Context, model::AnyModel, msg::Msg};

pub struct CheckModel {}

impl AnyModel for CheckModel {
    type Context = Context;
    type Cmd = Cmd;
    type Msg = Msg;
    fn draw(
        &mut self,
        _frame: &mut ratatui::Frame,
        _area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()> {
        todo!()
    }

    fn update(&mut self, _msg: &Self::Msg, _ctx: &Self::Context) -> Self::Cmd {
        todo!()
    }
}
