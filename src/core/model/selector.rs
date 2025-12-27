use std::{collections::VecDeque, env, path::PathBuf};

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
    widgets::{Block, Borders, Paragraph, Widget as _},
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
        // 磁盘相关元数据
        // old_path: PathBuf,
        // new_path: PathBuf,
    },
    // Delete {
    //     item_id: u64,
    //     from_list: Focus,
    //     from_index: usize,
    //     original_item: FileItem, // 撤销删除时需要重新创建该项
    // },
    // 归类通常是移动到某个文件夹，逻辑上类似于 Move
    // Categorize {
    //     item_id: u64,
    //     from_list: Focus,
    //     from_index: usize,
    //     category: String,
    //     target_path: PathBuf,
    // },
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
    pub fn new() -> Res<Self> {
        let current_path = env::current_dir()?;
        let mut res: VecDeque<FileItem> = VecDeque::with_capacity(8);
        let mut id = 0;
        for entry in std::fs::read_dir(&current_path)? {
            let entry = entry?;
            let item = FileItem {
                id: id,
                path: entry.path(),
                display_name: entry.file_name().to_string_lossy().to_string(),
                is_dir: entry.file_type()?.is_dir(),
            };
            res.push_back(item);
            id += 1;
        }
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

    fn log_history(&mut self, cmd: SelectOperation) {
        self.history.log(cmd);
    }

    fn get_list_mut(&mut self, list_type: Focus) -> &mut VecDeque<FileItem> {
        match list_type {
            Focus::Left => &mut self.left.items,
            Focus::Mid => &mut self.mid.items,
            Focus::Right => &mut self.right.items,
        }
    }

    fn get_list(&self, list_type: Focus) -> &VecDeque<FileItem> {
        match list_type {
            Focus::Left => &self.left.items,
            Focus::Mid => &self.mid.items,
            Focus::Right => &self.right.items,
        }
    }

    fn move_item(&mut self, from: Cursor, to: Cursor) -> Option<()> {
        let from_list = self.get_list_mut(from.focus);
        if from_list.is_empty() {
            return None;
        }
        let item = from_list.remove(from.index)?;
        let item_id = item.id;

        let cmd = SelectOperation::Move {
            item_id,
            from_list: from.focus,
            from_index: from.index,
            to_list: to.focus,
        };

        let updated_item = item;
        self.get_list_mut(to.focus).push_front(updated_item);

        self.history.log(cmd);

        Some(())
    }

    fn undo(&mut self) -> Option<()> {
        match self.history.last() {
            Some(cmd) => match cmd.clone() {
                // Clone the command to own it
                SelectOperation::Move {
                    item_id,
                    from_list,
                    from_index,
                    to_list,
                    ..
                } => {
                    // 1. 从“去向列表”中移除该项
                    let target_list = self.get_list_mut(to_list);
                    let pos = target_list.iter().position(|i| i.id == item_id)?;
                    let item = target_list.remove(pos)?;

                    // 2. 恢复其原始路径，因为在执行 Move 命令时，item 的 path 已经被更新为 new_path

                    // 3. 放回“来源列表”的原始位置
                    let source_list = self.get_list_mut(from_list);
                    if from_index >= source_list.len() {
                        source_list.push_back(item);
                    } else {
                        source_list.insert(from_index, item);
                    }
                }
            },
            None => {}
        }

        self.history.undo();
        Some(())
    }

   pub(crate) fn render_list<'a>(&'a self, list_type: Focus) -> Vec<Line<'a>> {
        self.get_list(list_type)
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.as_line();
                if self.cursor.focus == list_type && self.cursor.index == i {
                    line = line.reversed();
                }
                line
            })
            .collect::<Vec<Line>>()
    }

    fn render_as_stateful(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &Cursor,
    ) {
        let columns = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ]);
        let [left_area, mid_area, right_area] = columns.areas(area);

        let left_items = self.render_list(Focus::Left);
        Paragraph::new(left_items)
            .block(Block::bordered().title("Left"))
            .render(left_area, buf);

        let _left_block = Block::new()
            .borders(Borders::ALL)
            .title_top("Mid List")
            .border_style(match state.focus {
                Focus::Left => ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .bold(),
                _ => ratatui::style::Style::default(),
            })
            .render(left_area, buf);

        let mid_items = self.render_list(Focus::Mid);
        Paragraph::new(mid_items)
            .block(Block::bordered().title("Mid"))
            .render(mid_area, buf);

        let right_items = self.render_list(Focus::Right);
        Paragraph::new(right_items)
            .block(Block::bordered().title("Right"))
            .render(right_area, buf);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Res<Cmd> {
        match key_event.code {
            KeyCode::Left => {
                self.move_item(self.cursor, self.cursor.move_left());
            }
            KeyCode::Right => {
                self.move_item(self.cursor, self.cursor.move_right());
            }
            KeyCode::Up => {
                self.cursor = self.cursor.shift_up();
            }
            KeyCode::Down => {
                if self.cursor.index < self.get_list(self.cursor.focus).len() - 1 {
                    self.cursor = self.cursor.shift_down();
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Cmd::Exit),
            KeyCode::Char('z') if key_event.modifiers == KeyModifiers::CONTROL => {
                _ = self.undo();
            }
            KeyCode::Enter => return Ok(Cmd::IntoProcess(self.clone())),
            _ => {}
        }
        Ok(Cmd::None)
    }
}

impl crate::core::traits::Model for SelectModel {
    fn update(&mut self, msg: Msg, _: &crate::core::context::Context) -> Cmd {
        match msg {
            Msg::Exit => Cmd::Exit,
            Msg::Key(ket_event) => {
                tracing::info!("[SelectModel] got key {:?}", ket_event);
                self.handle_key_event(ket_event)
                    .unwrap_or_else(|e| Cmd::Error(e.to_string()))
            }
            _ => Cmd::None,
        }
    }

    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let instructions = Line::from(vec![
            " Move Left ".into(),
            "<LEFT>".blue().bold(),
            " Move Right ".into(),
            "<RIGHT>".blue().bold(),
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

        let left_items = self.render_list(Focus::Left);
        let left_style = if self.cursor.focus == Focus::Left {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        Paragraph::new(left_items)
            .block(Block::bordered().title("Left").border_style(left_style))
            .render(left_area, buf);

        let mid_items = self.render_list(Focus::Mid);
        let mid_style = if self.cursor.focus == Focus::Mid {
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Yellow)
                .bold()
        } else {
            ratatui::style::Style::default()
        };
        Paragraph::new(mid_items)
            .block(Block::bordered().title("Mid").border_style(mid_style))
            .render(mid_area, buf);

        let right_items = self.render_list(Focus::Right);
        let right_style = if self.cursor.focus == Focus::Right {
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
