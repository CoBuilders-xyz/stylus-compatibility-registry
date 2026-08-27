use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

/// Checks if a crate is known to be incompatible with the `wasm32-unknown-unknown` target.
///
/// TODO: Implement actual compilation check by attempting `cargo check --target wasm32-unknown-unknown`
/// in a sandboxed environment. For now, this uses a blocklist of crates that depend on
/// OS-level APIs (filesystem, networking, threads) unavailable in WASM.
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

impl CrateCheck for WasmTargetCheck {
    fn name(&self) -> &str {
        "wasm_target"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_wasm_incompatible_crate() {
        let info = CrateInfo {
            name: "libc".to_string(),
            version: Some("0.2.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = WasmTargetCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Error);
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
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }
}
