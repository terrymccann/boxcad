//! Command-line access to the core, so geometry can be tested and swept
//! without the app. Subcommands arrive with the features they exercise.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "boxcad-cli", version, about = "Box CAD from the command line")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the core library's version.
    Version,
}

fn main() {
    match Cli::parse().command {
        Command::Version => println!("boxcad-core {}", boxcad_core::version()),
    }
}
