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

#[test]
fn allocator_name_warning_does_not_fail_strict_mode() {
    let root = data_dir().parent().unwrap().to_path_buf();
    let output = Command::new(env!("CARGO_BIN_EXE_stylus-registry"))
        .args(["check-deps", "--strict", "--json", "--manifest"])
        .arg(root.join("fixtures/allocator-advisory/Cargo.toml"))
        .arg("--data-dir")
        .arg(data_dir())
        // Isolate allocator classification from the compilation check. The fixture
        // also supports a separate real cargo build for wasm32-unknown-unknown.
        .env("STYLUS_COMPAT_CARGO", "stylus-compat-no-such-cargo")
        .output()
        .expect("the CLI binary should run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["error_count"], 0);
    let allocator = report["crate_reports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["crate_info"]["name"] == "wee_alloc")
        .unwrap()["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["check_name"] == "allocator")
        .unwrap();
    assert_eq!(allocator["severity"], "Warning");
    assert!(allocator["message"]
        .as_str()
        .unwrap()
        .contains("may conflict"));
}
