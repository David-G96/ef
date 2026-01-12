use std::path::PathBuf;

use crate::core::model::{component::ScrollList, selector::SelectModel};

#[derive(Debug, Default)]
pub enum Cmd {
    #[default]
    None,
    Error(String),
    /// Sequential commands
    Seq(Vec<Self>),
    /// Not sequential commands
    Batch(Vec<Self>),
    IntoProcess(SelectModel),
    IntoSelect(
        PathBuf,
        Option<ScrollList>,
        Option<ScrollList>,
        Option<ScrollList>,
    ),
    // /// Requires a command to be performed as a async task.
    // /// The first argument is a u64 id, which should be internally managed by the model.
    // / The second argument should be the async task to perform.
    // AsyncTask(u64, Box<Self>),
    AsyncOrganize(u64,Vec<PathBuf>, PathBuf),
    AsyncDelete(u64,Vec<PathBuf>),
    AsyncCopy(u64,Vec<PathBuf>, PathBuf),
    AsyncTrash(u64, Vec<PathBuf>),
    AsyncMove(u64, Vec<PathBuf>, PathBuf),
    Organize(Vec<PathBuf>, PathBuf),
    Delete(Vec<PathBuf>),
    Copy(Vec<PathBuf>, PathBuf),
    Trash(Vec<PathBuf>),
    Move(Vec<PathBuf>, PathBuf),
    Exit,
    ToggleShowHidden,
    ToggleRespectGitIgnore,
    LoadDir(PathBuf),
}
