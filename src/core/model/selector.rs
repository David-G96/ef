use std::{collections::VecDeque, env, path::PathBuf};

use crate::core::file_ops::FileOperator;
use crate::core::{
    cmd::Cmd,
    model::component::{Cursor, FileItem, Focus, History, ScrollList},
    msg::Msg,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, List, StatefulWidget, Widget as _},
};

use color_eyre::{Result as Res, eyre::Ok};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum SelectOperation {
    // 移动操作：记录从哪来、到哪去，以及它在原列表中的原始位置（用于完美还原）
    Move {
        item_id: u64,
        from_list: Focus,
        from_index: usize, // 撤销时放回原位所需
        to_list: Focus,
    },
}

/// 分三列的工作区！（左右中）
#[derive(Debug, Default, Clone)]
pub struct SelectModel {
    pub(crate) path: PathBuf,
    pub(crate) mid: ScrollList,
    pub(crate) left: ScrollList,
    pub(crate) right: ScrollList,
    pub(crate) cursor: Cursor,
    pub(crate) history: History<SelectOperation>,
}

impl SelectModel {
    pub fn new(file_op: &FileOperator) -> Res<Self> {
        let current_path = env::current_dir()?;
        let res = file_op.list_items(&current_path)?;
        Ok(Self::new_with(current_path, res))
    }

    fn new_with(path: PathBuf, pending: VecDeque<FileItem>) -> Self {
        Self {
            path,
            mid: ScrollList::new(pending),
            left: ScrollList::default(),
            right: ScrollList::default(),
            history: History::default(),
            cursor: Cursor::new(Focus::Mid),
        }
    }

    fn get_list_mut(&mut self, list_type: Focus) -> &mut ScrollList {
        match list_type {
            Focus::Left => &mut self.left,
            Focus::Mid => &mut self.mid,
            Focus::Right => &mut self.right,
        }
    }

    fn get_list(&self, list_type: Focus) -> &ScrollList {
        match list_type {
            Focus::Left => &self.left,
            Focus::Mid => &self.mid,
            Focus::Right => &self.right,
        }
    }

    fn move_item(&mut self, from_focus: Focus, to_focus: Focus) -> Option<()> {
        let from_list = self.get_list_mut(from_focus);
        let from_index = from_list.state.selected()?;
        if from_list.items.is_empty() {
            return None;
        }

        let item = from_list.items.remove(from_index)?;
        let item_id = item.id;

        // 移除后修正原列表的选中项（防止越界）
        if from_index >= from_list.items.len() {
            from_list
                .state
                .select(Some(from_list.items.len().saturating_sub(1)));
        } else if from_list.items.is_empty() {
            from_list.state.select(None);
        }

        let cmd = SelectOperation::Move {
            item_id,
            from_list: from_focus,
            from_index,
            to_list: to_focus,
        };

        let updated_item = item;
        let to_list = self.get_list_mut(to_focus);
        to_list.items.push_front(updated_item);
        to_list.state.select(Some(0)); // 移动过去后选中新项

        self.history.log(cmd);

        Some(())
    }

    fn undo(&mut self) -> Option<()> {
        if let Some(cmd) = self.history.last() {
            match cmd.clone() {
                // Clone the command to own it
                SelectOperation::Move {
                    item_id,
                    from_list,
                    from_index,
                    to_list,
                    ..
                } => {
                    // 1. 从“去向列表”中移除该项
                    let target_list = &mut self.get_list_mut(to_list).items;
                    let pos = target_list.iter().position(|i| i.id == item_id)?;
                    let item = target_list.remove(pos)?;

                    // 2. 恢复其原始路径，因为在执行 Move 命令时，item 的 path 已经被更新为 new_path

                    // 3. 放回“来源列表”的原始位置
                    let source_list = &mut self.get_list_mut(from_list).items;
                    if from_index >= source_list.len() {
                        source_list.push_back(item);
                    } else {
                        source_list.insert(from_index, item);
                    }
                }
            }
        }

        self.history.undo();
        Some(())
    }

