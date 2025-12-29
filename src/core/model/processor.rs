use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};

use crate::core::{
    cmd::Cmd,
    model::{
        AnyModel,
        component::{ScrollList, input::InputBox},
        selector::SelectModel,
    },
    msg::Msg,
    traits::Model,
};
use color_eyre::{Result as Res, eyre::Ok};
use ratatui::{
    Frame,
    layout::Layout,
    macros::constraints,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph, Widget as _},
};

#[derive(Debug, Default)]
pub enum InProcess {
    /// alias 'n'
    #[default]
    None,
    /// alias 'd'
    Delete,
    /// alias 't' 'tr'
    Trash,
    /// alias 'o' 'og'
    Organize(InputBox),
    /// alias 'm' 'mv'
    Move(InputBox),
    /// alias 'c' 'cp'
    Copy(InputBox),
    /// not yet impl
    Zip(),
    Rename,
}

impl std::fmt::Display for InProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use InProcess::*;
        match self {
            None => write!(f, ""),
            Delete => write!(f, "Delete"),
            Trash => write!(f, "Trash"),
            Organize(path) => write!(f, "Organize:`{:?}`", path),
            Move(path) => write!(f, "Move:`{:?}`", path),
            Copy(path) => write!(f, "Copy:`{:?}`", path),
            Zip() => todo!(),
            x => write!(f, "{:?}", x),
        }
    }
}

#[derive(Debug, Default)]
pub struct Processor {
    mid: ScrollList,
    // mid_process : InProcess,
    left: ScrollList,
    left_proc: InProcess,
    right: ScrollList,
    right_proc: InProcess,

    /// default false, which is left
    focus_right: bool,

    /// 用来指示列表切换是否被锁住了，当被锁住时，说明正在输入，应该使用input box逻辑
    is_locked: bool,
}

impl Processor {
    pub fn new(inner: SelectModel) -> Self {
        Self {
            left: inner.left.clone(),
            right: inner.right.clone(),
            focus_right: false,
            ..Default::default()
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Res<Cmd> {
        match key_event.code {
            KeyCode::Left => {
                self.focus_right = !self.focus_right;
            }
            KeyCode::Right => {
                self.focus_right = !self.focus_right;
            }
            KeyCode::Char('o') => {
                self.is_locked = true;
            }
            KeyCode::Char('c') => {
                self.is_locked = true;
            }
            KeyCode::Char('d') => {
                self.is_locked = true;
                if self.focus_right {
                    self.right_proc = InProcess::Delete;
                } else {
                    self.left_proc = InProcess::Delete;
                }
            }
            KeyCode::Char('q') => return Ok(Cmd::Exit),
            _ => {}
        }
        Ok(Cmd::None)
    }

    fn render_list_panel(
        &self,
        list: &ScrollList,
        proc: &InProcess,
        is_focused: bool,
        side_name: &str,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let title = if matches!(proc, InProcess::None) {
            if is_focused {
                format!("{} - Processing", side_name)
            } else {
                side_name.to_string()
            }
        } else {
            format!("{} - *{}", side_name, proc)
        };
        list.render(is_focused, None, &title).render(area, buf);
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let area = f.area();
        let buf = f.buffer_mut();
    }
}

impl Model for Processor {
    fn update(
        &mut self,
        msg: crate::core::msg::Msg,
        _: &crate::core::context::Context,
    ) -> crate::core::cmd::Cmd {
        match msg {
            Msg::Exit => Cmd::Exit,
            Msg::Key(key_event) => self
                .handle_key_event(key_event)
                .unwrap_or_else(|e| Cmd::Error(e.to_string())),

            _ => Cmd::None,
        }
    }

    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let instructions = Line::from(vec![
            " Delete ".into(),
            "<D>".blue().bold(),
            " Organize ".into(),
            "<O>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);

        let block = Block::bordered()
            // .title(Line::from("Processing...").centered())
            .title_bottom(instructions.centered())
            // .border_set(border::THICK)
            ;

        let inner_area = block.inner(area);
        block.render(area, buf);

        let columns = Layout::horizontal(constraints![==50%, ==50%]);
        let [left_area, right_area] = columns.areas(inner_area);

        self.render_list_panel(
            &self.left,
            &self.left_proc,
            !self.focus_right,
            "Left",
            left_area,
            buf,
        );
        self.render_list_panel(
            &self.right,
            &self.right_proc,
            self.focus_right,
            "Right",
            right_area,
            buf,
        );
    }
}

impl AnyModel for Processor {
    type Cmd = crate::core::cmd::Cmd;
    type Msg = crate::core::msg::Msg;
    type Context = crate::core::context::Context;

    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()> {
        todo!()
    }

    fn update(&mut self, msg: &Self::Msg, _: &Self::Context) -> Option<Self::Cmd> {
        todo!()
    }
}
