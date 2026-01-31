mod app;
mod cli;
mod core;

use clap::Parser as _;
use color_eyre::eyre::Result as Res;

use crate::{app::App, cli::Cli};

#[tokio::main]
async fn main() -> Res<()> {
    let _guard = log_init()?;
    tracing::info!("[main] program start...");

    let args = Cli::parse();
    let mut app = App::new(args)?;
    app.run().await?;

    tracing::info!("[main] program ended");
    Ok(())
}

fn log_init() -> color_eyre::Result<tracing_appender::non_blocking::WorkerGuard> {
    // 1. 创建一个非阻塞的文件写入器 (写入到 logs/app.log)
    let file_appender = tracing_appender::rolling::hourly(get_log_dir("ef"), "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // 2. 初始化订阅者，设置格式包含 线程ID、时间、日志级别
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_thread_ids(true) // 关键：看到是哪个线程在说话
        .with_thread_names(true)
        .with_target(false)
        .with_ansi(false)
        .init();
    Ok(guard)
}

use directories::ProjectDirs;
use std::fs;

fn get_log_dir(app_name: &str) -> std::path::PathBuf {
    // Qualifier 通常用你的域名反写，如 "com.github.user"
    if let Some(proj_dirs) = ProjectDirs::from("com", "David-G96", app_name) {
        // 对于 CLI 程序的日志，推荐使用 cache_dir 或 state_dir
        let log_dir = proj_dirs.cache_dir().join("logs");

        // 确保文件夹存在
        let _ = fs::create_dir_all(&log_dir);
        return log_dir;
    }

    // 备选方案：如果获取不到系统路径，退回到当前目录的隐藏文件夹
    std::path::PathBuf::from(".logs")
}