    fn render_lines(&self, list_type: Focus) -> Vec<Line<'static>> {
        let list = self.get_list(list_type);
        list.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.as_line();
                if self.cursor.focus == list_type && list.state.selected() == Some(i) {
                    line = line.reversed();
                }
                line
            })
            .collect::<Vec<Line>>()
    }
    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Res<Cmd> {
        match key_event.code {
            KeyCode::Left => {
                self.move_item(self.cursor.focus, self.cursor.focus.left());
            }
            KeyCode::Right => {
                self.move_item(self.cursor.focus, self.cursor.focus.right());
            }
            KeyCode::Up => {
                self.get_list_mut(self.cursor.focus).up();
            }
            KeyCode::Down => {
                self.get_list_mut(self.cursor.focus).down();
            }
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Cmd::Exit),
            KeyCode::Char('z') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                _ = self.undo();
            }
            KeyCode::Enter => return Ok(Cmd::IntoProcess(self.clone())),
            KeyCode::Tab => {}
            _ => {}
        }
        Ok(Cmd::None)
    }
}

impl crate::core::model::Model for SelectModel {
    type Cmd = crate::core::cmd::Cmd;
    type Msg = crate::core::msg::Msg;
    type Context = crate::core::context::Context;
    fn draw(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
    ) -> color_eyre::Result<()> {
        let buf = frame.buffer_mut();
        let instructions = Line::from(vec![
            " Move Left ".into(),
            "<LEFT>".blue().bold(),
            " Move Right ".into(),
            "<RIGHT>".blue().bold(),
            " Quit ".into(),
            "ctrl + <Z>".into(),
            "undo".into(),
            "<Q> ".blue().bold(),
        ]);

        let title = Line::from(self.path.to_string_lossy().to_string());

        let block = Block::bordered()
            .title(title.centered().bold().blue())
            .title_bottom(instructions.centered())
            .border_set(ratatui::symbols::border::THICK);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let columns = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ]);
        let [left_area, mid_area, right_area] = columns.areas(inner_area);

        // let left_items = self.render_list(Focus::Left);
        // let left_style = if self.cursor.focus == Focus::Left {
        //     ratatui::style::Style::default()
        //         .fg(ratatui::style::Color::Yellow)
        //         .bold()
        // } else {
        //     ratatui::style::Style::default()
        // };
        // Paragraph::new(left_items)
        //     .block(Block::bordered().title("Left").border_style(left_style))
        //     .render(left_area, buf);

        let left_list = List::new(self.render_lines(Focus::Left));
        let left_style = if self.cursor.focus == Focus::Left {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        let left_block = Block::bordered().title("Left").border_style(left_style);
        StatefulWidget::render(
            left_list.block(left_block),
            left_area,
            buf,
            &mut self.left.state,
        );

        let mid_items = self.render_lines(Focus::Mid);
        let mid_style = if self.cursor.focus == Focus::Mid {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        let mid_list =
            List::new(mid_items).block(Block::bordered().title("Mid").border_style(mid_style));
        StatefulWidget::render(mid_list, mid_area, buf, &mut self.mid.state);

        let right_items = self.render_lines(Focus::Right);
        let right_style = if self.cursor.focus == Focus::Right {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        let right_list = List::new(right_items)
            .block(Block::bordered().title("Right").border_style(right_style));
        StatefulWidget::render(right_list, right_area, buf, &mut self.right.state);
        Ok(())
    }
    fn update(&mut self, msg: &Self::Msg, _ctx: &Self::Context) -> Self::Cmd {
        match msg {
            Msg::Exit => Cmd::Exit,
            Msg::Key(ket_event) => self
                .handle_key_event(ket_event)
                .unwrap_or_else(|e| Cmd::Error(e.to_string())),
            _ => Cmd::None,
        }
    }
}
