use crate::checks::CrateCheck;
use crate::registry::KnownCrateEntry;
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

impl AsyncUsageCheck {
    pub fn check_against_registry(
        &self,
        crate_info: &CrateInfo,
        entry: Option<&KnownCrateEntry>,
    ) -> CheckResult {
        if let Some(entry) = entry {
            if entry.has_async {
                return CheckResult::error(
                    self.name(),
                    format!(
                        "`{}` pulls in an async runtime: {}",
                        crate_info.name,
                        entry.notes.as_deref().unwrap_or("no details")
                    ),
                );
            }
            return CheckResult::pass(
                self.name(),
                format!("`{}` is verified free of async runtimes", crate_info.name),
            );
        }

        // Not in registry, fall back to the blocklist
        self.run(crate_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    fn crate_info(name: &str) -> CrateInfo {
        CrateInfo {
            name: name.to_string(),
            version: None,
            features: vec![],
            default_features: true,
            is_transitive: false,
        }
    }

    fn registry_entry(name: &str, has_async: bool) -> KnownCrateEntry {
        KnownCrateEntry {
            name: name.to_string(),
            requires_std: true,
            has_float: false,
            has_async,
            max_version: None,
            alternative: None,
            notes: Some("async web framework".to_string()),
        }
    }

    #[test]
    fn detects_async_runtime() {
        let result = AsyncUsageCheck.run(&crate_info("tokio"));
        assert_eq!(result.severity, Severity::Error);
    }

    #[test]
    fn passes_sync_crate() {
        let result = AsyncUsageCheck.run(&crate_info("alloc-stdlib"));
        assert_eq!(result.severity, Severity::Pass);
    }

    #[test]
    fn detects_async_via_registry() {
        let entry = registry_entry("axum", true);
        let result = AsyncUsageCheck.check_against_registry(&crate_info("axum"), Some(&entry));
        assert_eq!(result.severity, Severity::Error);
        assert!(result.message.contains("async web framework"));
    }

    #[test]
    fn registry_wins_over_the_blocklist() {
        let entry = registry_entry("tokio", false);
        let result = AsyncUsageCheck.check_against_registry(&crate_info("tokio"), Some(&entry));
        assert_eq!(result.severity, Severity::Pass);
    }

    #[test]
    fn falls_back_to_the_blocklist_without_an_entry() {
        let result = AsyncUsageCheck.check_against_registry(&crate_info("tokio"), None);
        assert_eq!(result.severity, Severity::Error);
    }
}
