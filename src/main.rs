mod core;

use color_eyre::{Result, eyre::Ok};

#[tokio::main]
async fn main() -> Result<()> {
    // // 1. 创建一个非阻塞的文件写入器 (写入到 logs/app.log)
    // let file_appender = tracing_appender::rolling::hourly("logs", "app.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // // 2. 初始化订阅者，设置格式包含 线程ID、时间、日志级别
    // tracing_subscriber::fmt()
    //     .with_writer(non_blocking)
    //     .with_thread_ids(true) // 关键：看到是哪个线程在说话
    //     .with_thread_names(true)
    //     .with_target(true)
    //     // .with_ansi(false)
    //     .init();

    console_subscriber::init();

    // let mut sys = sysinfo::System::new_all();

    tracing::info!("[main] program start...");
    color_eyre::install()?;

    let mut term: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>> =
        ratatui::init();

    let mut app = core::runner::Runner::new();
    app.run(&mut term).await?;

    tracing::info!("[main] program ended");
    ratatui::restore();
    Ok(())
}
