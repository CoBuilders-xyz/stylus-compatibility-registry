use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};
use flate2::read::GzDecoder;
use regex::Regex;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;
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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct FloatScanResult {
    direct: usize,
    conditional: usize,
}

impl FloatScanResult {
    fn has_any(&self) -> bool {
        self.direct > 0 || self.conditional > 0
    }
}

const MAX_CRATE_TARBALL_BYTES: u64 = 50 * 1024 * 1024;
const MAX_EXTRACTED_BYTES: u64 = 200 * 1024 * 1024;
const MAX_ARCHIVE_ENTRIES: usize = 100_000;

fn float_type_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b(?:f32|f64)\b").expect("valid float type regex"))
}

fn float_literal_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"\b(?:\d[\d_]*\.[\d_]*(?:[eE][+-]?\d[\d_]*)?(?:_?(?:f32|f64))?|\d[\d_]*(?:[eE][+-]?\d[\d_]*)(?:_?(?:f32|f64))?|\d[\d_]*_?(?:f32|f64))\b",
        )
        .expect("valid float literal regex")
    })
}

fn trailing_dot_float_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\b\d[\d_]*\.(?:[^\w.]|$)").expect("valid float regex"))
}

#[derive(Debug, Default)]
struct LexState {
    block_comment_depth: usize,
    raw_string_hashes: Option<usize>,
    in_string: bool,
    escaped: bool,
}

fn raw_string_start(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut index = start;
    if bytes.get(index) == Some(&b'b') {
        index += 1;
    }
    if bytes.get(index) != Some(&b'r') {
        return None;
    }
    index += 1;

    let hash_start = index;
    while bytes.get(index) == Some(&b'#') {
        index += 1;
    }
    (bytes.get(index) == Some(&b'"')).then_some((index - hash_start, index + 1))
}

fn raw_string_end(bytes: &[u8], quote: usize, hashes: usize) -> bool {
    bytes.get(quote) == Some(&b'"')
        && (0..hashes).all(|offset| bytes.get(quote + 1 + offset) == Some(&b'#'))
}

fn sanitize_rust_line(line: &str, state: &mut LexState) -> String {
    let bytes = line.as_bytes();
    let mut sanitized = bytes.to_vec();
    let mut index = 0;

    while index < bytes.len() {
        if let Some(hashes) = state.raw_string_hashes {
            sanitized[index] = b' ';
            if raw_string_end(bytes, index, hashes) {
                for offset in 1..=hashes {
                    sanitized[index + offset] = b' ';
                }
                index += hashes + 1;
                state.raw_string_hashes = None;
            } else {
                index += 1;
            }
            continue;
        }

        if state.block_comment_depth > 0 {
            sanitized[index] = b' ';
            if bytes.get(index..index + 2) == Some(b"/*") {
                sanitized[index + 1] = b' ';
                state.block_comment_depth += 1;
                index += 2;
            } else if bytes.get(index..index + 2) == Some(b"*/") {
                sanitized[index + 1] = b' ';
                state.block_comment_depth -= 1;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }

        if state.in_string {
            sanitized[index] = b' ';
            if state.escaped {
                state.escaped = false;
            } else if bytes[index] == b'\\' {
                state.escaped = true;
            } else if bytes[index] == b'"' {
                state.in_string = false;
            }
            index += 1;
            continue;
        }

        if bytes.get(index..index + 2) == Some(b"//") {
            sanitized[index..].fill(b' ');
            break;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            sanitized[index..index + 2].fill(b' ');
            state.block_comment_depth = 1;
            index += 2;
            continue;
        }
        if let Some((hashes, content_start)) = raw_string_start(bytes, index) {
            sanitized[index..content_start].fill(b' ');
            state.raw_string_hashes = Some(hashes);
            index = content_start;
            continue;
        }
        if bytes[index] == b'"' {
            sanitized[index] = b' ';
            state.in_string = true;
        }
        index += 1;
    }

    String::from_utf8(sanitized).expect("sanitizing preserves UTF-8")
}

fn float_matches_in_line(line: &str) -> usize {
    float_type_regex().find_iter(line).count()
        + float_literal_regex().find_iter(line).count()
        + trailing_dot_float_regex().find_iter(line).count()
}

fn is_cfg_attribute_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#[cfg") || trimmed.starts_with("#[cfg_attr")
}

