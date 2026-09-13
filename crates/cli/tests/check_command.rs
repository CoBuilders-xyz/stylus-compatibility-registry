use std::path::PathBuf;
use std::process::Command;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
}

/// Runs the CLI with a cargo binary that does not exist, so the `wasm_target` check
/// degrades instead of compiling for wasm32 on every assertion.
fn run_check(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_stylus-registry"))
        .args(args)
        .env("STYLUS_COMPAT_CARGO", "stylus-compat-no-such-cargo")
        .output()
        .expect("the CLI binary should run");
    String::from_utf8(output.stdout).expect("stdout should be utf-8")
}

#[test]
fn reads_the_registry_when_given_a_data_dir() {
    let data = data_dir();
    let out = run_check(&[
        "check",
        "tokio",
        "--json",
        "--data-dir",
        data.to_str().unwrap(),
    ]);

    assert!(
        out.contains("requires std, threads, I/O polling"),
        "expected tokio's registry notes in the output, got:\n{out}"
    );
}

#[test]
fn falls_back_to_the_blocklists_without_a_data_dir() {
    let out = run_check(&["check", "tokio", "--json"]);

    assert!(
        out.contains("is an async runtime"),
        "expected the blocklist message, got:\n{out}"
    );
    assert!(
        !out.contains("requires std, threads, I/O polling"),
        "registry notes should not appear without a data dir, got:\n{out}"
    );
}

/// Runs the CLI the same way but keeps the exit status and stderr, which is what the
/// rejected-input path is about.
fn run_raw(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_stylus-registry"))
        .args(args)
        .env("STYLUS_COMPAT_CARGO", "stylus-compat-no-such-cargo")
        .output()
        .expect("the CLI binary should run")
}

fn parsed_features(args: &[&str]) -> Vec<String> {
    let out = run_check(args);
    let report: serde_json::Value = serde_json::from_str(&out).expect("the report should be json");
    report["crate_info"]["features"]
        .as_array()
        .expect("features should be an array")
        .iter()
        .map(|f| f.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn rejects_a_feature_name_cargo_cannot_accept() {
    let output = run_raw(&["check", "tiny-keccak", "--features", "not.valid"]);

    assert!(!output.status.success(), "the run should fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid feature name `not.valid`"),
        "got: {stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "no report should be printed for rejected input"
    );
}

#[test]
fn trims_whitespace_around_comma_separated_features() {
    let features = parsed_features(&[
        "check",
        "tiny-keccak",
        "--features",
        "keccak, sha3",
        "--json",
    ]);
    assert_eq!(features, ["keccak", "sha3"]);
}

#[test]
fn accepts_repeated_feature_flags_and_no_default_features() {
    let features = parsed_features(&[
        "check",
        "tiny-keccak",
        "--features",
        "keccak",
        "--features",
        "sha3",
        "--no-default-features",
        "--json",
    ]);
    assert_eq!(features, ["keccak", "sha3"]);
}

#[test]
fn rejects_an_empty_feature_left_by_a_trailing_comma() {
    let output = run_raw(&["check", "tiny-keccak", "--features", "keccak,"]);
    assert!(!output.status.success());
}
