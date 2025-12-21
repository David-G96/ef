//! Reusable components for the App
//!
use std::{collections::VecDeque, path::Path};

use ratatui::widgets::ListState;

use std::path::PathBuf;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileItem {
    pub id: Uuid,
    pub path: PathBuf,
    /// dir后面加斜杠以示区分，如果必要，名称还要使用蓝色
    /// e.g. `lib/
    pub display_name: String,
    pub is_dir: bool,
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

#[derive(Debug, Default)]
pub struct FileManager {
    items: Vec<FileItem>,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ListType {
    /// The middle list, used as pending/register
    #[default]
    Mid,
    Left,
    Right,
}

impl ListType {
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
pub enum Command {
    // 移动操作：记录从哪来、到哪去，以及它在原列表中的原始位置（用于完美还原）
    Move {
        item_id: Uuid,
        from_list: ListType,
        from_index: usize, // 撤销时放回原位所需
        to_list: ListType,
        // 磁盘相关元数据
        old_path: PathBuf,
        new_path: PathBuf,
    },
    Delete {
        item_id: Uuid,
        from_list: ListType,
        from_index: usize,
        original_item: FileItem, // 撤销删除时需要重新创建该项
    },
    // 归类通常是移动到某个文件夹，逻辑上类似于 Move
    Categorize {
        item_id: Uuid,
        from_list: ListType,
        from_index: usize,
        category: String,
        target_path: PathBuf,
    },
}

#[derive(Default, Debug)]
pub struct History {
    history: Vec<Command>,
    /// points to the next command
    top: usize,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, cmd: Command) {
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

    pub fn last(&self) -> Option<&Command> {
        if self.top == 0 {
            return None;
        } else {
            Some(&self.history[self.top - 1])
        }
    }
}

#[derive(Debug, Default)]
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
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    pub focus: ListType,
    pub index: usize,
}

impl Cursor {
    pub fn new(focus: ListType) -> Self {
        Self { focus, index: 0 }
    }

    pub fn scroll_up(self) -> Self {
        Self {
            index: self.index.saturating_add(1),
            ..self
        }
    }

    pub fn scroll_down(self) -> Self {
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

/// 分三列的工作区！（左右中）
#[derive(Debug, Default)]
pub struct WorkSpace {
    // pub lists: Vec<ScrollList>,
    pub mid: ScrollList,
    pub left: ScrollList,
    pub right: ScrollList,
    history: History,
}

impl WorkSpace {
    pub fn new(pending: VecDeque<FileItem>) -> Self {
        Self {
            mid: ScrollList::new(pending),
            left: ScrollList::default(),
            right: ScrollList::default(),
            history: History::default(),
        }
    }
    pub fn execute(&mut self, cmd: Command) {
        self.history.push(cmd);
    }

    pub fn get_list_mut(&mut self, list_type: ListType) -> &mut VecDeque<FileItem> {
        match list_type {
            ListType::Left => &mut self.left.items,
            ListType::Mid => &mut self.mid.items,
            ListType::Right => &mut self.right.items,
        }
    }

    pub fn get_list(&self, list_type: ListType) -> &VecDeque<FileItem> {
        match list_type {
            ListType::Left => &self.left.items,
            ListType::Mid => &self.mid.items,
            ListType::Right => &self.right.items,
        }
    }

    #[allow(unused)]
    pub fn calculate_new_path(&self, old: &Path, to: ListType) -> PathBuf {
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

    pub fn undo(&mut self) -> Option<()> {
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
}
