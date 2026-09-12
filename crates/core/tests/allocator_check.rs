use std::path::PathBuf;
use std::process::Command;
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
/// dependency tree down so the advisory message reflects the SDK configuration.
#[test]
fn reports_an_advisory_when_the_project_also_pulls_the_sdk() {
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

    assert_eq!(severity_of("wee_alloc"), Severity::Warning);
    let allocator = report
        .crate_reports
        .iter()
        .find(|r| r.crate_info.name == "wee_alloc")
        .unwrap()
        .results
        .iter()
        .find(|r| r.check_name == "allocator")
        .unwrap();
    assert!(allocator
        .message
        .contains("may conflict with stylus-sdk's mini-alloc"));
    assert!(!allocator
        .message
        .contains("no dependency tree was inspected"));
    assert_eq!(severity_of("dlmalloc"), Severity::Pass);
    assert_eq!(severity_of("tiny-keccak"), Severity::Pass);
}

/// The advisory severity rests on this build being valid. Depending on `wee_alloc`
/// next to the SDK's mini-alloc is not a duplicate registration, and if that ever
/// stops holding the check should go back to reporting an error.
#[test]
#[ignore = "requires network access and the wasm32-unknown-unknown target"]
fn the_advisory_fixture_builds_for_wasm32() {
    let manifest = fixtures_dir().join("allocator-advisory/Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "build",
            "--target",
            "wasm32-unknown-unknown",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .expect("cargo should run");

    assert!(
        output.status.success(),
        "wee_alloc and stylus-sdk should build together:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
