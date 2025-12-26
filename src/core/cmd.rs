use std::path::PathBuf;

// #[derive(Debug, PartialEq, Eq)]
// pub struct CmdEnvelope<T> {
//     epoch: u32,
//     payload: T,
// }

#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    None,
    QueryFileType(PathBuf),
    Batch(Vec<Self>),
    Exit,
}

// #[derive(Debug, PartialEq, Eq)]
// pub struct GuardedCmd {
//     pub cmd: Cmd,
//     pub epoch: u32,
// }
