mod app;
mod cli;
mod core;
mod errors;
mod logging;

use clap::Parser;
use color_eyre::eyre::Result as Res;
use std::io::stdout;

use crossterm::{cursor::SetCursorStyle, execute};

use crate::{app::App, cli::Cli};

#[tokio::main]
async fn main() -> Res<()> {
    // 必须持有 guard，否则日志系统会在 init 结束后立即关闭
    let _guard = logging::init()?;

    tracing::info!("[main] program start...");
    color_eyre::install()?;
    execute!(
        stdout(),
        SetCursorStyle::BlinkingBlock // 或者 BlinkingBar, BlinkingUnderline
    )?;
    // console_subscriber::init();


    let args = Cli::parse();
    let mut app = App::new(args);
    app.run().await?;

    tracing::info!("[main] program ended");
    ratatui::restore();
    Ok(())
}
