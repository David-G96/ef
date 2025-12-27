use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};

use crate::core::model::{component::FileItem, selector::SelectModel};

// #[derive(Debug, PartialEq, Eq)]
// pub struct CmdEnvelope<T> {
//     epoch: u32,
//     payload: T,
// }

#[derive(Debug)]
pub enum Cmd {
    None,
    Error(String),
    // QueryFileType(PathBuf),
    Batch(Vec<Self>),
    IntoProcess(SelectModel),
    Organize(Vec<PathBuf>, PathBuf),
    Delete(Vec<PathBuf>),
    Exit,
}

// pub struct OrganizeCmd<'a, P: AsRef<Path>> {
//     items: &'a [P],
//     target_dir_path: &'a Path,
// }

// #[derive(Debug, PartialEq, Eq)]
// pub struct GuardedCmd {
//     pub cmd: Cmd,
//     pub epoch: u32,
// }
