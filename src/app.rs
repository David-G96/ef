use color_eyre::eyre::Result as Res;

use crate::{
    cli::Cli,
    core::{config::Config, runner::Runner},
};

use tracing_appender::non_blocking::WorkerGuard;

#[derive(Debug)]
pub struct App {
    term: ratatui::Terminal<ratatui::prelude::CrosstermBackend<std::io::Stdout>>,
    runner: Runner,
}

impl App {
    pub fn new(args: Cli) -> Res<Self> {
        let config_status = Config::parse()?;
        match config_status {
            crate::core::config::ConfigStatus::Loaded(_) => tracing::info!("loaded config"),
            _ => tracing::info!("loaded config from default"),
        }
        let config = config_status.config();

        Ok(Self {
            term: ratatui::init(),
            runner: Runner::new(config.clone()).with_dry_run(args.dry_run),
        })
    }

    pub async fn run(&mut self) -> Res<()> {
        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::SetCursorStyle::BlinkingBlock // 或者 BlinkingBar, BlinkingUnderline
        )?;
        color_eyre::install()?;

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
            .with_target(false)
            // .with_ansi(false)
            .init();
        Ok(guard)
    }

    fn error_init() -> Res<()> {
        color_eyre::install()?;
        Ok(())
    }
}
