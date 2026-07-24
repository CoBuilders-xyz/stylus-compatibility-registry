pub mod check;
pub mod check_deps;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "stylus-registry",
    about = "Check Rust crate compatibility with Arbitrum Stylus",
    version,
    author = "CoBuilders"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check a single crate by name against Stylus constraints
    Check(check::CheckArgs),

    /// Analyze all dependencies in a Cargo.toml for Stylus compatibility
    CheckDeps(check_deps::CheckDepsArgs),
}
