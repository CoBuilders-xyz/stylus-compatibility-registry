use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

/// Checks if a crate can emit SIMD opcodes.
///
/// The Stylus VM rejects the WASM SIMD proposal, so a contract carrying `v128`
/// opcodes fails to activate even if it compiles.
///
/// This is a warning rather than an error: these crates gate their SIMD paths behind
/// `target_feature = "simd128"`, which is off by default for `wasm32-unknown-unknown`,
/// so they fall back to scalar code unless the build turns it on.
///
/// Only crate names are matched. A contract reaching for `core::simd` directly, or
/// pulling SIMD in through a crate not listed here, is not detected.
pub struct SimdCheck;

const KNOWN_SIMD_CRATES: &[&str] = &[
    "packed_simd",
    "packed_simd_2",
    "wide",
    "simdeez",
    "ultraviolet",
];

impl CrateCheck for SimdCheck {
    fn name(&self) -> &str {
        "simd_usage"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        if KNOWN_SIMD_CRATES.contains(&crate_info.name.as_str()) {
            return CheckResult::warning(
                self.name(),
                format!(
                    "`{}` emits SIMD opcodes when simd128 is enabled, and the Stylus VM rejects them",
                    crate_info.name
                ),
            );
        }

        CheckResult::pass(
            self.name(),
            format!("`{}` is not a known SIMD crate", crate_info.name),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Severity;

    fn crate_info(name: &str) -> CrateInfo {
        CrateInfo {
            name: name.to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        }
    }

    #[test]
    fn detects_simd_crate() {
        let result = SimdCheck.run(&crate_info("wide"));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result.message.contains("SIMD opcodes"));
    }

    #[test]
    fn detects_renamed_packed_simd_fork() {
        let result = SimdCheck.run(&crate_info("packed_simd_2"));
        assert_eq!(result.severity, Severity::Warning);
    }

    #[test]
    fn passes_non_simd_crate() {
        let result = SimdCheck.run(&crate_info("tiny-keccak"));
        assert_eq!(result.severity, Severity::Pass);
    }
}
