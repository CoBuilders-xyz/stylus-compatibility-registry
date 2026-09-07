use clap::Args;
use colored::Colorize;
use stylus_compat_core::checks::run_all_checks;
use stylus_compat_core::score::compute_score;
use stylus_compat_core::types::{CrateInfo, Severity};

#[derive(Args)]
pub struct CheckArgs {
    /// Crate name to check
    pub crate_name: String,

    /// Crate version (optional)
    #[arg(short, long)]
    pub version: Option<String>,

    /// Features to enable, comma separated or repeated
    #[arg(long, value_delimiter = ',')]
    pub features: Vec<String>,

    /// Do not enable the crate's default features
    #[arg(long)]
    pub no_default_features: bool,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: CheckArgs) -> Result<(), Box<dyn std::error::Error>> {
    let crate_info = CrateInfo {
        name: args.crate_name.clone(),
        version: args.version,
        features: args.features,
        default_features: !args.no_default_features,
        is_transitive: false,
    };

    let results = run_all_checks(&crate_info);
    let score = compute_score(&results);

    if args.json {
        let report = stylus_compat_core::types::CrateReport {
            crate_info,
            results,
            score,
        };
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("\n{} {}", "Checking crate:".bold(), args.crate_name.cyan());
    println!("{}", "─".repeat(50));

    for result in &results {
        let icon = match result.severity {
            Severity::Pass => "✓".green(),
            Severity::Warning => "⚠".yellow(),
            Severity::Error => "✗".red(),
        };
        println!(
            "  {} [{}] {}",
            icon,
            result.check_name,
            super::indent_message(&result.message)
        );
    }

    println!("{}", "─".repeat(50));
    let score_display = format!("Score: {score}");
    let score_colored = if score.value >= 90 {
        score_display.green()
    } else if score.value >= 50 {
        score_display.yellow()
    } else {
        score_display.red()
    };
    println!("  {score_colored}");
    println!();

    Ok(())
}
