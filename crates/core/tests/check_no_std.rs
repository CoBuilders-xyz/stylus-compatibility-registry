use std::path::PathBuf;
use stylus_compat_core::types::Severity;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("fixtures")
}

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
}

#[test]
fn analyze_fixture_project_detects_incompatible_deps() {
    let manifest = fixtures_dir().join("test-project/Cargo.toml");
    let data = data_dir();

    let report = stylus_compat_core::analyze_project(&manifest, Some(&data), false).unwrap();

    assert!(
        report.error_count > 0,
        "Expected errors for std-dependent crates, got 0"
    );

    let tokio_report = report
        .crate_reports
        .iter()
        .find(|r| r.crate_info.name == "tokio")
        .expect("tokio should be in the report");

    let has_std_error = tokio_report
        .results
        .iter()
        .any(|r| r.severity == Severity::Error && r.check_name == "no_std");

    assert!(has_std_error, "tokio should fail the no_std check");
}

#[test]
fn analyze_fixture_project_passes_compatible_deps() {
    let manifest = fixtures_dir().join("test-project/Cargo.toml");
    let data = data_dir();

    let report = stylus_compat_core::analyze_project(&manifest, Some(&data), false).unwrap();

    let serde_report = report
        .crate_reports
        .iter()
        .find(|r| r.crate_info.name == "serde")
        .expect("serde should be in the report");

    let no_std_result = serde_report
        .results
        .iter()
        .find(|r| r.check_name == "no_std")
        .expect("no_std check should exist for serde");

    assert_eq!(
        no_std_result.severity,
        Severity::Pass,
        "serde (with default-features=false) should pass no_std check"
    );
}

#[test]
fn analyze_fixture_detects_float_warning() {
    let manifest = fixtures_dir().join("test-project/Cargo.toml");
    let data = data_dir();

    let report = stylus_compat_core::analyze_project(&manifest, Some(&data), false).unwrap();

    let nalgebra_report = report
        .crate_reports
        .iter()
        .find(|r| r.crate_info.name == "nalgebra")
        .expect("nalgebra should be in the report");

    let has_float_warning = nalgebra_report
        .results
        .iter()
        .any(|r| r.severity == Severity::Warning && r.check_name == "float_usage");

    assert!(
        has_float_warning,
        "nalgebra should trigger a float_usage warning"
    );
}

#[test]
fn strict_mode_would_fail_with_errors() {
    let manifest = fixtures_dir().join("test-project/Cargo.toml");
    let data = data_dir();

    let report = stylus_compat_core::analyze_project(&manifest, Some(&data), false).unwrap();

    assert!(
        report.error_count > 0,
        "Fixture project has incompatible deps — strict mode should fail"
    );
    assert!(
        report.overall_score.value < 90,
        "Score should be below 90 with incompatible deps, got {}",
        report.overall_score.value
    );
}
