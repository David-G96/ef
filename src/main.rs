mod app;
mod apps;
mod commands;
mod core;
mod tui;
mod ui;

use color_eyre::Result;

use crate::apps::App;

// 架构简述：
// core是所有操作的核心，尤其是文件操作
// app是链接ui和core的胶水层，会同时包含当前数据和状态，例如history，暂存区，left right pending和对应的liststate等
// ui只负责渲染

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut term = ratatui::init();

    // let mut app = App::new()?;
    // let res = app.run(&mut term);
    let mut app = crate::app::App::new();
    let res = app.run(&mut term);

    ratatui::restore();
    res
}
