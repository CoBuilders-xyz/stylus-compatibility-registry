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

/// Covers the wiring the unit tests cannot reach: `analyze_project` has to hand the
/// dependency tree down, and passing `None` there would silently downgrade every
/// allocator conflict to a warning.
#[test]
fn reports_an_error_when_the_project_also_pulls_the_sdk() {
    // The fake cargo binary keeps `wasm_target` from compiling for wasm32 here.
    std::env::set_var("STYLUS_COMPAT_CARGO", "stylus-compat-no-such-cargo");

    let manifest = fixtures_dir().join("stylus-project/Cargo.toml");
    let report = stylus_compat_core::analyze_project(&manifest, None).unwrap();

    let severity_of = |name: &str| {
        report
            .crate_reports
            .iter()
            .find(|r| r.crate_info.name == name)
            .unwrap_or_else(|| panic!("{name} should be in the report"))
            .results
            .iter()
            .find(|r| r.check_name == "allocator")
            .expect("the allocator check should run on every crate")
            .severity
    };

    assert_eq!(severity_of("wee_alloc"), Severity::Error);
    assert_eq!(severity_of("dlmalloc"), Severity::Pass);
    assert_eq!(severity_of("tiny-keccak"), Severity::Pass);
}
