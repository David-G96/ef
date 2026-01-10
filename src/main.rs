mod app;
mod cli;
mod core;

use clap::Parser as _;
use color_eyre::eyre::Result as Res;
use tracing_appender::non_blocking::WorkerGuard;

use crate::{app::App, cli::Cli};

#[tokio::main]
async fn main() -> Res<()> {
    let _guard = log_init()?;
    tracing::info!("[main] program start...");

    let args = Cli::try_parse()?;
    let mut app = App::new(args)?;
    app.run().await?;

    tracing::info!("[main] program ended");
    Ok(())
}

fn log_init() -> color_eyre::Result<WorkerGuard> {
    // 1. 创建一个非阻塞的文件写入器 (写入到 logs/app.log)
    let file_appender = tracing_appender::rolling::hourly("logs", "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 2. 初始化订阅者，设置格式包含 线程ID、时间、日志级别
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_thread_ids(true) // 关键：看到是哪个线程在说话
        .with_thread_names(true)
        .with_target(false)
        // .with_ansi(false)
        .init();
    Ok(guard)
}
