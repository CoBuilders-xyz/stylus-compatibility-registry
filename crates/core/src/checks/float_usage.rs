use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};
use flate2::read::GzDecoder;
use regex::Regex;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tar::Archive;

/// Checks if a crate uses floating-point operations.
///
/// Stylus WASM contracts must avoid `f32`/`f64` — the Arbitrum WASM runtime disallows
/// floating-point opcodes for determinism. Crates that perform float arithmetic will cause
/// activation failure on-chain.
///
/// Uses a fast-path blocklist for known float crates, then downloads crate source from
/// crates.io and scans `.rs` files for `f32`, `f64`, and float literals.
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum FloatUsageKind {
    Direct,
    Conditional,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FloatFinding {
    file: String,
    line: usize,
    kind: FloatUsageKind,
    snippet: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct FloatScanResult {
    direct: Vec<FloatFinding>,
    conditional: Vec<FloatFinding>,
}

impl FloatScanResult {
    fn has_any(&self) -> bool {
        !self.direct.is_empty() || !self.conditional.is_empty()
    }
}

fn float_type_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:f32|f64)\b").expect("valid float type regex"))
}

fn float_literal_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d+\.\d+(?:f32|f64)?\b").expect("valid float literal regex"))
}

fn strip_line_comment(line: &str) -> String {
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '/' if chars.peek().map(|(_, n)| *n) == Some('/') => {
                return line[..idx].to_string();
            }
            _ => {}
        }
    }

    line.to_string()
}

fn float_matches_in_line(line: &str) -> usize {
    let line = strip_line_comment(line);
    float_type_regex().find_iter(&line).count() + float_literal_regex().find_iter(&line).count()
}

fn line_has_float_usage(line: &str) -> bool {
    float_matches_in_line(line) > 0
}

fn is_cfg_attribute_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#[cfg") || trimmed.starts_with("#[cfg_attr")
}

fn is_previous_line_cfg(line_idx: usize, lines: &[&str]) -> bool {
    for i in (0..line_idx).rev() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }
        if is_cfg_attribute_line(lines[i]) {
            return true;
        }
        if trimmed.starts_with('#') {
            continue;
        }
        return false;
    }
    false
}

fn is_cfg_guarded(brace_depth: usize, cfg_guarded_depths: &[usize]) -> bool {
    cfg_guarded_depths.iter().any(|&depth| brace_depth >= depth)
}

fn scan_rust_source(source: &str, file_label: &str) -> FloatScanResult {
    let lines: Vec<&str> = source.lines().collect();
    let mut result = FloatScanResult::default();

    let mut brace_depth: usize = 0;
    let mut cfg_guarded_depths: Vec<usize> = Vec::new();
    let mut pending_cfg = false;

    for (line_num, line) in lines.iter().enumerate() {
        if is_cfg_attribute_line(line) {
            pending_cfg = true;
        }

        if line_has_float_usage(line) {
            let is_conditional = pending_cfg
                || is_previous_line_cfg(line_num, &lines)
                || is_cfg_guarded(brace_depth, &cfg_guarded_depths);

            let match_count = float_matches_in_line(line);
            for _ in 0..match_count {
                let finding = FloatFinding {
                    file: file_label.to_string(),
                    line: line_num + 1,
                    kind: if is_conditional {
                        FloatUsageKind::Conditional
                    } else {
                        FloatUsageKind::Direct
                    },
                    snippet: line.trim().to_string(),
                };

                match finding.kind {
                    FloatUsageKind::Direct => result.direct.push(finding),
                    FloatUsageKind::Conditional => result.conditional.push(finding),
                }
            }
        }

        let scan_line = strip_line_comment(line);
        for ch in scan_line.chars() {
            match ch {
                '{' => {
                    if pending_cfg {
                        cfg_guarded_depths.push(brace_depth + 1);
                    }
                    brace_depth += 1;
                    pending_cfg = false;
                }
                '}' => {
                    if cfg_guarded_depths.last() == Some(&brace_depth) {
                        cfg_guarded_depths.pop();
                    }
                    brace_depth = brace_depth.saturating_sub(1);
                }
                _ => {}
            }
        }

        let trimmed = line.trim();
        if pending_cfg
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.ends_with('{')
            && !trimmed.ends_with(',')
        {
            pending_cfg = false;
        }
    }

    result
}

fn visit_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn scan_crate_directory(crate_dir: &Path) -> FloatScanResult {
    let mut combined = FloatScanResult::default();
    let mut rs_files = Vec::new();
    visit_rs_files(crate_dir, &mut rs_files);

    for path in rs_files {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let label = path
            .strip_prefix(crate_dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        let partial = scan_rust_source(&source, &label);
        combined.direct.extend(partial.direct);
        combined.conditional.extend(partial.conditional);
    }

    combined
}

fn download_and_extract_crate(name: &str, version: &str) -> Result<tempfile::TempDir, String> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}/download",
        name, version
    );

    let response = ureq::get(&url)
        .call()
        .map_err(|err| format!("failed to download crate source: {err}"))?;

    let mut bytes = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|err| format!("failed to read crate tarball: {err}"))?;

    let temp_dir =
        tempfile::tempdir().map_err(|err| format!("failed to create temp dir: {err}"))?;
    let decoder = GzDecoder::new(bytes.as_slice());
    let mut archive = Archive::new(decoder);
    archive
        .unpack(temp_dir.path())
        .map_err(|err| format!("failed to extract crate tarball: {err}"))?;

    Ok(temp_dir)
}

