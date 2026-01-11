use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None,)]
pub struct Cli {
    /// Optional path to open
    #[arg(short, long, value_name = "PATH")]
    pub path: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Do not actually perform any operations
    #[arg(short = 'n', long = "dry-run")]
    pub dry_run: bool,

    /// Do not print log messages
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Use verbose output
    #[arg(short = 'v', long = "verbose")]
    pub verbose: bool,
}
