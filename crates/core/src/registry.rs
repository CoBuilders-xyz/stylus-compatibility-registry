use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("failed to read registry file at {path}: {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse registry TOML: {0}")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownCrateEntry {
    pub name: String,
    pub requires_std: bool,
    pub has_float: bool,
    pub has_async: bool,
    pub max_version: Option<String>,
    pub alternative: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    #[serde(rename = "crate")]
    crates: Vec<KnownCrateEntry>,
}

#[derive(Debug, Default)]
pub struct KnownCratesRegistry {
    entries: HashMap<String, KnownCrateEntry>,
}

impl KnownCratesRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads crate entries from a TOML file and merges them into this registry.
    pub fn load_file(&mut self, path: &Path) -> Result<usize, RegistryError> {
        let content = std::fs::read_to_string(path).map_err(|e| RegistryError::ReadError {
            path: path.display().to_string(),
            source: e,
        })?;

        let registry_file: RegistryFile = toml::from_str(&content)?;
        let count = registry_file.crates.len();

        for entry in registry_file.crates {
            self.entries.insert(entry.name.clone(), entry);
        }

        Ok(count)
    }

    /// Loads both known-compatible and known-incompatible TOML files from a data directory.
    pub fn load_data_dir(&mut self, data_dir: &Path) -> Result<usize, RegistryError> {
        let mut total = 0;

        let compatible_path = data_dir.join("known-compatible.toml");
        if compatible_path.exists() {
            total += self.load_file(&compatible_path)?;
        }

        let incompatible_path = data_dir.join("known-incompatible.toml");
        if incompatible_path.exists() {
            total += self.load_file(&incompatible_path)?;
        }

        Ok(total)
    }

    pub fn lookup(&self, crate_name: &str) -> Option<&KnownCrateEntry> {
        self.entries.get(crate_name)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn loads_registry_from_toml() {
        let mut file = NamedTempFile::new().unwrap();
        write!(
            file,
            r#"
[[crate]]
name = "tiny-keccak"
requires_std = false
has_float = false
has_async = false
notes = "Pure Rust keccak256, perfect for Stylus"

[[crate]]
name = "serde_json"
requires_std = true
has_float = false
has_async = false
alternative = "serde-json-core"
notes = "Uses std::io for formatting"
"#
        )
        .unwrap();

        let mut registry = KnownCratesRegistry::new();
        let count = registry.load_file(file.path()).unwrap();

        assert_eq!(count, 2);
        assert_eq!(registry.len(), 2);

        let tk = registry.lookup("tiny-keccak").unwrap();
        assert!(!tk.requires_std);

        let sj = registry.lookup("serde_json").unwrap();
        assert!(sj.requires_std);
        assert_eq!(sj.alternative.as_deref(), Some("serde-json-core"));
    }

    #[test]
    fn returns_none_for_unknown_crate() {
        let registry = KnownCratesRegistry::new();
        assert!(registry.lookup("unknown-crate-xyz").is_none());
    }
}