fn crate_source_root(extract_dir: &Path, name: &str, version: &str) -> PathBuf {
    let nested = extract_dir.join(format!("{name}-{version}"));
    if nested.is_dir() {
        nested
    } else {
        extract_dir.to_path_buf()
    }
}

fn analyze_crate_source(name: &str, version: &str) -> Result<FloatScanResult, String> {
    let temp_dir = download_and_extract_crate(name, version)?;
    let source_root = crate_source_root(temp_dir.path(), name, version);
    Ok(scan_crate_directory(&source_root))
}

fn format_scan_warning(crate_name: &str, scan: &FloatScanResult) -> String {
    let direct = scan.direct.len();
    let conditional = scan.conditional.len();

    if direct > 0 && conditional > 0 {
        format!(
            "`{crate_name}` uses floating-point operations in source ({direct} direct, {conditional} cfg-guarded) — Stylus disallows f32/f64 for determinism"
        )
    } else if direct > 0 {
        format!(
            "`{crate_name}` uses floating-point operations in source ({direct} direct) — Stylus disallows f32/f64 for determinism"
        )
    } else {
        format!(
            "`{crate_name}` uses floating-point operations in source ({conditional} cfg-guarded) — Stylus disallows f32/f64 for determinism"
        )
    }
}

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

        if let Some(version) = &crate_info.version {
            match analyze_crate_source(&crate_info.name, version) {
                Ok(scan) if scan.has_any() => {
                    return CheckResult::warning(
                        self.name(),
                        format_scan_warning(&crate_info.name, &scan),
                    );
                }
                Ok(_) => {}
                Err(_) => {
                    return CheckResult::pass(
                        self.name(),
                        format!(
                            "`{}` source could not be analyzed — assuming no float usage",
                            crate_info.name
                        ),
                    );
                }
            }
        }

        CheckResult::pass(
            self.name(),
            format!(
                "`{}` has no detected floating-point usage in source",
                crate_info.name
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crate_info(name: &str, version: Option<&str>) -> CrateInfo {
        CrateInfo {
            name: name.to_string(),
            version: version.map(str::to_string),
            features: vec![],
            default_features: true,
        }
    }

    #[test]
    fn detects_direct_f32_usage() {
        let source = "fn compute() -> f32 { 1.0 }";
        let scan = scan_rust_source(source, "lib.rs");
        assert_eq!(scan.direct.len(), 2);
        assert!(scan.conditional.is_empty());
    }

    #[test]
    fn detects_direct_f64_and_literals() {
        let source = "let x: f64 = 3.14;\nlet y = 0.5f32;";
        let scan = scan_rust_source(source, "lib.rs");
        assert_eq!(scan.direct.len(), 3);
        assert!(scan.conditional.is_empty());
    }

    #[test]
    fn detects_cfg_guarded_float_on_next_line() {
        let source = "#[cfg(feature = \"float\")]\nlet x: f32 = 1.0;";
        let scan = scan_rust_source(source, "lib.rs");
        assert!(scan.direct.is_empty());
        assert_eq!(scan.conditional.len(), 2);
    }

    #[test]
    fn detects_cfg_guarded_float_in_block() {
        let source =
            "#[cfg(test)]\nmod tests {\n    fn float_fn() {\n        let x: f32 = 1.0;\n    }\n}";
        let scan = scan_rust_source(source, "lib.rs");
        assert!(scan.direct.is_empty());
        assert!(!scan.conditional.is_empty());
    }

    #[test]
    fn ignores_floats_in_line_comments() {
        let source = "let x = 0; // uses f32 internally";
        let scan = scan_rust_source(source, "lib.rs");
        assert!(!scan.has_any());
    }

    #[test]
    fn warns_on_float_crate() {
        let result = FloatUsageCheck.run(&crate_info("nalgebra", Some("0.33.0")));
        assert_eq!(result.severity, crate::types::Severity::Warning);
    }

    #[test]
    fn passes_clean_crate_without_download() {
        let result = FloatUsageCheck.run(&crate_info("tiny-keccak", None));
        assert_eq!(result.severity, crate::types::Severity::Pass);
    }

    #[test]
    fn falls_back_to_pass_when_download_fails() {
        let result = FloatUsageCheck.run(&crate_info(
            "this-crate-definitely-does-not-exist-xyz",
            Some("0.0.0"),
        ));
        assert_eq!(result.severity, crate::types::Severity::Pass);
        assert!(result.message.contains("could not be analyzed"));
    }

    #[test]
    fn download_and_extract_invalid_crate_returns_error() {
        let result =
            download_and_extract_crate("this-crate-definitely-does-not-exist-xyz", "0.0.0");
        assert!(result.is_err());
    }
}
