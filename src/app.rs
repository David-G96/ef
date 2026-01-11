use color_eyre::eyre::Result as Res;

use crate::{
    cli::Cli,
    core::{config::Config, runner::Runner},
};

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
}
