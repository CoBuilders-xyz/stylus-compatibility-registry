use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

/// Checks if a crate depends on async runtimes.
///
/// Stylus contracts execute synchronously within the Arbitrum WASM VM. Async runtimes
/// (tokio, async-std) bring in thread spawning, I/O polling, and timers that are
/// incompatible with the WASM sandbox.
///
/// TODO: Implement dependency-tree analysis to detect transitive async runtime usage.
/// For now, uses a blocklist of known async-runtime crates.
pub struct AsyncUsageCheck;

const KNOWN_ASYNC_CRATES: &[&str] = &[
    "tokio",
    "async-std",
    "smol",
    "futures-executor",
    "actix-rt",
    "embassy-executor",
];

impl CrateCheck for AsyncUsageCheck {
    fn name(&self) -> &str {
        "async_usage"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        if KNOWN_ASYNC_CRATES.contains(&crate_info.name.as_str()) {
            return CheckResult::error(
                self.name(),
                format!(
                    "`{}` is an async runtime — Stylus contracts execute synchronously",
                    crate_info.name
                ),
            );
        }

        CheckResult::pass(
            self.name(),
            format!("`{}` is not a known async runtime", crate_info.name),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_async_runtime() {
        let info = CrateInfo {
            name: "tokio".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = AsyncUsageCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Error);
    }

    #[test]
    fn passes_sync_crate() {
        let info = CrateInfo {
            name: "alloc-stdlib".to_string(),
            version: None,
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = AsyncUsageCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }
}
