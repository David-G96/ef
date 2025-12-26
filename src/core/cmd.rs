use std::path::PathBuf;

// #[derive(Debug, PartialEq, Eq)]
// pub struct CmdEnvelope<T> {
//     epoch: u32,
//     payload: T,
// }

#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    None,
    // no one should call render, which will break the 声明式设计。
    // 只有在runtime收到了应当render的信息时呼叫render
    // Render,
    Error(String),
    QueryFileType(PathBuf),
    Batch(Vec<Self>),
    Exit,
}

// #[derive(Debug, PartialEq, Eq)]
// pub struct GuardedCmd {
//     pub cmd: Cmd,
//     pub epoch: u32,
// }
