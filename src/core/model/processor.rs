use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};

use crate::core::{
    cmd::Cmd,
    model::{
        component::ScrollList,
        selector::SelectModel,
    },
    msg::Msg,
    traits::Model,
};
use color_eyre::{Result as Res, eyre::Ok};
use ratatui::{
    layout::Layout,
    macros::constraints,
    style::Stylize,
    text::Line,
    widgets::{Block, Widget as _},
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
    Organize(String),
    /// alias 'm' 'mv'
    Move(PathBuf),
    /// alias 'c' 'cp'
    Copy,
    /// not yet impl
    Zip(),
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

    pub inner: SelectModel,
}

impl Processor {
    pub fn new(inner: SelectModel) -> Self {
        Self {
            inner: inner.clone(),
            left: inner.left.clone(),
            right: inner.right.clone(),
            focus_right: false,
            ..Default::default()
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Res<Cmd> {
        match key_event.code {
            KeyCode::Left => {
                self.inner.cursor = self.inner.cursor.move_left();
            }
            KeyCode::Right => {
                self.inner.cursor = self.inner.cursor.move_right();
            }
            KeyCode::Up => {
                self.inner.cursor = self.inner.cursor.shift_up();
            }
            KeyCode::Down => {
                self.inner.cursor = self.inner.cursor.shift_down();
            }
            KeyCode::Char('o') => {}
            KeyCode::Char('d') => {}
            KeyCode::Char('q') => return Ok(Cmd::Exit),
            _ => {}
        }
        Ok(Cmd::None)
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

        // waiting
        if matches!(self.left_proc, InProcess::None) {
            self.left
                .render(!self.focus_right, None, "Left")
                .render(left_area, buf);
        }
        // processing
        else if !self.focus_right {
            self.left
                .render(!self.focus_right, None, "Left - Processing")
                .render(left_area, buf);
        }
        // processed but not applied
        else {
            self.left
                .render(
                    !self.focus_right,
                    None,
                    &format!("Left - *{:?}", self.left_proc),
                )
                .render(left_area, buf);
        }

        // waiting
        if matches!(self.right_proc, InProcess::None) {
            self.right
                .render(self.focus_right, None, "Right")
                .render(right_area, buf);
        }
        // processing
        else if self.focus_right {
            self.right
                .render(self.focus_right, None, "Right - Processing")
                .render(right_area, buf);
        }
        // processed but not applied
        else {
            self.right
                .render(
                    self.focus_right,
                    None,
                    &format!("Right - *{:?}", self.right_proc),
                )
                .render(right_area, buf);
        }
    }
}
