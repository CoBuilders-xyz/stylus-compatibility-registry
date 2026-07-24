# Contributing to Stylus Crate Compatibility Registry

Thank you for contributing! This is a collaborative project built by the Stylus Fellowship cohort. This guide will help you get started.

## Architecture Overview

This is a Cargo workspace with two crates:

| Crate | Role |
|-------|------|
| `crates/core` | Library: compatibility checks, scoring, manifest parsing, registry |
| `crates/cli` | Binary: user-facing CLI with `check` and `check-deps` commands |

**Data flow:**

```
Cargo.toml → Manifest Parser → CrateInfo list → CrateCheck pipeline → Score → Report
                                      ↑
                               Known Crates Registry (TOML files)
```

The CLI **MUST** use the core library for all logic. No check logic should live in the CLI crate — it only handles argument parsing, output formatting, and exit codes.

### Extension Point: CrateCheck Trait

All compatibility checks implement the `CrateCheck` trait:

```rust
pub trait CrateCheck {
    fn name(&self) -> &str;
    fn run(&self, crate_info: &CrateInfo) -> CheckResult;
}
```

To add a new check, implement this trait and register it in `checks/mod.rs`. See "Adding a New Check" below.

### Data Model

Key types in `crates/core/src/types.rs`:

- `CrateInfo` — name, version, features, default_features
- `CheckResult` — check_name, severity (Pass/Warning/Error), message
- `CompatibilityScore` — value (0-100), label (Excellent/Good/Needs Review/Incompatible)
- `CrateReport` — per-crate results + score
- `ProjectReport` — all crates + overall score + error/warning counts

## Development Workflow

### 1. Pick an Issue

All work is tracked in [GitHub Issues](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues). Issues are organized by area and ordered by priority — pick the next available one that isn't blocked or assigned.

**Labels by area:**

- `core` — Check logic, scoring, types, registry
- `cli` — CLI commands, output formatting
- `registry` — Known-crates data files (TOML)
- `infra` — CI/CD, tooling, automation
- `docs` — Documentation

### 2. Create a Branch

```bash
git checkout main
git pull origin main
git checkout -b feat/your-feature-name
```

**Branch naming conventions:**

- `feat/description` — New feature
- `fix/description` — Bug fix
- `refactor/description` — Code refactoring
- `docs/description` — Documentation
- `chore/description` — Tooling, CI, dependencies

### 3. Make Your Changes

```bash
# Build the project
cargo build

# Run the CLI during development
cargo run -- check-deps --manifest fixtures/test-project/Cargo.toml --data-dir data/
```

### 4. Verify Your Work

Before pushing, ensure:

```bash
cargo fmt --check    # Code is formatted
cargo clippy -- -D warnings  # No lint warnings
cargo test           # Tests pass
cargo build          # Project builds
```

CI runs all four automatically on every PR.

### 5. Submit a Pull Request

Push your branch and open a PR against `main`:

```bash
git push -u origin feat/your-feature-name
```

In your PR description, include:

- **What** — Brief description of the change
- **Why** — Link to the issue it addresses
- **How to test** — Steps to verify the change works

## Code Guidelines

### Rust

- Use `thiserror` for error types — no `unwrap()` in library code
- Prefer returning `Result` over panicking
- Use `clippy` with `-D warnings` — all warnings are errors in CI

### Module Organization

| What | Where |
|------|-------|
| Check implementations | `crates/core/src/checks/` |
| Shared types | `crates/core/src/types.rs` |
| Manifest parsing | `crates/core/src/manifest.rs` |
| Registry data loading | `crates/core/src/registry.rs` |
| Scoring algorithm | `crates/core/src/score.rs` |
| CLI commands | `crates/cli/src/commands/` |
| Known-crates data | `data/` |
| Test fixtures | `fixtures/` |
| Integration tests | `crates/core/tests/` |

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add binary size estimation check
fix: handle missing version field in Cargo.toml
docs: add crates.io API integration guide
chore: update clap to v4.5
test: add integration tests for float detection
```

## Adding a New Check

This is the most common contribution. Follow these steps:

### 1. Create the check file

Create `crates/core/src/checks/your_check.rs`:

```rust
use crate::checks::CrateCheck;
use crate::types::{CheckResult, CrateInfo};

pub struct YourCheck;

impl CrateCheck for YourCheck {
    fn name(&self) -> &str {
        "your_check"
    }

    fn run(&self, crate_info: &CrateInfo) -> CheckResult {
        // Your check logic here
        CheckResult::pass(self.name(), format!("`{}` passed", crate_info.name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_issue() {
        // Test with a crate that should fail
    }

    #[test]
    fn passes_clean_crate() {
        // Test with a crate that should pass
    }
}
```

### 2. Register the check

Add it to `crates/core/src/checks/mod.rs`:

```rust
pub mod your_check;

// In all_checks():
pub fn all_checks() -> Vec<Box<dyn CrateCheck>> {
    vec![
        // ... existing checks ...
        Box::new(your_check::YourCheck),
    ]
}
```

### 3. Add unit tests

Every check must have at least two unit tests: one that detects an issue and one that passes a clean crate. See the existing checks for examples.

### 4. Update the scoring (if needed)

If your check has different severity weights than the default, update `crates/core/src/score.rs`.

## Adding a Crate to the Registry

This is a great first contribution that doesn't require writing Rust code.

### 1. Determine compatibility

Check if the crate supports `no_std`, compiles for `wasm32-unknown-unknown`, avoids floats, and avoids async. You can test with:

```bash
# Quick check: does it declare no_std support?
# Look at the crate's Cargo.toml and lib.rs on crates.io or GitHub

# Compilation check:
cargo new --lib test-crate && cd test-crate
echo '#![no_std]' > src/lib.rs
# Add the crate as a dependency
cargo build --target wasm32-unknown-unknown
```

### 2. Add the entry

Edit `data/known-compatible.toml` or `data/known-incompatible.toml`:

```toml
[[crate]]
name = "crate-name"
requires_std = false          # true if it needs std
has_float = false             # true if it uses f32/f64
has_async = false             # true if it uses async runtimes
alternative = "other-crate"  # optional: Stylus-friendly alternative
notes = "Brief explanation"   # optional: configuration tips
```

### 3. Open a PR

Include evidence: a link to the crate's `Cargo.toml` showing `no_std` support, or a compilation error log if incompatible.

## Getting Help

- Open an issue with the `question` label
- Tag maintainers in PR comments for reviews
- Check existing issues before creating new ones
