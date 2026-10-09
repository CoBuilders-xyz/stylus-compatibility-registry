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
                    format!("`{}` requires std{}", crate_info.name, entry.note_suffix()),
                );
            }
            if entry.requires_no_default_features && crate_info.default_features {
                return CheckResult::warning(
                    self.name(),
                    format!(
                        "`{}` is no_std compatible only with default-features = false{}",
                        crate_info.name,
                        entry.note_suffix()
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
            requires_no_default_features: false,
            max_version: None,
            alternative: Some("serde-json-core".to_string()),
            notes: Some("Uses std::io for formatting".to_string()),
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.severity, crate::types::Severity::Error);
        assert_eq!(
            result.message,
            "`serde_json` requires std. Registry note: Uses std::io for formatting"
        );
    }

    #[test]
    fn omits_the_note_when_the_entry_has_none() {
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
            requires_no_default_features: false,
            max_version: None,
            alternative: None,
            notes: None,
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.message, "`serde_json` requires std");
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
            requires_no_default_features: false,
            max_version: None,
            alternative: None,
            notes: Some("Serialization framework".to_string()),
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }

    #[test]
    fn warns_when_the_entry_needs_default_features_off() {
        let info = CrateInfo {
            name: "hex".to_string(),
            version: Some("0.4.3".to_string()),
            features: vec![],
            default_features: true,
            is_transitive: false,
        };
        let entry = KnownCrateEntry {
            name: "hex".to_string(),
            requires_std: false,
            has_float: false,
            has_async: false,
            requires_no_default_features: true,
            max_version: None,
            alternative: None,
            notes: Some("Hex encoding/decoding".to_string()),
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.severity, crate::types::Severity::Warning);
        assert_eq!(
            result.message,
            "`hex` is no_std compatible only with default-features = false. Registry note: Hex encoding/decoding"
        );
    }

    #[test]
    fn passes_when_default_features_are_already_off() {
        let info = CrateInfo {
            name: "hex".to_string(),
            version: Some("0.4.3".to_string()),
            features: vec![],
            default_features: false,
            is_transitive: false,
        };
        let entry = KnownCrateEntry {
            name: "hex".to_string(),
            requires_std: false,
            has_float: false,
            has_async: false,
            requires_no_default_features: true,
            max_version: None,
            alternative: None,
            notes: None,
        };
        let result = NoStdCheck.check_against_registry(&info, Some(&entry));
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }
}
