use color_eyre::eyre::Result;

use crate::{cli::Cli, core::runner::Runner};

use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug)]
pub struct App {
    term: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
    runner: Runner,
}

impl App {
    pub fn new(args: Cli) -> Self {
        Self {
            term: ratatui::init(),
            runner: Runner::new().with_dry_run(args.dry_run),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        color_eyre::install()?;
        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::SetCursorStyle::BlinkingBlock // 或者 BlinkingBar, BlinkingUnderline
        )?;
        // 必须持有 guard，否则日志系统会在 init 结束后立即关闭
        let _guard = Self::log_init()?;

        tracing::info!("[App] app started");
        self.runner.run(&mut self.term).await?;
        tracing::info!("[App] app ended");
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
            // .with_target(true)
            // .with_ansi(false)
            .init();
        Ok(guard)
    }

    fn error_init() -> Result<()> {
        color_eyre::install()?;
        Ok(())
    }
}
