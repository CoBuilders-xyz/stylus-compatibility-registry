mod commands;

use clap::Parser;
use commands::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Check(args) => commands::check::run(args),
        Commands::CheckDeps(args) => commands::check_deps::run(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
