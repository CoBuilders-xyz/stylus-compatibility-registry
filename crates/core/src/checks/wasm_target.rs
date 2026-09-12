use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use wait_timeout::ChildExt;

const CARGO_TIMEOUT: Duration = Duration::from_secs(120);

/// Diagnostic lines kept in a report before it stops being readable. Caps both the
/// error list and the raw tail shown when no error line was recognised.
const MAX_REPORTED_LINES: usize = 10;

/// Checks whether a crate compiles for the `wasm32-unknown-unknown` target.
///
/// Attempts `cargo check --target wasm32-unknown-unknown` in a temporary project.
/// Falls back to a blocklist when compilation cannot be run (e.g. no network or cargo).
pub struct WasmTargetCheck;

const KNOWN_WASM_INCOMPATIBLE: &[&str] = &[
    "libc",
    "nix",
    "mio",
    "socket2",
    "signal-hook",
    "ctrlc",
    "notify",
    "fs_extra",
    "walkdir",
    "inotify",
];

/// Outcome of attempting a WASM compilation check.
#[derive(Debug, PartialEq, Eq)]
pub enum CompileCheckOutcome {
    Pass(String),
    Error(String),
    Unavailable,
}

fn cargo_bin() -> String {
    std::env::var("STYLUS_COMPAT_CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn is_valid_version(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 64
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
}

fn validate_crate_info(crate_info: &CrateInfo) -> Result<(), &'static str> {
    if !is_valid_crate_name(&crate_info.name) {
        return Err("invalid crate name");
    }
    if let Some(version) = &crate_info.version {
        if !is_valid_version(version) {
            return Err("invalid version string");
        }
    }
    for feature in &crate_info.features {
        if !feature
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/')
        {
            return Err("invalid feature name");
        }
    }
    Ok(())
}

fn dependency_spec(crate_info: &CrateInfo) -> String {
    let name = &crate_info.name;
    let simple = crate_info.version.is_some()
        && crate_info.default_features
        && crate_info.features.is_empty();

    if simple {
        return format!("{name} = \"{}\"", crate_info.version.as_ref().unwrap());
    }

    let mut parts = Vec::new();
    if let Some(version) = &crate_info.version {
        parts.push(format!("version = \"{version}\""));
    } else {
        parts.push("version = \"*\"".to_string());
    }
    if !crate_info.default_features {
        parts.push("default-features = false".to_string());
    }
    if !crate_info.features.is_empty() {
        let features: Vec<String> = crate_info
            .features
            .iter()
            .map(|feature| format!("\"{feature}\""))
            .collect();
        parts.push(format!("features = [{}]", features.join(", ")));
    }

    format!("{name} = {{ {} }}", parts.join(", "))
}

fn write_minimal_project(dir: &Path, crate_info: &CrateInfo) -> io::Result<()> {
    let cargo_toml = format!(
        r#"[package]
name = "wasm-compat-check"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"

[dependencies]
{}
"#,
        dependency_spec(crate_info)
    );

    fs::write(dir.join("Cargo.toml"), cargo_toml)?;
    fs::create_dir_all(dir.join("src"))?;
    fs::write(dir.join("src/lib.rs"), "")?;
    Ok(())
}

fn is_unavailable_output(output: &str) -> bool {
    let text = output.to_lowercase();
    const UNAVAILABLE_PATTERNS: &[&str] = &[
        "failed to fetch",
        "failed to download",
        "could not connect",
        "connection refused",
        "network unreachable",
        "dns error",
        "failed to resolve host",
        "couldn't resolve host",
        "failed to get package",
        "no matching package named",
        "target `wasm32-unknown-unknown` not installed",
        "target 'wasm32-unknown-unknown' not installed",
    ];

    UNAVAILABLE_PATTERNS
        .iter()
        .any(|pattern| text.contains(pattern))
}

fn extract_compiler_errors(output: &str) -> String {
    let errors: Vec<&str> = output
        .lines()
        .filter(|line| line.contains("error:") || line.contains("error[E"))
        .collect();

    if errors.is_empty() {
        return output
            .lines()
            .rev()
            .take(MAX_REPORTED_LINES)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
    }

    let head = errors
        .iter()
        .take(MAX_REPORTED_LINES)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");

    // A crate like packed_simd repeats the same few codes hundreds of times, and the
    // whole list buries the report it is printed into.
    if errors.len() > MAX_REPORTED_LINES {
        format!(
            "{head}\n... and {} more errors",
            errors.len() - MAX_REPORTED_LINES
        )
    } else {
        head
    }
}

