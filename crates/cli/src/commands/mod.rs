pub mod check;
pub mod check_deps;

use clap::{Parser, Subcommand};

/// Indents the continuation lines of a check message. Compiler output arrives with
/// embedded newlines and would otherwise hug the left margin instead of nesting
/// under the check it belongs to.
pub fn indent_message(message: &str) -> String {
    message.replace('\n', "\n      ")
}

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

#[cfg(test)]
mod tests {
    use super::indent_message;

    #[test]
    fn indents_continuation_lines() {
        let indented = indent_message("failed:\nerror: first\nerror: second");
        assert_eq!(indented, "failed:\n      error: first\n      error: second");
    }

    #[test]
    fn leaves_single_line_messages_untouched() {
        assert_eq!(indent_message("all good"), "all good");
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check a single crate by name against Stylus constraints
    Check(check::CheckArgs),

    /// Analyze all dependencies in a Cargo.toml for Stylus compatibility
    CheckDeps(check_deps::CheckDepsArgs),
}
