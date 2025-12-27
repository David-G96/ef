//! Reusable common components for models
//!
use std::collections::VecDeque;

use ratatui::{
    style::{self, Style, Stylize as _},
    text::Line,
    widgets::{Block, ListState, Paragraph, StatefulWidget, Widget},
};

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub id: u64,
    pub path: PathBuf,
    /// dir后面加斜杠以示区分，并使用蓝色
    /// e.g. `lib/
    pub display_name: String,
    pub is_dir: bool,
}

impl FileItem {
    pub fn as_line(&self) -> Line<'_> {
        if self.is_dir {
            Line::from(self.display_name.clone().blue())
        } else {
            Line::from(self.display_name.clone())
        }
    }
}

impl Widget for &FileItem {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.is_dir {
            Line::from(self.display_name.clone().blue())
        } else {
            Line::from(self.display_name.clone())
        }
        .render(area, buf);
    }
}

impl std::fmt::Display for FileItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "File<{}>: path={}, display_name={}, is_dir={}",
            self.id,
            self.path.to_string_lossy(),
            self.display_name,
            self.is_dir
        )
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Focus {
    /// The middle list, used as pending/register
    #[default]
    Mid,
    Left,
    Right,
}

impl Focus {
    pub fn left(self) -> Self {
        match self {
            Self::Mid => Self::Left,
            Self::Left => Self::Right,
            Self::Right => Self::Mid,
        }
    }
    pub fn right(self) -> Self {
        match self {
            Self::Mid => Self::Right,
            Self::Left => Self::Mid,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct History<T> {
    history: Vec<T>,
    /// points to the next command
    top: usize,
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            top: 0,
        }
    }
}

impl<T> History<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn log(&mut self, cmd: T) {
        if self.history.len() <= self.top {
            self.history.push(cmd);
        } else {
            self.history[self.top] = cmd;
        }
        self.top = self.top.saturating_add(1);
    }

    pub fn undo(&mut self) {
        self.top = self.top.saturating_sub(1);
    }

    pub fn redo(&mut self) {
        if self.top < self.history.len() - 1 {
            self.top = self.top.saturating_add(1);
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.top == 0 {
            None
        } else {
            Some(&self.history[self.top - 1])
        }
    }

    pub fn count(&self) -> usize {
        self.history.len()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ScrollList {
    pub items: VecDeque<FileItem>,
    pub state: ListState,
}

impl ScrollList {
    pub fn new(items: VecDeque<FileItem>) -> Self {
        Self {
            items,
            state: ListState::default(),
        }
    }

    pub fn render<'a>(
        &'a self,
        is_focus: bool,
        index: Option<usize>,
        tittle: &'a str,
    ) -> Paragraph<'a> {
        let lines = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let mut line = item.as_line();
                if is_focus && index.map(|_i| _i == i).unwrap_or(false) {
                    line = line.reversed();
                }
                line
            })
            .collect::<Vec<Line>>();
        let style = if is_focus {
            Style::default().fg(style::Color::Yellow).bold()
        } else {
            style::Style::default().dim()
        };
        Paragraph::new(lines).block(Block::bordered().title(tittle).border_style(style))
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    pub focus: Focus,
    pub index: usize,
}

impl Cursor {
    pub fn new(focus: Focus) -> Self {
        Self { focus, index: 0 }
    }

    pub fn shift_down(self) -> Self {
        Self {
            index: self.index.saturating_add(1),
            ..self
        }
    }

    pub fn shift_up(self) -> Self {
        Self {
            index: self.index.saturating_sub(1),
            ..self
        }
    }

    pub fn move_left(self) -> Self {
        Self {
            focus: self.focus.left(),
            ..self
        }
    }

    pub fn move_right(self) -> Self {
        Self {
            focus: self.focus.right(),
            ..self
        }
    }
}

// /// 分三列的工作区！（左右中）
// #[derive(Debug, Default)]
// pub struct WorkSpace {
//     pub path: PathBuf,
//     pub mid: ScrollList,
//     pub left: ScrollList,
//     pub right: ScrollList,
//     history: History,
// }

// impl WorkSpace {
//     pub fn new(path: PathBuf, pending: VecDeque<FileItem>) -> Self {
//         Self {
//             path,
//             mid: ScrollList::new(pending),
//             left: ScrollList::default(),
//             right: ScrollList::default(),
//             history: History::default(),
//         }
//     }

//     pub fn execute(&mut self, cmd: Command) {
//         self.history.push(cmd);
//     }

//     pub fn get_list_mut(&mut self, list_type: Focus) -> &mut VecDeque<FileItem> {
//         match list_type {
//             Focus::Left => &mut self.left.items,
//             Focus::Mid => &mut self.mid.items,
//             Focus::Right => &mut self.right.items,
//         }
//     }

