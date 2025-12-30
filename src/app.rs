use color_eyre::eyre::Result;

use crate::{cli::Cli, core::runner::Runner};

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
        self.runner.run(&mut self.term).await?;
        Ok(())
    }
}