fn is_previous_line_cfg(line_idx: usize, lines: &[String]) -> bool {
    for i in (0..line_idx).rev() {
        let trimmed = lines[i].trim();
        if trimmed.is_empty() || trimmed.starts_with("///") || trimmed.starts_with("//!") {
            continue;
        }
        if is_cfg_attribute_line(&lines[i]) {
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

fn scan_rust_source(source: &str) -> FloatScanResult {
    let mut lex_state = LexState::default();
    let lines: Vec<String> = source
        .lines()
        .map(|line| sanitize_rust_line(line, &mut lex_state))
        .collect();
    let mut result = FloatScanResult::default();

    let mut brace_depth: usize = 0;
    let mut cfg_guarded_depths: Vec<usize> = Vec::new();
    let mut pending_cfg = false;

    for (line_num, line) in lines.iter().enumerate() {
        if is_cfg_attribute_line(line) {
            pending_cfg = true;
        }

        let match_count = float_matches_in_line(line);
        if match_count > 0 {
            let is_conditional = pending_cfg
                || is_previous_line_cfg(line_num, &lines)
                || is_cfg_guarded(brace_depth, &cfg_guarded_depths);

            if is_conditional {
                result.conditional += match_count;
            } else {
                result.direct += match_count;
            }
        }

        for ch in line.chars() {
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
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            visit_rs_files(&path, files);
        } else if metadata.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
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
        let partial = scan_rust_source(&source);
        combined.direct += partial.direct;
        combined.conditional += partial.conditional;
    }

    combined
}

fn download_and_extract_crate(name: &str, version: &str) -> Result<tempfile::TempDir, String> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}/download",
        name, version
    );

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(30))
        .timeout(Duration::from_secs(60))
        .build();
    let response = agent
        .get(&url)
        .call()
        .map_err(|err| format!("failed to download crate source: {err}"))?;

    if response
        .header("Content-Length")
        .and_then(|value| value.parse::<u64>().ok())
        .is_some_and(|length| length > MAX_CRATE_TARBALL_BYTES)
    {
        return Err("failed to read crate tarball: compressed archive is too large".to_string());
    }

    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_CRATE_TARBALL_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("failed to read crate tarball: {err}"))?;
    if bytes.len() as u64 > MAX_CRATE_TARBALL_BYTES {
        return Err("failed to read crate tarball: compressed archive is too large".to_string());
    }

    extract_crate_tarball(bytes.as_slice())
}

