mod app;
mod cli;
mod core;

use clap::Parser as _;
use color_eyre::eyre::Result as Res;

use crate::{app::App, cli::Cli};

#[tokio::main]
async fn main() -> Res<()> {
    tracing::info!("[main] program start...");

    let args = Cli::parse();
    let mut app = App::new(args);
    app.run().await?;

    tracing::info!("[main] program ended");
    Ok(())
}
