use std::path::PathBuf;

use crate::core::model::{component::ScrollList, selector::SelectModel};

#[derive(Debug, Default)]
pub enum Cmd {
    #[default]
    None,
    SuggestRerender,
    SuggestNoRerender,
    Error(String),
    // QueryFileType(PathBuf),
    Seq(Vec<Self>),
    IntoProcess(SelectModel),
    IntoSelect(
        PathBuf,
        Option<ScrollList>,
        Option<ScrollList>,
        Option<ScrollList>,
    ),
    Organize(Vec<PathBuf>, PathBuf),
    Delete(Vec<PathBuf>),
    Copy(Vec<PathBuf>, PathBuf),
    Trash(Vec<PathBuf>),
    Move(Vec<PathBuf>, PathBuf),
    Exit,
}
