use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent};

use crate::core::{
    cmd::Cmd,
    model::{component::Focus, selector::SelectModel},
    msg::Msg,
    traits::Model,
};
use color_eyre::{Result as Res, eyre::Ok};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget as _},
};

#[derive(Debug, Default)]
pub struct Processor {
    pub inner: SelectModel,
}

impl Processor {
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
            // .title(title.centered())
            .title_bottom(instructions.centered())
            // .border_set(border::THICK)
            ;

        let inner_area = block.inner(area);
        block.render(area, buf);

        let columns = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ]);
        let [left_area, mid_area, right_area] = columns.areas(inner_area);

        let left_items = self.inner.render_list(Focus::Left);
        let left_style = if self.inner.cursor.focus == Focus::Left {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        Paragraph::new(left_items)
            .block(Block::bordered().title("Left").border_style(left_style))
            .render(left_area, buf);

        let mid_items = self.inner.render_list(Focus::Mid);
        let mid_style = if self.inner.cursor.focus == Focus::Mid {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        Paragraph::new(mid_items)
            .block(Block::bordered().title("Mid").border_style(mid_style))
            .render(mid_area, buf);

        let right_items = self.inner.render_list(Focus::Right);
        let right_style = if self.inner.cursor.focus == Focus::Right {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        Paragraph::new(right_items)
            .block(Block::bordered().title("Right").border_style(right_style))
            .render(right_area, buf);
    }
}
