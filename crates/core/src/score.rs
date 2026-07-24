use crate::types::{CheckResult, CompatibilityScore, Severity};

/// Computes a compatibility score from a set of check results.
///
/// Scoring algorithm:
/// - Start at 100
/// - Each Error: -30 points
/// - Each Warning: -10 points
/// - Minimum score is 0
///
/// This is intentionally simple for the scaffold. Fellows can refine the weights
/// and add per-check scoring (e.g., `no_std` failure is more severe than float warning).
pub fn compute_score(results: &[CheckResult]) -> CompatibilityScore {
    let mut score: i32 = 100;

    for result in results {
        match result.severity {
            Severity::Error => score -= 30,
            Severity::Warning => score -= 10,
            Severity::Pass => {}
        }
    }

    CompatibilityScore::new(score.clamp(0, 100) as u8)
}

/// Computes an overall project score from individual crate scores.
///
/// Uses the minimum score across all crates — a project is only as compatible
/// as its least compatible dependency.
pub fn compute_project_score(crate_scores: &[CompatibilityScore]) -> CompatibilityScore {
    if crate_scores.is_empty() {
        return CompatibilityScore::new(100);
    }

    let min = crate_scores.iter().map(|s| s.value).min().unwrap_or(100);
    CompatibilityScore::new(min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_score_with_all_passing() {
        let results = vec![
            CheckResult::pass("no_std", "ok"),
            CheckResult::pass("wasm_target", "ok"),
            CheckResult::pass("float_usage", "ok"),
            CheckResult::pass("async_usage", "ok"),
        ];
        let score = compute_score(&results);
        assert_eq!(score.value, 100);
        assert_eq!(score.label, "Excellent");
    }

    #[test]
    fn score_decreases_with_warnings() {
        let results = vec![
            CheckResult::pass("no_std", "ok"),
            CheckResult::warning("float_usage", "may use floats"),
        ];
        let score = compute_score(&results);
        assert_eq!(score.value, 90);
    }

    #[test]
    fn score_decreases_heavily_with_errors() {
        let results = vec![
            CheckResult::error("no_std", "requires std"),
            CheckResult::pass("float_usage", "ok"),
        ];
        let score = compute_score(&results);
        assert_eq!(score.value, 70);
    }

    #[test]
    fn score_floors_at_zero() {
        let results = vec![
            CheckResult::error("no_std", "requires std"),
            CheckResult::error("wasm_target", "not wasm-compatible"),
            CheckResult::error("float_usage", "uses floats"),
            CheckResult::error("async_usage", "uses async"),
        ];
        let score = compute_score(&results);
        assert_eq!(score.value, 0);
        assert_eq!(score.label, "Incompatible");
    }

    #[test]
    fn project_score_uses_minimum() {
        let scores = vec![
            CompatibilityScore::new(100),
            CompatibilityScore::new(70),
            CompatibilityScore::new(90),
        ];
        let overall = compute_project_score(&scores);
        assert_eq!(overall.value, 70);
    }

    #[test]
    fn empty_project_gets_perfect_score() {
        let overall = compute_project_score(&[]);
        assert_eq!(overall.value, 100);
    }
}
