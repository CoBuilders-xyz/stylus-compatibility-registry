use clap::Args;
use colored::Colorize;
use std::path::PathBuf;
use stylus_compat_core::types::Severity;

#[derive(Args)]
pub struct CheckDepsArgs {
    #[arg(short, long, default_value = "Cargo.toml")]
    pub manifest: PathBuf,
    #[arg(short, long)]
    pub data_dir: Option<PathBuf>,
    #[arg(long)]
    pub strict: bool,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub include_transitive: bool,
}

pub fn run(args: CheckDepsArgs) -> Result<(), Box<dyn std::error::Error>> {
    let report = stylus_compat_core::analyze_project_with_transitive(
        &args.manifest,
        args.data_dir.as_deref(),
        args.include_transitive,
    )?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_report(&report);
    }
    if args.strict && report.error_count > 0 {
        std::process::exit(1);
    }
    Ok(())
}

fn print_report(report: &stylus_compat_core::types::ProjectReport) {
    println!("\n{} {}", "Analyzing:".bold(), report.manifest_path.cyan());
    println!(
        "{} dependencies found\n",
        report.crate_reports.len().to_string().bold()
    );
    for crate_report in &report.crate_reports {
        let name = &crate_report.crate_info.name;
        let version = crate_report.crate_info.version.as_deref().unwrap_or("*");
        let transitive = if crate_report.crate_info.is_transitive {
            " (transitive)".dimmed().to_string()
        } else {
            String::new()
        };
        let has_issues = crate_report
            .results
            .iter()
            .any(|r| r.severity != Severity::Pass);
        if has_issues {
            println!(
                "  {} {}{} {}",
                "●".red(),
                name.bold(),
                transitive,
                version.dimmed()
            );
            for result in &crate_report.results {
                if result.severity == Severity::Pass {
                    continue;
                }
                let icon = match result.severity {
                    Severity::Warning => "⚠".yellow(),
                    Severity::Error => "✗".red(),
                    Severity::Pass => unreachable!(),
                };
                println!("    {} {}", icon, result.message);
            }
        } else {
            println!(
                "  {} {}{} {}",
                "●".green(),
                name,
                transitive,
                version.dimmed()
            );
        }
    }
    println!("\n{}", "─".repeat(50));
    println!(
        "  Errors: {}  Warnings: {}",
        report.error_count.to_string().red().bold(),
        report.warning_count.to_string().yellow().bold()
    );
    let score_display = format!("  Overall: {}", report.overall_score);
    let score_colored = if report.overall_score.value >= 90 {
        score_display.green()
    } else if report.overall_score.value >= 50 {
        score_display.yellow()
    } else {
        score_display.red()
    };
    println!("{score_colored}\n");
}
