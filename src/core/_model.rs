use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use crate::{
    core::{
        cmd::Cmd,
        component::{AppMessage, Command, Cursor, FileItem, Focus, History, ScrollList},
        msg::Msg,
    },
    commands::ListType,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::line,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

pub trait Model {
    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer);
    fn update(&mut self, msg: Msg) -> Cmd;
}


#[derive(Debug, Default)]
pub enum AppMode {
    #[default]
    /// 处理中，三列
    InProcess,
    /// aka, normal
    ReadOnly,
    Eidting,
    Popup,
    Selecting,
}

/// 分三列的工作区！（左右中）
#[derive(Debug, Default)]
pub struct SelectModel {
    pub path: PathBuf,
    pub mid: ScrollList,
    pub left: ScrollList,
    pub right: ScrollList,
    pub cursor: Cursor,
    history: History,
}

impl SelectModel {
    pub fn new(path: PathBuf, pending: VecDeque<FileItem>) -> Self {
        Self {
            path,
            mid: ScrollList::new(pending),
            left: ScrollList::default(),
            right: ScrollList::default(),
            history: History::default(),
            cursor: Cursor::new(Focus::Mid),
        }
    }

    fn execute(&mut self, cmd: Command) {
        self.history.push(cmd);
    }

    pub fn get_list_mut(&mut self, list_type: Focus) -> &mut VecDeque<FileItem> {
        match list_type {
            Focus::Left => &mut self.left.items,
            Focus::Mid => &mut self.mid.items,
            Focus::Right => &mut self.right.items,
        }
    }

    pub fn get_list(&self, list_type: Focus) -> &VecDeque<FileItem> {
        match list_type {
            Focus::Left => &self.left.items,
            Focus::Mid => &self.mid.items,
            Focus::Right => &self.right.items,
        }
    }

    #[allow(unused)]
    pub fn calculate_new_path(&self, old: &Path, to: Focus) -> PathBuf {
        // TODO:
        old.to_path_buf()
    }

    pub fn move_item(&mut self, from: Cursor, to: Cursor) -> Option<()> {
        let from_list = self.get_list_mut(from.focus);
        if from_list.is_empty() {
            return None;
        }
        let item = from_list.remove(from.index)?;
        let item_id = item.id;

        let new_path = self.calculate_new_path(&item.path, to.focus);

        let cmd = Command::Move {
            item_id,
            from_list: from.focus,
            from_index: from.index,
            to_list: to.focus,
            old_path: item.path.clone(),
            new_path: new_path.clone(),
        };

        let mut updated_item = item;
        updated_item.path = new_path;
        self.get_list_mut(to.focus).push_front(updated_item);

        self.history.push(cmd);

        Some(())
    }

    fn undo(&mut self) -> Option<()> {
        match self.history.last() {
            Some(cmd) => match cmd.clone() {
                // Clone the command to own it
                Command::Move {
                    item_id,
                    from_list,
                    from_index,
                    to_list,
                    old_path,
                    ..
                } => {
                    // 1. 从“去向列表”中移除该项
                    let target_list = self.get_list_mut(to_list);
                    let pos = target_list.iter().position(|i| i.id == item_id)?;
                    let mut item = target_list.remove(pos)?;

                    // 2. 恢复其原始路径，因为在执行 Move 命令时，item 的 path 已经被更新为 new_path
                    item.path = old_path;

                    // 3. 放回“来源列表”的原始位置
                    let source_list = self.get_list_mut(from_list);
                    if from_index >= source_list.len() {
                        source_list.push_back(item);
                    } else {
                        source_list.insert(from_index, item);
                    }
                }
                Command::Delete {
                    item_id: _,
                    from_list,
                    from_index,
                    original_item,
                } => {
                    let source_list = self.get_list_mut(from_list);
                    if from_index >= source_list.len() {
                        source_list.push_back(original_item);
                    } else {
                        source_list.insert(from_index, original_item);
                    }
                }
                _ => {
                    todo!()
                }
            },
            None => {}
        }

        self.history.undo();
        Some(())
    }

    fn render_list_with_cursor<'a>(&'a self, list_type: Focus, state: Cursor) -> Vec<Line<'a>> {
        self.get_list(list_type)
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.colorize();
                if state.focus == list_type && state.index == i {
                    line = line.reversed();
                }
                line
            })
            .collect::<Vec<Line>>()
    }
    fn render_list<'a>(&'a self, list_type: Focus) -> Vec<Line<'a>> {
        self.get_list(list_type)
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.colorize();
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

        let left_items = self.render_list_with_cursor(Focus::Left, *state);
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

        let mid_items = self.render_list_with_cursor(Focus::Mid, *state);
        Paragraph::new(mid_items)
            .block(Block::bordered().title("Mid"))
            .render(mid_area, buf);

        let right_items = self.render_list_with_cursor(Focus::Right, *state);
        Paragraph::new(right_items)
            .block(Block::bordered().title("Right"))
            .render(right_area, buf);
    }
}

impl Model for SelectModel {
    fn update(&mut self, msg: Msg) -> Cmd {
        Cmd::None
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
        let [left_area, mid_area, right_area] = columns.areas(area);

        let left_items = self.render_list(Focus::Left);
        Paragraph::new(left_items)
            .block(Block::bordered().title("Left"))
            .render(left_area, buf);

        let _left_block = Block::new()
            .borders(Borders::ALL)
            .title_top("Mid List")
            .border_style(match self.cursor.focus {
                Focus::Left => ratatui::style::Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .bold(),
                _ => ratatui::style::Style::default(),
            })
            .render(left_area, buf);

        let mid_items = self.render_list_with_cursor(Focus::Mid, self.cursor);
        Paragraph::new(mid_items)
            .block(Block::bordered().title("Mid"))
            .render(mid_area, buf);

        let right_items = self.render_list_with_cursor(Focus::Right, self.cursor);
        Paragraph::new(right_items)
            .block(Block::bordered().title("Right"))
            .render(right_area, buf);
    }
}