fn classify_output(crate_name: &str, output: &Output) -> CompileCheckOutcome {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    if is_unavailable_output(&combined) {
        return CompileCheckOutcome::Unavailable;
    }

    if output.status.success() {
        return CompileCheckOutcome::Pass(format!(
            "`{crate_name}` compiles for wasm32-unknown-unknown"
        ));
    }

    CompileCheckOutcome::Error(extract_compiler_errors(&combined))
}

/// Runs `cargo check --target wasm32-unknown-unknown` for the given crate.
pub fn compile_check(crate_info: &CrateInfo) -> CompileCheckOutcome {
    compile_check_with_cargo(crate_info, &cargo_bin())
}

pub fn compile_check_with_cargo(crate_info: &CrateInfo, cargo: &str) -> CompileCheckOutcome {
    if validate_crate_info(crate_info).is_err() {
        return CompileCheckOutcome::Unavailable;
    }

    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return CompileCheckOutcome::Unavailable,
    };

    if write_minimal_project(temp_dir.path(), crate_info).is_err() {
        return CompileCheckOutcome::Unavailable;
    }

    // cargo writes to files, not pipes. Nothing drains a pipe while wait_timeout
    // blocks, so a build noisy enough to fill the pipe buffer used to stall until
    // the timeout and get reported as a check that could not run.
    let stdout_path = temp_dir.path().join("cargo-stdout.log");
    let stderr_path = temp_dir.path().join("cargo-stderr.log");
    let (stdout, stderr) = match (File::create(&stdout_path), File::create(&stderr_path)) {
        (Ok(out), Ok(err)) => (out, err),
        _ => return CompileCheckOutcome::Unavailable,
    };

    let mut child = match Command::new(cargo)
        .args(["check", "--target", "wasm32-unknown-unknown", "--quiet"])
        .current_dir(temp_dir.path())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return CompileCheckOutcome::Unavailable,
    };

    match child.wait_timeout(CARGO_TIMEOUT) {
        Ok(Some(status)) => classify_output(
            &crate_info.name,
            &Output {
                status,
                stdout: fs::read(&stdout_path).unwrap_or_default(),
                stderr: fs::read(&stderr_path).unwrap_or_default(),
            },
        ),
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            CompileCheckOutcome::Unavailable
        }
        Err(_) => CompileCheckOutcome::Unavailable,
    }
}

impl WasmTargetCheck {
    fn blocklist_check(&self, crate_info: &CrateInfo) -> CheckResult {
        if KNOWN_WASM_INCOMPATIBLE.contains(&crate_info.name.as_str()) {
            return CheckResult::error(
                self.name(),
                format!(
                    "`{}` uses OS APIs incompatible with wasm32-unknown-unknown",
                    crate_info.name
                ),
            );
        }

        CheckResult::pass(
            self.name(),
            format!(
                "`{}` is not in the known wasm-incompatible blocklist",
                crate_info.name
            ),
        )
    }

    fn run_with_cargo(&self, crate_info: &CrateInfo, cargo: &str) -> CheckResult {
        if KNOWN_WASM_INCOMPATIBLE.contains(&crate_info.name.as_str()) {
            return self.blocklist_check(crate_info);
        }

        match compile_check_with_cargo(crate_info, cargo) {
            CompileCheckOutcome::Pass(message) => CheckResult::pass(self.name(), message),
            CompileCheckOutcome::Error(errors) => CheckResult::error(
                self.name(),
                format!(
                    "`{}` failed wasm32-unknown-unknown compilation check:\n{errors}",
                    crate_info.name
                ),
            ),
            CompileCheckOutcome::Unavailable => self.blocklist_check(crate_info),
        }
    }
}

