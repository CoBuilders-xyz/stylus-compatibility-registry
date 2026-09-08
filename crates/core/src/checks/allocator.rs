use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

/// Checks for crates that register their own `#[global_allocator]`.
///
/// `stylus-sdk` ships `default = ["mini-alloc"]` and registers a global allocator on
/// wasm32. Rust allows one per binary, so a second one is a compile error rather than
/// anything the Stylus VM rejects at activation.
///
/// Only crate names are matched, and a name in the tree is not proof that someone wrote
/// the attribute. That is why the verdict drops to a warning when `stylus-sdk` is not
/// there to collide with.
pub struct AllocatorCheck;

/// Allocators that build for wasm32 and fail only on the duplicate registration.
///
/// `dlmalloc` is deliberately absent: it is what `wasm32-unknown-unknown` uses by
/// default through std, so it sits in every normal tree. `mini-alloc` is absent for the
/// same reason, it is the one the SDK already uses.
const CONFLICTING_ALLOCATORS: &[&str] = &["wee_alloc", "talc", "lol_alloc"];

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
        if !CONFLICTING_ALLOCATORS.contains(&crate_info.name.as_str()) {
            return CheckResult::pass(
                self.name(),
                format!("`{}` does not register a global allocator", crate_info.name),
            );
        }

        match deps {
            Some(deps) if sdk_registers_mini_alloc(deps) => CheckResult::error(
                self.name(),
                format!(
                    "`{}` registers a global allocator and stylus-sdk already registers \
                     mini-alloc; Rust allows only one per binary",
                    crate_info.name
                ),
            ),
            Some(_) => CheckResult::warning(
                self.name(),
                format!(
                    "`{}` registers a global allocator; stylus-sdk was not found in the \
                     dependency tree",
                    crate_info.name
                ),
            ),
            None => CheckResult::warning(
                self.name(),
                format!(
                    "`{}` registers a global allocator; no dependency tree was inspected, \
                     so a conflict with stylus-sdk's mini-alloc is possible",
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
    fn errors_when_sdk_pulls_mini_alloc_by_default() {
        let tree = [sdk(true, &[]), crate_info("wee_alloc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("wee_alloc"), Some(&tree));
        assert_eq!(result.severity, Severity::Error);
        assert!(result.message.contains("only one per binary"));
    }

    #[test]
    fn errors_when_sdk_opts_into_mini_alloc_explicitly() {
        let tree = [sdk(false, &["mini-alloc"]), crate_info("talc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("talc"), Some(&tree));
        assert_eq!(result.severity, Severity::Error);
    }

    #[test]
    fn warns_when_sdk_opted_out_of_mini_alloc() {
        let tree = [sdk(false, &["export-abi"]), crate_info("wee_alloc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("wee_alloc"), Some(&tree));
        assert_eq!(result.severity, Severity::Warning);
    }

    #[test]
    fn warns_when_tree_has_no_stylus_sdk() {
        let tree = [crate_info("serde"), crate_info("lol_alloc")];
        let result = AllocatorCheck.check_in_tree(&crate_info("lol_alloc"), Some(&tree));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result.message.contains("not found in the dependency tree"));
    }

    #[test]
    fn warns_when_no_tree_was_inspected() {
        let result = AllocatorCheck.run(&crate_info("wee_alloc"));
        assert_eq!(result.severity, Severity::Warning);
        assert!(result.message.contains("no dependency tree was inspected"));
    }

    #[test]
    fn passes_allocators_that_do_not_conflict() {
        let tree = [sdk(true, &[])];
        for name in ["dlmalloc", "mini-alloc"] {
            let result = AllocatorCheck.check_in_tree(&crate_info(name), Some(&tree));
            assert_eq!(result.severity, Severity::Pass, "{name} should pass");
        }
    }

    #[test]
    fn passes_unrelated_crate() {
        let result = AllocatorCheck.check_in_tree(&crate_info("tiny-keccak"), None);
        assert_eq!(result.severity, Severity::Pass);
    }
}
