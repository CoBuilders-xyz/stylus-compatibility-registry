use std::collections::{HashSet, VecDeque};
use std::path::Path;

use cargo_metadata::{DependencyKind, MetadataCommand, NodeDep, PackageId};

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
    #[error("failed to resolve dependency metadata: {0}")]
    MetadataError(#[from] cargo_metadata::Error),
    #[error("no root package found in Cargo metadata for {path}")]
    NoRootPackage { path: String },
    #[error("dependency graph missing from Cargo metadata for {path}")]
    MissingResolve { path: String },
}

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
            is_transitive: false,
        });
    }
    Ok(deps)
}

pub fn resolve_full_tree(manifest_path: &Path) -> Result<Vec<CrateInfo>, ManifestError> {
    let metadata = MetadataCommand::new()
        .manifest_path(manifest_path)
        .other_options(vec![
            "--filter-platform".to_string(),
            "wasm32-unknown-unknown".to_string(),
        ])
        .exec()?;
    let root = metadata
        .root_package()
        .ok_or_else(|| ManifestError::NoRootPackage {
            path: manifest_path.display().to_string(),
        })?;
    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| ManifestError::MissingResolve {
            path: manifest_path.display().to_string(),
        })?;
    let root_node = resolve
        .nodes
        .iter()
        .find(|node| node.id == root.id)
        .ok_or_else(|| ManifestError::MissingResolve {
            path: manifest_path.display().to_string(),
        })?;
    let direct_package_ids: HashSet<PackageId> = root_node
        .deps
        .iter()
        .filter(|dep| is_normal_dep(dep))
        .map(|dep| dep.pkg.clone())
        .collect();
    let mut package_ids = collect_reachable_packages(root_node, &resolve.nodes);
    package_ids.remove(&root.id);
    let mut deps: Vec<CrateInfo> = package_ids
        .into_iter()
        .filter_map(|package_id| {
            let package = metadata.packages.iter().find(|pkg| pkg.id == package_id)?;
            let node = resolve.nodes.iter().find(|node| node.id == package_id);
            Some(CrateInfo {
                name: package.name.clone(),
                version: Some(package.version.to_string()),
                features: node.map(|n| n.features.clone()).unwrap_or_default(),
                default_features: node
                    .is_some_and(|node| node.features.iter().any(|feature| feature == "default")),
                is_transitive: !direct_package_ids.contains(&package.id),
            })
        })
        .collect();
    deps.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
    Ok(deps)
}

fn is_normal_dep(dep: &NodeDep) -> bool {
    dep.dep_kinds
        .iter()
        .any(|kind| kind.kind == DependencyKind::Normal)
}

fn collect_reachable_packages(
    root_node: &cargo_metadata::Node,
    all_nodes: &[cargo_metadata::Node],
) -> HashSet<PackageId> {
    let nodes_by_id: std::collections::HashMap<&PackageId, &cargo_metadata::Node> =
        all_nodes.iter().map(|node| (&node.id, node)).collect();
    let mut reachable = HashSet::new();
    let mut queue: VecDeque<PackageId> = root_node
        .deps
        .iter()
        .filter(|dep| is_normal_dep(dep))
        .map(|dep| dep.pkg.clone())
        .collect();
    while let Some(package_id) = queue.pop_front() {
        if !reachable.insert(package_id.clone()) {
            continue;
        }
        if let Some(node) = nodes_by_id.get(&package_id) {
            for dep in &node.deps {
                if is_normal_dep(dep) {
                    queue.push_back(dep.pkg.clone());
                }
            }
        }
    }
    reachable
}

fn extract_dep_details(dep: &cargo_toml::Dependency) -> (Option<String>, Vec<String>, bool) {
    match dep {
        cargo_toml::Dependency::Simple(version) => (Some(version.clone()), vec![], true),
        cargo_toml::Dependency::Inherited(detail) => (None, detail.features.clone(), true),
        cargo_toml::Dependency::Detailed(detail) => (
            detail.version.clone(),
            detail.features.clone(),
            detail.default_features,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    fn write_manifest(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
    }

    #[test]
    fn parses_simple_dependencies() {
        let file = write_manifest("[package]\nname=\"test-project\"\nversion=\"0.1.0\"\nedition=\"2021\"\n\n[dependencies]\nserde=\"1.0\"\ntiny-keccak=\"2.0\"\n");
        let deps = parse_dependencies(file.path()).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().all(|d| !d.is_transitive));
    }

    #[test]
    fn parses_detailed_dependencies() {
        let file = write_manifest("[package]\nname=\"test-project\"\nversion=\"0.1.0\"\nedition=\"2021\"\n\n[dependencies]\nserde={version=\"1.0\",features=[\"derive\"],default-features=false}\n");
        let deps = parse_dependencies(file.path()).unwrap();
        assert_eq!(deps.len(), 1);
        assert!(!deps[0].default_features);
        assert!(!deps[0].is_transitive);
    }

    #[test]
    fn returns_error_for_missing_file() {
        assert!(parse_dependencies(Path::new("/nonexistent/Cargo.toml")).is_err());
    }

    #[test]
    fn resolve_full_tree_includes_more_deps_than_parse() {
        let manifest = fixtures_dir().join("transitive-test/Cargo.toml");
        let direct = parse_dependencies(&manifest).unwrap();
        let full = resolve_full_tree(&manifest).unwrap();
        assert!(full.len() > direct.len());
    }

    #[test]
    fn resolve_full_tree_marks_transitive_deps() {
        let manifest = fixtures_dir().join("transitive-test/Cargo.toml");
        let full = resolve_full_tree(&manifest).unwrap();
        let hex = full.iter().find(|d| d.name == "hex").unwrap();
        assert!(!hex.is_transitive);
        assert!(!hex.default_features);
        assert!(
            full.iter()
                .find(|d| d.name == "libc")
                .unwrap()
                .is_transitive
        );
        assert!(full.iter().all(|dep| dep.name != "mio"));
        assert!(full.iter().all(|dep| dep.name != "winapi"));
    }
}
