use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// The easy way to do
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Optional path to open
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
    // #[command(subcommand)]
    // command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // /// does testing things
    // Test {
    //     /// lists test values
    //     #[arg(short, long) ]
    //     list: bool,
    // },
}

// fn main() {
//     let cli = Cli::parse();

//     // You can check the value provided by positional arguments, or option arguments
//     if let Some(name) = cli.name.as_deref() {
//         println!("Value for name: {name}");
//     }

//     if let Some(config_path) = cli.config.as_deref() {
//         println!("Value for config: {}", config_path.display());
//     }

//     // You can see how many times a particular flag or argument occurred
//     // Note, only flags can have multiple occurrences
//     match cli.debug {
//         0 => println!("Debug mode is off"),
//         1 => println!("Debug mode is kind of on"),
//         2 => println!("Debug mode is on"),
//         _ => println!("Don't be crazy"),
//     }

//     // You can check for the existence of subcommands, and if found use their
//     // matches just as you would the top level cmd
//     match &cli.command {
//         Some(Commands::Test { list }) => {
//             if *list {
//                 println!("Printing testing lists...");
//             } else {
//                 println!("Not printing testing lists...");
//             }
//         }
//         None => {}
//     }

//     // Continued program logic goes here...
// }
