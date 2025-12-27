pub mod models;
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
    // /// epoch的意思类似于generation，是用来区分model实例的不可变id，是执行/接收异步任务必不可少的特征
    // fn current_epoch(&self) -> u64;
}

pub trait AnyModel: ratatui::widgets::StatefulWidget {
    type Cmd;
    type Msg;
    type Context;

    fn update(&mut self, msg: &Self::Msg, ctx: &Self::Context) -> Option<Self::Cmd>;
}
