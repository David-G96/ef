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
