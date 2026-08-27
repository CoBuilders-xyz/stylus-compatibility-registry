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
#[test]
fn analyze_with_transitive_finds_incompatible_transitive_dep() {
    let manifest = fixtures_dir().join("transitive-test/Cargo.toml");
    let direct = stylus_compat_core::analyze_project(&manifest, None).unwrap();
    let full = stylus_compat_core::analyze_project_with_transitive(&manifest, None, true).unwrap();
    assert!(full.crate_reports.len() > direct.crate_reports.len());
    let libc = full
        .crate_reports
        .iter()
        .find(|r| r.crate_info.name == "libc")
        .unwrap();
    assert!(libc.crate_info.is_transitive);
    assert!(libc
        .results
        .iter()
        .any(|r| r.severity == Severity::Error && r.check_name == "wasm_target"));
    assert!(full
        .crate_reports
        .iter()
        .all(|report| report.crate_info.name != "mio"));
    assert!(full
        .crate_reports
        .iter()
        .all(|report| report.crate_info.name != "winapi"));
    assert!(direct
        .crate_reports
        .iter()
        .all(|r| r.crate_info.name != "libc"));
}
