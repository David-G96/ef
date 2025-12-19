//! 这是操作底层的核心，用来记录、撤销、执行操作

use color_eyre::eyre::{Ok, Result};
use core::panic;
use ratatui::style::Stylize;
use std::collections::{HashMap, VecDeque};
use std::fs::DirEntry;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Copy)]
#[non_exhaustive]
pub enum ListType {
    Left,
    Pending,
    Right,
    // 如果一个元素在结束时依旧是pending，那么就作为未修改
    // Unmodified,
}

impl Default for ListType {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub id: Uuid,
    pub path: PathBuf,
    /// dir后面加斜杠以示区分，如果必要，名称还要使用蓝色
    /// e.g. `lib/
    pub display_name: String,
    pub is_dir: bool,
}

impl FileItem {
    pub fn new(id: Uuid, dir: DirEntry) -> Result<Self> {
        let mut display_name = dir.file_name().to_string_lossy().to_string();
        if dir.file_type()?.is_dir() {
            display_name.push('/');
        }
        Ok(Self {
            id,
            path: dir.path(),
            display_name,
            is_dir: dir.file_type()?.is_dir(),
        })
    }
    pub fn colorize(&self) -> ratatui::text::Line {
        if self.is_dir {
            ratatui::text::Line::from(vec![
                self.display_name[..self.display_name.len() - 1].blue(),
                "/".into(),
            ])
        } else {
            ratatui::text::Line::from(self.display_name.clone())
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

#[derive(Debug)]
pub struct CommandManager {
    pub left: VecDeque<FileItem>,
    pub pending: VecDeque<FileItem>,
    pub right: VecDeque<FileItem>,

    /// 撤销栈（暂存的指令）
    history: Vec<Command>,
    /// 指向最新的指令的下一位，用来实现redo重做
    /// 也就是说，当其为0时代表着当前历史没有指令！
    top: usize,
}

impl CommandManager {
    pub fn new(initial_pending: impl Into<VecDeque<FileItem>>) -> Self {
        Self {
            left: VecDeque::new(),
            pending: VecDeque::from(initial_pending.into()),
            right: VecDeque::new(),
            history: Vec::new(),
            top: 0,
        }
    }

    // 辅助函数：通过 ListType 获取对应的列表引用
    fn get_list_mut(&mut self, list_type: ListType) -> &mut VecDeque<FileItem> {
        match list_type {
            ListType::Left => &mut self.left,
            ListType::Pending => &mut self.pending,
            ListType::Right => &mut self.right,
        }
    }

    fn get_list(&self, list_type: ListType) -> &VecDeque<FileItem> {
        match list_type {
            ListType::Left => &self.left,
            ListType::Pending => &self.pending,
            ListType::Right => &self.right,
        }
    }

    // --- 正向操作 ---

    /// 将待处理列表的第一个元素移向左侧/右侧
    pub fn move_item(&mut self, from: ListType, to: ListType) -> Option<()> {
        let from_list = self.get_list_mut(from);
        if from_list.is_empty() {
            return None;
        }

        // 1. 从源列表取出（这里演示从开头取，你可以根据 index 取）
        let item = from_list.pop_front()?;
        let item_id = item.id;

        // 计算新路径（示意逻辑）
        let new_path = self.calculate_new_path(&item.path, to);

        // 2. 记录命令
        let cmd = Command::Move {
            item_id,
            from_list: from,
            from_index: 0, // 记录它是从哪个位置被取走的
            to_list: to,
            old_path: item.path.clone(),
            new_path: new_path.clone(),
        };

        // 3. 执行内存移动（插入目标列表开头）
        let mut updated_item = item;
        updated_item.path = new_path;
        self.get_list_mut(to).push_front(updated_item);

        // 4. 存入历史记录
        self.log_history(cmd);

        Some(())
    }

    // --- 撤销操作 ---

    pub fn undo(&mut self) -> Option<()> {
        // 在这里，如果没有历史的话刚好会返回None，导致没有操作
        if self.top == 0 {
            return None;
        }
        let last_cmd = self.history.get(self.top - 1)?.clone();
        // 成功获取上一条指令，开始undo
        self.top -= 1;

        match last_cmd {
            Command::Move {
                item_id,
                from_list,
                from_index,
                to_list,
                old_path,
                ..
            } => {
                // 1. 从“去向列表”中移除该项（它刚才被放在了开头）
                let target_list = self.get_list_mut(to_list);
                let pos = target_list.iter().position(|i| i.id == item_id)?;
                let mut item = target_list.remove(pos)?;

                // 2. 恢复其原始路径
                item.path = old_path;

                // 3. 放回“来源列表”的原始位置
                let source_list = self.get_list_mut(from_list);
                if from_index >= source_list.len() {
                    source_list.push_back(item);
                } else {
                    source_list.insert(from_index, item);
                }
            }
            // undo删除操作似乎不需要任何操作
            Command::Delete {
                item_id,
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
            // 其他命令的 undo 逻辑...
            _ => todo!(),
        }
        Some(())
    }

    fn log_history(&mut self, cmd: Command) {
        if self.history.len() <= self.top {
            self.history.push(cmd);
        } else {
            self.history[self.top] = cmd;
        }
        self.top += 1;
    }

    // 最终提交时，获取所有待执行的磁盘指令
    pub fn get_pending_commands(&self) -> &[Command] {
        &self.history[0..self.top]
    }

    fn calculate_new_path(&self, old: &PathBuf, to: ListType) -> PathBuf {
        // 实际逻辑：根据目标列表代表的文件夹生成新路径
        old.clone()
    }
}
