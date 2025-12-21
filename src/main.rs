mod app;
// mod apps;
// mod commands;
mod core;
mod tui;
mod ui;

use color_eyre::Result;

// 架构简述：
// core是所有操作的核心，尤其是文件操作
// app是链接ui和core的胶水层，会同时包含当前数据和状态，例如history，暂存区，left right pending和对应的liststate等
// ui只负责渲染

fn main() -> Result<()> {
    // 1. 创建一个非阻塞的文件写入器 (写入到 logs/app.log)
    let file_appender = tracing_appender::rolling::hourly("logs", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 2. 初始化订阅者，设置格式包含 线程ID、时间、日志级别
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_thread_ids(true) // 关键：看到是哪个线程在说话
        .with_thread_names(true)
        .with_target(true)
        // .with_ansi(false)
        .init();

    tracing::info!("program start...");
    color_eyre::install()?;

    let mut term: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> =
        ratatui::init();

    let mut app = crate::app::App::new()?;
    let res = app.run_with_term(&mut term);

    ratatui::restore();
    tracing::info!("program ended");
    res
}
