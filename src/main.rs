mod app;
mod commands;
mod ui;

use std::env;

use color_eyre::Result;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut term = ratatui::init();
    let mut app = App::new()?;
    let res = app.run(&mut term);
    ratatui::restore();

    res
}