impl CrateCheck for WasmTargetCheck {
    fn name(&self) -> &str {
        "wasm_target"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        self.run_with_cargo(crate_info, &cargo_bin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    #[test]
    fn detects_wasm_incompatible_crate() {
        let info = CrateInfo {
            name: "mio".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = WasmTargetCheck.run(&info);
        assert_eq!(result.severity, Severity::Error);
    }

    #[test]
    fn passes_wasm_compatible_crate() {
        let info = CrateInfo {
            name: "alloy-primitives".to_string(),
            version: Some("0.8.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = WasmTargetCheck.run(&info);
        assert_eq!(result.severity, Severity::Pass);
    }

    #[test]
    #[ignore = "requires network access and the wasm32-unknown-unknown target"]
    fn compile_check_passes_compatible_crate() {
        let info = CrateInfo {
            name: "hex".to_string(),
            version: Some("0.4.3".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let outcome = compile_check(&info);
        assert_eq!(
            outcome,
            CompileCheckOutcome::Pass("`hex` compiles for wasm32-unknown-unknown".to_string())
        );
    }

    #[test]
    #[ignore = "requires network access and the wasm32-unknown-unknown target"]
    fn compile_check_fails_incompatible_crate() {
        let info = CrateInfo {
            name: "mio".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let outcome = compile_check(&info);
        assert!(matches!(outcome, CompileCheckOutcome::Error(_)));
    }

    #[test]
    fn known_incompatible_crate_short_circuits_compilation() {
        let info = CrateInfo {
            name: "nix".to_string(),
            version: Some("0.27.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = WasmTargetCheck.run_with_cargo(&info, "/nonexistent/cargo");
        assert_eq!(result.severity, Severity::Error);
        assert!(result.message.contains("OS APIs incompatible"));
    }

    #[test]
    fn run_falls_back_to_blocklist_pass_for_unknown_crate() {
        let info = CrateInfo {
            name: "some-unknown-crate-xyz".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = WasmTargetCheck.run_with_cargo(&info, "/nonexistent/cargo");
        assert_eq!(result.severity, Severity::Pass);
        assert!(result.message.contains("blocklist"));
    }

    #[test]
    fn reports_every_error_under_the_cap() {
        let output = "error[E0001]: one\nerror[E0002]: two";
        assert_eq!(extract_compiler_errors(output), output);
    }

    #[test]
    fn caps_the_reported_compiler_errors() {
        let output = (0..25)
            .map(|n| format!("error[E0512]: failure {n}"))
            .collect::<Vec<_>>()
            .join("\n");
        let extracted = extract_compiler_errors(&output);
        assert_eq!(extracted.lines().count(), MAX_REPORTED_LINES + 1);
        assert!(extracted.ends_with("... and 15 more errors"));
    }

    /// A pipe buffer holds about 64 KB. Output past that used to stall the child
    /// until the timeout, which surfaced as a passing check instead of a failure.
    #[cfg(unix)]
    #[test]
    fn classifies_a_failure_that_outgrows_the_pipe_buffer() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let fake_cargo = dir.path().join("cargo");
        fs::write(
            &fake_cargo,
            "#!/bin/sh\nawk 'BEGIN { while (n++ < 5000) \
             print \"error[E0557]: feature has been removed\" }' >&2\nexit 1\n",
        )
        .unwrap();
        fs::set_permissions(&fake_cargo, fs::Permissions::from_mode(0o755)).unwrap();

        let info = CrateInfo {
            name: "noisy-crate".to_string(),
            version: Some("0.3.9".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };

        match compile_check_with_cargo(&info, fake_cargo.to_str().unwrap()) {
            CompileCheckOutcome::Error(errors) => assert!(errors.contains("E0557")),
            other => panic!("a failing build should be an error, got {other:?}"),
        }
    }

    #[test]
    fn rejects_invalid_crate_name() {
        let info = CrateInfo {
            name: "evil\n[build-dependencies]\nattacker = \"1\"".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let outcome = compile_check(&info);
        assert_eq!(outcome, CompileCheckOutcome::Unavailable);
    }

    #[test]
    fn rejects_invalid_version() {
        let info = CrateInfo {
            name: "hex".to_string(),
            version: Some("1.0\"\n[build-dependencies]\nx = \"1\"".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let outcome = compile_check(&info);
        assert_eq!(outcome, CompileCheckOutcome::Unavailable);
    }
}
