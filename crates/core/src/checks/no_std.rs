use crate::checks::CrateCheck;
use crate::registry::KnownCrateEntry;
use crate::types::{CheckResult, CrateInfo};

pub struct NoStdCheck;

/// Crates known to require std with no `no_std` alternative configuration.
const KNOWN_STD_CRATES: &[&str] = &[
    "std",
    "tokio",
    "reqwest",
    "hyper",
    "actix-web",
    "rocket",
    "diesel",
    "sqlx",
    "rusqlite",
    "tungstenite",
    "native-tls",
    "openssl",
];

impl CrateCheck for NoStdCheck {
    fn name(&self) -> &str {
        "no_std"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        if KNOWN_STD_CRATES.contains(&crate_info.name.as_str()) {
            return CheckResult::error(
                self.name(),
                format!(
                    "`{}` requires std and cannot be used in Stylus contracts",
                    crate_info.name
                ),
            );
        }

        CheckResult::pass(
            self.name(),
            format!("`{}` is not in the known-std blocklist", crate_info.name),
        )
    }
}

impl NoStdCheck {
    pub fn check_against_registry(
        &self,
        crate_info: &CrateInfo,
        entry: Option<&KnownCrateEntry>,
    ) -> CheckResult {
        if let Some(entry) = entry {
            if entry.requires_std {
                return CheckResult::error(
                    self.name(),
                    format!(
                        "`{}` requires std: {}",
                        crate_info.name,
                        entry.notes.as_deref().unwrap_or("no details")
                    ),
                );
            }
            return CheckResult::pass(
                self.name(),
                format!("`{}` is verified no_std compatible", crate_info.name),
            );
        }

        // Not in registry — fall back to the blocklist check
        self.run(crate_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_std_crate() {
        let info = CrateInfo {
            name: "tokio".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = NoStdCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Error);
        assert!(result.message.contains("requires std"));
    }

    #[test]
    fn passes_unknown_crate() {
        let info = CrateInfo {
            name: "tiny-keccak".to_string(),
            version: Some("2.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let result = NoStdCheck.run(&info);
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }

    #[test]
    fn detects_std_via_registry() {
        let info = CrateInfo {
            name: "serde_json".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let entry = KnownCrateEntry {
            name: "serde_json".to_string(),
            requires_std: true,
            has_float: false,
            has_async: false,
            max_version: None,
            alternative: Some("serde-json-core".to_string()),
            notes: Some("Uses std::io for formatting".to_string()),
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.severity, crate::types::Severity::Error);
    }

    #[test]
    fn passes_via_registry_when_compatible() {
        let info = CrateInfo {
            name: "serde".to_string(),
            version: Some("1.0.0".to_string()),
            features: vec![],
            default_features: false,
            is_transitive: false,
        };
        let entry = KnownCrateEntry {
            name: "serde".to_string(),
            requires_std: false,
            has_float: false,
            has_async: false,
            max_version: None,
            alternative: None,
            notes: Some("Use default-features = false".to_string()),
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }
}
