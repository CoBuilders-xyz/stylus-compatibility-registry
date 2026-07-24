use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

/// Checks if a crate is known to use floating-point operations.
///
/// Stylus WASM contracts must avoid `f32`/`f64` — the Arbitrum WASM runtime disallows
/// floating-point opcodes for determinism. Crates that perform float arithmetic will cause
/// activation failure on-chain.
///
/// TODO: Implement source-level analysis by downloading the crate source and scanning for
/// `f32`, `f64`, and float literals. For now, uses a blocklist of crates that are known
/// to rely on floating-point math.
pub struct FloatUsageCheck;

const KNOWN_FLOAT_CRATES: &[&str] = &[
    "num",
    "nalgebra",
    "ndarray",
    "rand",
    "ordered-float",
    "half",
    "float-ord",
    "approx",
    "decorum",
    "float-cmp",
];

impl CrateCheck for FloatUsageCheck {
    fn name(&self) -> &str {
        "float_usage"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        if KNOWN_FLOAT_CRATES.contains(&crate_info.name.as_str()) {
            return CheckResult::warning(
                self.name(),
                format!(
                    "`{}` may use floating-point operations — Stylus disallows f32/f64 for determinism",
                    crate_info.name
                ),
            );
        }

        CheckResult::pass(
            self.name(),
            format!(
                "`{}` is not in the known float-using blocklist",
                crate_info.name
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_on_float_crate() {
        let info = CrateInfo {
            name: "nalgebra".to_string(),
            version: Some("0.33.0".to_string()),
            features: vec![],
            default_features: true,
        };
        let result = FloatUsageCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Warning);
    }

    #[test]
    fn passes_non_float_crate() {
        let info = CrateInfo {
            name: "tiny-keccak".to_string(),
            version: Some("2.0.0".to_string()),
            features: vec![],
            default_features: true,
        };
        let result = FloatUsageCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }
}
