use std::path::Path;

use crate::types::CrateInfo;

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("failed to read Cargo.toml at {path}: {source}")]
    ReadError {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse Cargo.toml: {0}")]
    ParseError(#[from] cargo_toml::Error),
}

/// Parses a Cargo.toml file and extracts all dependencies as `CrateInfo` entries.
///
/// Reads both `[dependencies]` and `[dev-dependencies]`, but marks dev-dependencies
/// separately so checks can optionally skip them (dev-deps don't end up in the
/// final WASM binary).
pub fn parse_dependencies(manifest_path: &Path) -> Result<Vec<CrateInfo>, ManifestError> {
    let content = std::fs::read_to_string(manifest_path).map_err(|e| ManifestError::ReadError {
        path: manifest_path.display().to_string(),
        source: e,
    })?;

    let manifest = cargo_toml::Manifest::from_str(&content)?;
    let mut deps = Vec::new();

    for (name, dep) in &manifest.dependencies {
        let (version, features, default_features) = extract_dep_details(dep);
        deps.push(CrateInfo {
            name: name.clone(),
            version,
            features,
            default_features,
        });
    }

    Ok(deps)
}

fn extract_dep_details(dep: &cargo_toml::Dependency) -> (Option<String>, Vec<String>, bool) {
    match dep {
        cargo_toml::Dependency::Simple(version) => (Some(version.clone()), vec![], true),
        cargo_toml::Dependency::Inherited(detail) => {
            let features = detail.features.clone();
            (None, features, true)
        }
        cargo_toml::Dependency::Detailed(detail) => {
            let version = detail.version.clone();
            let features = detail.features.clone();
            let default_features = detail.default_features;
            (version, features, default_features)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_manifest(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn parses_simple_dependencies() {
        let file = write_manifest(
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = "1.0"
tiny-keccak = "2.0"
"#,
        );

        let deps = parse_dependencies(file.path()).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "serde"));
        assert!(deps.iter().any(|d| d.name == "tiny-keccak"));
    }

    #[test]
    fn parses_detailed_dependencies() {
        let file = write_manifest(
            r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"], default-features = false }
"#,
        );

        let deps = parse_dependencies(file.path()).unwrap();
        assert_eq!(deps.len(), 1);
        let serde = &deps[0];
        assert_eq!(serde.name, "serde");
        assert!(!serde.default_features);
        assert!(serde.features.contains(&"derive".to_string()));
    }

    #[test]
    fn returns_error_for_missing_file() {
        let result = parse_dependencies(Path::new("/nonexistent/Cargo.toml"));
        assert!(result.is_err());
    }
}
