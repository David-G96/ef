use std::path::PathBuf;

use crossterm::event::{KeyEvent, MouseEvent};

#[derive(Debug, PartialEq, Eq)]
pub enum Msg {
    // UI/User input
    // UiInput(UiEvent),

    // Cross term Events
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    // OtherCrosstermEvent(crossterm::event::Event),
    Init,
    Exit,
    Error(String),
    Tick,
    /// 请注意，render和tick是不同的
    Render,
    // 更推荐使用Key(KeyEvent)，因为其包含了modifier ctrl的信息
    // Input(KeyCode),
    /// Now, call `read_dir()` !
    FileChanged,
    // Exit,

    // File system watcher -> high-level event
    FileEvent(FileEvent),

    // Results from async tasks (Cmds)
    FileLoaded {
        path: PathBuf,
        data: Vec<u8>,
    },
    // FileSaved {
    //     path: PathBuf,
    //     result: Result<(), IoError>,
    // },
}

#[derive(Debug, PartialEq, Eq)]
pub enum FileEvent {
    FileChanged,
}
