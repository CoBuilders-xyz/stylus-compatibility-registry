pub mod checks;
pub mod manifest;
pub mod registry;
pub mod score;
pub mod types;

use std::path::Path;

use checks::run_all_checks;
use registry::KnownCratesRegistry;
use score::{compute_project_score, compute_score};
use types::{CrateReport, ProjectReport, Severity};

#[derive(Debug, thiserror::Error)]
pub enum AnalyzeError {
    #[error(transparent)]
    Manifest(#[from] manifest::ManifestError),
    #[error(transparent)]
    Registry(#[from] registry::RegistryError),
}

/// Analyzes a project's direct dependencies against Stylus compatibility constraints.
///
/// Use [`analyze_project_with_transitive`] to optionally include the full dependency tree.
pub fn analyze_project(
    manifest_path: &Path,
    data_dir: Option<&Path>,
) -> Result<ProjectReport, AnalyzeError> {
    analyze_project_with_transitive(manifest_path, data_dir, false)
}

/// Analyzes a project's Cargo.toml against Stylus compatibility constraints.
///
/// This is the main entry point for the library. It:
/// 1. Parses the manifest to extract dependencies
/// 2. Loads the known-crates registry (if a data directory is provided)
/// 3. Runs all compatibility checks on each dependency
/// 4. Computes per-crate and overall scores
pub fn analyze_project_with_transitive(
    manifest_path: &Path,
    data_dir: Option<&Path>,
    include_transitive: bool,
) -> Result<ProjectReport, AnalyzeError> {
    let deps = if include_transitive {
        manifest::resolve_full_tree(manifest_path)?
    } else {
        manifest::parse_dependencies(manifest_path)?
    };

    let mut registry = KnownCratesRegistry::new();
    if let Some(dir) = data_dir {
        registry.load_data_dir(dir)?;
    }

    let mut crate_reports = Vec::new();
    let mut total_errors = 0;
    let mut total_warnings = 0;

    for dep in &deps {
        let results = run_all_checks(dep, registry.lookup(&dep.name));

        let error_count = results
            .iter()
            .filter(|r| r.severity == Severity::Error)
            .count();
        let warning_count = results
            .iter()
            .filter(|r| r.severity == Severity::Warning)
            .count();
        total_errors += error_count;
        total_warnings += warning_count;

        let score = compute_score(&results);

        crate_reports.push(CrateReport {
            crate_info: dep.clone(),
            results,
            score,
        });
    }

    let crate_scores: Vec<_> = crate_reports.iter().map(|r| r.score.clone()).collect();
    let overall_score = compute_project_score(&crate_scores);

    Ok(ProjectReport {
        manifest_path: manifest_path.display().to_string(),
        crate_reports,
        overall_score,
        error_count: total_errors,
        warning_count: total_warnings,
    })
}
