use std::path::PathBuf;

use crate::core::model::component::FileItem;
use crossterm::event::{KeyEvent, MouseEvent};
use std::collections::VecDeque;

#[derive(Debug)]
pub enum Msg {
    // Cross term Events
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    Init,
    Exit,
    Error(String),
    Tick,
    FileChanged,

    // File system watcher -> high-level event
    FileEvent(FileEvent),

    // Results from async tasks (Cmds)
    FileLoaded {
        path: PathBuf,
        data: Vec<u8>,
    },
    DirLoaded(PathBuf, VecDeque<FileItem>),
    TaskState(TaskState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing(f32), // 进度百分比
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaskState {
    pub id: u64,
    pub epoch: u32,
    pub status: TaskStatus,
}

impl TaskState {
    pub fn new(id: u64, epoch: u32, status: TaskStatus) -> Self {
        Self { id, epoch, status }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileEvent {
    FileChanged,
}
