use std::path::PathBuf;

use crate::core::model::{component::ScrollList, selector::SelectModel};

// #[derive(Debug, PartialEq, Eq)]
// pub struct CmdEnvelope<T> {
//     epoch: u32,
//     payload: T,
// }

#[derive(Debug, Default)]
pub enum Cmd {
    #[default]
    None,
    SuggestRerender,
    SuggestNoRerender,
    Error(String),
    // QueryFileType(PathBuf),
    Batch(Vec<Self>),
    IntoProcess(SelectModel),
    IntoSelect(
        PathBuf,
        Option<ScrollList>,
        Option<ScrollList>,
        Option<ScrollList>,
    ),
    Organize(Vec<PathBuf>, PathBuf),
    Delete(Vec<PathBuf>),
    Exit,
}