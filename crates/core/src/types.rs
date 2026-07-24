use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: String,
    pub version: Option<String>,
    pub features: Vec<String>,
    pub default_features: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Pass,
    Warning,
    Error,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Pass => write!(f, "PASS"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "FAIL"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub severity: Severity,
    pub message: String,
}

impl CheckResult {
    pub fn pass(check_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            severity: Severity::Pass,
            message: message.into(),
        }
    }

    pub fn warning(check_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            severity: Severity::Warning,
            message: message.into(),
        }
    }

    pub fn error(check_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            check_name: check_name.into(),
            severity: Severity::Error,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateReport {
    pub crate_info: CrateInfo,
    pub results: Vec<CheckResult>,
    pub score: CompatibilityScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityScore {
    pub value: u8,
    pub max: u8,
    pub label: String,
}

impl CompatibilityScore {
    pub fn new(value: u8) -> Self {
        let label = match value {
            90..=100 => "Excellent".to_string(),
            70..=89 => "Good".to_string(),
            50..=69 => "Needs Review".to_string(),
            _ => "Incompatible".to_string(),
        };
        Self {
            value,
            max: 100,
            label,
        }
    }
}

impl fmt::Display for CompatibilityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{} ({})", self.value, self.max, self.label)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReport {
    pub manifest_path: String,
    pub crate_reports: Vec<CrateReport>,
    pub overall_score: CompatibilityScore,
    pub error_count: usize,
    pub warning_count: usize,
}