fn extract_crate_tarball(reader: impl Read) -> Result<tempfile::TempDir, String> {
    let temp_dir =
        tempfile::tempdir().map_err(|err| format!("failed to create temp dir: {err}"))?;
    let decoder = GzDecoder::new(reader);
    let mut archive = Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|err| format!("failed to extract crate tarball: {err}"))?;
    let mut extracted_bytes = 0_u64;

    for (entry_index, entry) in entries.enumerate() {
        if entry_index >= MAX_ARCHIVE_ENTRIES {
            return Err("failed to extract crate tarball: too many archive entries".to_string());
        }
        let mut entry = entry.map_err(|err| format!("failed to extract crate tarball: {err}"))?;
        extracted_bytes = extracted_bytes
            .checked_add(entry.size())
            .filter(|size| *size <= MAX_EXTRACTED_BYTES)
            .ok_or_else(|| {
                "failed to extract crate tarball: extracted archive is too large".to_string()
            })?;

        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() && !entry_type.is_dir() {
            continue;
        }
        entry
            .unpack_in(temp_dir.path())
            .map_err(|err| format!("failed to extract crate tarball: {err}"))?;
    }

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
    let direct = scan.direct;
    let conditional = scan.conditional;

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
                Ok(_) => {
                    return CheckResult::pass(
                        self.name(),
                        format!(
                            "`{}` has no detected floating-point usage in source",
                            crate_info.name
                        ),
                    );
                }
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
                "`{}` has no pinned version — source was not analyzed",
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
        let scan = scan_rust_source(source);
        assert_eq!(scan.direct, 2);
        assert_eq!(scan.conditional, 0);
    }

    #[test]
    fn detects_direct_f64_and_literals() {
        let source = "let x: f64 = 3.14;\nlet y = 0.5f32;";
        let scan = scan_rust_source(source);
        assert_eq!(scan.direct, 3);
        assert_eq!(scan.conditional, 0);
    }

    #[test]
    fn detects_common_float_literal_forms() {
        let source = "let a = 1f32; let b = 2f64; let c = 1e-3;\n\
                      let d = 1.0e5; let e = 1.; let f = 0.5_f32;";
        let scan = scan_rust_source(source);
        assert_eq!(scan.direct, 6);
    }

    #[test]
    fn detects_cfg_guarded_float_on_next_line() {
        let source = "#[cfg(feature = \"float\")]\nlet x: f32 = 1.0;";
        let scan = scan_rust_source(source);
        assert_eq!(scan.direct, 0);
        assert_eq!(scan.conditional, 2);
    }

    #[test]
    fn detects_cfg_guarded_float_in_block() {
        let source =
            "#[cfg(test)]\nmod tests {\n    fn float_fn() {\n        let x: f32 = 1.0;\n    }\n}";
        let scan = scan_rust_source(source);
        assert_eq!(scan.direct, 0);
        assert!(scan.conditional > 0);
    }

    #[test]
    fn ignores_floats_in_line_comments() {
        let source = "let x = 0; // uses f32 internally";
        let scan = scan_rust_source(source);
        assert!(!scan.has_any());
    }

    #[test]
    fn ignores_floats_in_block_comments_and_strings() {
        let source = r###"
            /* f32 and 1.0
               /* nested f64 */ 2f32 */
            let text = "f64 3.14";
            let raw = r#"f32 1e-3"#;
            let bytes = br##"f64 2."##;
            let integer = 2;
        "###;
        let scan = scan_rust_source(source);
        assert!(!scan.has_any());
    }

    #[test]
    fn detects_code_around_block_comments() {
        let source = "let a: f32 = /* f64 */ 1.0;";
        let scan = scan_rust_source(source);
        assert_eq!(scan.direct, 2);
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinks_while_scanning_crate_directory() {
        use std::os::unix::fs::symlink;

        let crate_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        std::fs::write(outside_dir.path().join("outside.rs"), "const X: f64 = 1.0;").unwrap();
        symlink(
            outside_dir.path().join("outside.rs"),
            crate_dir.path().join("linked.rs"),
        )
        .unwrap();
        symlink(crate_dir.path(), crate_dir.path().join("recursive")).unwrap();

        let scan = scan_crate_directory(crate_dir.path());
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
        assert!(result.message.contains("source was not analyzed"));
    }

    #[test]
    #[ignore = "requires network access to crates.io"]
    fn falls_back_to_pass_when_download_fails() {
        let result = FloatUsageCheck.run(&crate_info(
            "this-crate-definitely-does-not-exist-xyz",
            Some("0.0.0"),
        ));
        assert_eq!(result.severity, crate::types::Severity::Pass);
        assert!(result.message.contains("could not be analyzed"));
    }

    #[test]
    #[ignore = "requires network access to crates.io"]
    fn download_and_extract_invalid_crate_returns_error() {
        let result =
            download_and_extract_crate("this-crate-definitely-does-not-exist-xyz", "0.0.0");
        assert!(result.is_err());
    }

    #[test]
    fn extracts_and_scans_local_tarball() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let mut tarball = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = tar::Builder::new(&mut tarball);
            let source = b"pub fn value() -> f64 { 1e-3 }";
            let mut header = tar::Header::new_gnu();
            header.set_size(source.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "sample-1.0.0/src/lib.rs", source.as_slice())
                .unwrap();
            builder.finish().unwrap();
        }

        let temp_dir = extract_crate_tarball(tarball.finish().unwrap().as_slice()).unwrap();
        let source_root = crate_source_root(temp_dir.path(), "sample", "1.0.0");
        let scan = scan_crate_directory(&source_root);
        assert_eq!(scan.direct, 2);
    }
}