//     pub fn get_list(&self, list_type: Focus) -> &VecDeque<FileItem> {
//         match list_type {
//             Focus::Left => &self.left.items,
//             Focus::Mid => &self.mid.items,
//             Focus::Right => &self.right.items,
//         }
//     }

//     #[allow(unused)]
//     pub fn calculate_new_path(&self, old: &Path, to: Focus) -> PathBuf {
//         // TODO:
//         old.to_path_buf()
//     }

//     pub fn move_item(&mut self, from: Cursor, to: Cursor) -> Option<()> {
//         let from_list = self.get_list_mut(from.focus);
//         if from_list.is_empty() {
//             return None;
//         }
//         let item = from_list.remove(from.index)?;
//         let item_id = item.id;

//         let new_path = self.calculate_new_path(&item.path, to.focus);

//         let cmd = Command::Move {
//             item_id,
//             from_list: from.focus,
//             from_index: from.index,
//             to_list: to.focus,
//             old_path: item.path.clone(),
//             new_path: new_path.clone(),
//         };

//         let mut updated_item = item;
//         updated_item.path = new_path;
//         self.get_list_mut(to.focus).push_front(updated_item);

//         self.history.push(cmd);

//         Some(())
//     }

//     pub fn undo(&mut self) -> Option<()> {
//         match self.history.last() {
//             Some(cmd) => match cmd.clone() {
//                 // Clone the command to own it
//                 Command::Move {
//                     item_id,
//                     from_list,
//                     from_index,
//                     to_list,
//                     old_path,
//                     ..
//                 } => {
//                     // 1. 从“去向列表”中移除该项
//                     let target_list = self.get_list_mut(to_list);
//                     let pos = target_list.iter().position(|i| i.id == item_id)?;
//                     let mut item = target_list.remove(pos)?;

//                     // 2. 恢复其原始路径，因为在执行 Move 命令时，item 的 path 已经被更新为 new_path
//                     item.path = old_path;

//                     // 3. 放回“来源列表”的原始位置
//                     let source_list = self.get_list_mut(from_list);
//                     if from_index >= source_list.len() {
//                         source_list.push_back(item);
//                     } else {
//                         source_list.insert(from_index, item);
//                     }
//                 }
//                 Command::Delete {
//                     item_id: _,
//                     from_list,
//                     from_index,
//                     original_item,
//                 } => {
//                     let source_list = self.get_list_mut(from_list);
//                     if from_index >= source_list.len() {
//                         source_list.push_back(original_item);
//                     } else {
//                         source_list.insert(from_index, original_item);
//                     }
//                 }
//                 _ => {
//                     todo!()
//                 }
//             },
//             None => {}
//         }

//         self.history.undo();
//         Some(())
//     }

//     pub fn render_list<'a>(&'a self, list_type: Focus, state: Cursor) -> Vec<Line<'a>> {
//         self.get_list(list_type)
//             .iter()
//             .enumerate()
//             .map(|(i, item)| {
//                 let mut line = item.colorize();
//                 if state.focus == list_type && state.index == i {
//                     line = line.reversed();
//                 }
//                 line
//             })
//             .collect::<Vec<Line>>()
//     }

//     pub fn render_as_stateful(
//         &self,
//         area: ratatui::prelude::Rect,
//         buf: &mut ratatui::prelude::Buffer,
//         state: &Cursor,
//     ) {
//         let columns = Layout::horizontal([
//             Constraint::Percentage(33),
//             Constraint::Percentage(34),
//             Constraint::Percentage(33),
//         ]);
//         let [left_area, mid_area, right_area] = columns.areas(area);

//         let left_items = self.render_list(Focus::Left, *state);
//         Paragraph::new(left_items)
//             .block(Block::bordered().title("Left"))
//             .render(left_area, buf);

//         let _left_block = Block::new()
//             .borders(Borders::ALL)
//             .title_top("Mid List")
//             .border_style(match state.focus {
//                 Focus::Left => ratatui::style::Style::default()
//                     .fg(ratatui::style::Color::Yellow)
//                     .bold(),
//                 _ => ratatui::style::Style::default(),
//             })
//             .render(left_area, buf);

//         let mid_items = self.render_list(Focus::Mid, *state);
//         Paragraph::new(mid_items)
//             .block(Block::bordered().title("Mid"))
//             .render(mid_area, buf);

//         let right_items = self.render_list(Focus::Right, *state);
//         Paragraph::new(right_items)
//             .block(Block::bordered().title("Right"))
//             .render(right_area, buf);
//     }
// }
