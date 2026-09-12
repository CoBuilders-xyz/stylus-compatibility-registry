use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

/// Flags allocator dependencies that may be registered with `#[global_allocator]`.
///
/// Crate names do not establish whether an allocator is registered by the project.
/// Results are advisory even when stylus-sdk has mini-alloc enabled: an actual
/// duplicate registration must be established from code or a build diagnostic.
pub struct AllocatorCheck;

/// Allocator implementations to flag for review, not confirmed registrations.
/// The SDK's own mini-alloc and std's dlmalloc are not flagged by this heuristic.
const ALLOCATOR_CRATES: &[&str] = &["wee_alloc", "talc", "lol_alloc"];

impl CrateCheck for AllocatorCheck {
    fn name(&self) -> &str {
        "allocator"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        self.check_in_tree(crate_info, None)
    }
}

impl AllocatorCheck {
    /// `deps` is the project's dependency tree, or `None` when a single crate name was
    /// checked and there is no tree to look at.
    pub fn check_in_tree(&self, crate_info: &CrateInfo, deps: Option<&[CrateInfo]>) -> CheckResult {
        if !ALLOCATOR_CRATES.contains(&crate_info.name.as_str()) {
            return CheckResult::pass(
                self.name(),
                format!(
                    "`{}` is not flagged as a conflicting allocator",
                    crate_info.name
                ),
            );
        }

        match deps {
            Some(deps) if sdk_registers_mini_alloc(deps) => CheckResult::warning(
                self.name(),
                format!(
                    "`{}` provides an allocator; registering it with #[global_allocator] may \
                     conflict with stylus-sdk's mini-alloc; registration was not inspected",
                    crate_info.name
                ),
            ),
            Some(deps) if deps.iter().any(|dep| dep.name == "stylus-sdk") => {
                CheckResult::warning(
                    self.name(),
                    format!(
                        "`{}` provides an allocator; stylus-sdk has mini-alloc disabled in \
                         the inspected dependencies; global allocator registration was not inspected",
                        crate_info.name
                    ),
                )
            }
            Some(_) => CheckResult::warning(
                self.name(),
                format!(
                    "`{}` provides an allocator; stylus-sdk was not found in the \
                     inspected dependencies; global allocator registration was not inspected",
                    crate_info.name
                ),
            ),
            None => CheckResult::warning(
                self.name(),
                format!(
                    "`{}` provides an allocator; no dependency tree was inspected, \
                     so a conflict with stylus-sdk's mini-alloc is possible if registered \
                     with #[global_allocator]",
                    crate_info.name
                ),
            ),
        }
    }
}

fn sdk_registers_mini_alloc(deps: &[CrateInfo]) -> bool {
    deps.iter().any(|dep| {
        dep.name == "stylus-sdk"
            && (dep.default_features || dep.features.iter().any(|f| f == "mini-alloc"))
    })
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

    fn sdk(default_features: bool, features: &[&str]) -> CrateInfo {
        CrateInfo {
            features: features.iter().map(|f| f.to_string()).collect(),
            default_features,
            ..crate_info("stylus-sdk")
        }
    }

    #[test]
    fn warns_when_sdk_pulls_mini_alloc_by_default() {
        let tree = [sdk(true, &[]), crate_info("wee_alloc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("wee_alloc"), Some(&tree));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result.message.contains("may conflict"));
        assert!(result.message.contains("registration was not inspected"));
    }

    #[test]
    fn warns_when_sdk_opts_into_mini_alloc_explicitly() {
        let tree = [sdk(false, &["mini-alloc"]), crate_info("talc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("talc"), Some(&tree));
        assert_eq!(result.severity, Severity::Warning);
    }

    #[test]
    fn warns_when_sdk_opted_out_of_mini_alloc() {
        let tree = [sdk(false, &["export-abi"]), crate_info("wee_alloc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("wee_alloc"), Some(&tree));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result.message.contains("mini-alloc disabled"));
        assert!(!result.message.contains("not found"));
    }

    #[test]
    fn warns_when_tree_has_no_stylus_sdk() {
        let tree = [crate_info("serde"), crate_info("lol_alloc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("lol_alloc"), Some(&tree));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result
            .message
            .contains("not found in the inspected dependencies"));
    }

    #[test]
    fn warns_when_no_tree_was_inspected() {
        let result = AllocatorCheck.run(&crate_info("wee_alloc"));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result.message.contains("no dependency tree was inspected"));
    }

    #[test]
    fn does_not_flag_standard_allocator_dependencies() {
        let tree = [sdk(true, &[])];
        for name in ["dlmalloc", "mini-alloc", "stylus-sdk"] {
            let result = AllocatorCheck.check_in_tree(&crate_info(name), Some(&tree));
            assert_eq!(result.severity, Severity::Pass, "{name} should pass");
            assert!(result
                .message
                .contains("is not flagged as a conflicting allocator"));
        }
    }

    #[test]
    fn passes_unrelated_crate() {
        let result = AllocatorCheck.check_in_tree(&crate_info("tiny-keccak"), None);
        assert_eq!(result.severity, Severity::Pass);
    }
}
