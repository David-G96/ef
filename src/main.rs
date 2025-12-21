mod app;
// mod apps;
// mod commands;
mod core;
mod tui;
mod ui;

use color_eyre::Result;
use log::{error, info};

// 架构简述：
// core是所有操作的核心，尤其是文件操作
// app是链接ui和core的胶水层，会同时包含当前数据和状态，例如history，暂存区，left right pending和对应的liststate等
// ui只负责渲染

fn main() -> Result<()> {
    color_eyre::install()?;
    // env_logger::init();
    // info!("starting up");
    // error!("sample error");

    let mut term: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> =
        ratatui::init();

    let mut app = crate::app::App::new()?;
    let res = app.run_with_term(&mut term);

    ratatui::restore();
    // info!("ending");
    res
}
