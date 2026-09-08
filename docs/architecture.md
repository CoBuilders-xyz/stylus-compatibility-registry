# Architecture

## Purpose and boundaries

The registry is a native Rust command-line application with a reusable Rust library. It helps developers triage dependency choices for Stylus projects. It is not a contract compiler or an on-chain validator. The beta has no server, hosted API, persistent database or telemetry component.

## Workspace layout

| Component | Responsibility |
|---|---|
| `crates/cli` / `stylus-registry` | Clap arguments, selecting commands, console/JSON output and process exit codes |
| `crates/core` / `stylus-compat-core` | Manifest parsing, registry loading, check orchestration and scoring |
| `data/` | Version-controlled curated TOML records |
| `fixtures/` | Mixed, transitive and passing example projects |
| `scripts/` | Release verification, packaging, smoke checks and publication |
| `.github/workflows/` | Pull-request validation and tag-driven releases |

The CLI delegates compatibility decisions to the core. Output indentation and color remain in the CLI, so JSON/core messages do not depend on terminal formatting.

## Analysis flow

```text
check arguments ────────────────────────> CrateInfo
                                                 |
Cargo.toml -> parse declared dependencies         |
           or resolve Cargo metadata tree -------+
                                                 v
             optional TOML registry -> name lookup
                                                 |
                                                 v
                                    run_all_checks
                      no_std / wasm / float / async / simd
                                                 |
                                                 v
                                      per-crate score
                                                 |
                          project counts + minimum score
                                                 |
                                                 v
                                          text or JSON
```

1. `check` constructs a single `CrateInfo` from arguments, including features and default-feature selection. `check-deps` calls `analyze_project_with_transitive`.
2. Direct analysis uses `cargo_toml`; transitive analysis uses `cargo_metadata` and marks dependencies as transitive. See `manifest.rs` for the supported resolution behavior.
3. If a data directory is provided, `KnownCratesRegistry` loads `known-compatible.toml` followed by `known-incompatible.toml` into a `HashMap`. Later records with the same name replace earlier ones. Missing files are skipped; malformed existing TOML is an error.
4. `checks::run_all_checks` executes the five checks once, in a fixed order. Std, float and async checks receive the matching registry entry.
5. Each result has a name, severity and explanatory message. Scores and counts are computed from these results, and the CLI renders the report.

## Shared data model

- `CrateInfo`: name, optional version, selected features, default-feature flag and transitive flag.
- `KnownCrateEntry`: name, `requires_std`, `has_float`, `has_async`, optional `max_version`, alternative and notes. `max_version` and alternatives are stored but not enforced/rendered as automated advice.
- `CheckResult`: `check_name`, `severity` (`Pass`, `Warning`, `Error`), message.
- `CrateReport`: input crate information, ordered results and score.
- `ProjectReport`: manifest path, crate reports, aggregate error/warning counts and overall score.

Serde provides JSON serialization. TOML parsing errors and filesystem errors propagate through typed core errors to a CLI error message and nonzero exit status.

## Evidence and precedence

| Check | Registry hit | Registry miss |
|---|---|---|
| `no_std` | `requires_std=true` is an error; false passes | Known std-dependent names are errors |
| `wasm_target` | Registry is not consulted | Known OS-dependent names fail immediately; otherwise attempt Cargo compilation |
| `float_usage` | `has_float=true` warns; false passes without a scan | Known float names warn; otherwise scan downloaded Rust source if a version was supplied |
| `async_usage` | `has_async=true` is an error; false passes | Known async runtime names are errors |
| `simd_usage` | Registry is not consulted | Known SIMD crate names warn; other names pass |

Registry precedence is deliberate: adding a curated flag must change the result, and registry hits avoid unnecessary source downloads. It also places responsibility on data maintainers. A false flag is a positive override, not “unknown”. Records currently cannot express per-version or per-feature conditions.

## WASM compilation and source analysis

The compilation check writes a temporary minimal Cargo project and invokes `cargo check --target wasm32-unknown-unknown --quiet`. It passes the dependency's features and default-feature setting into its manifest. The timeout is 120 seconds. The executable can be overridden by `STYLUS_COMPAT_CARGO`, primarily for tests. A successful result confirms that this dependency configuration passed Cargo's check, not that a real contract was linked or activated.

The float check downloads a crates.io archive with bounded compressed/extracted size, extracts ordinary files/directories, and scans Rust source. It strips comments/string content and approximates `cfg` guards, without interpreting active feature expressions. The source scan is separate from Cargo resolution and the emitted WASM.

Both checks have fallbacks that may return `Pass` when evidence is unavailable. The pipe-buffer timeout bug and input-validation fallbacks are described in [limitations](limitations.md).

## Scores and CLI semantics

Each crate starts at 100. Errors subtract 30, warnings subtract 10; the result is clamped to 0–100. Labels are Excellent (90–100), Good (70–89), Needs Review (50–69), or Incompatible (0–49). An empty project scores 100; other project scores use the minimum crate score.

`check-deps --strict` returns 1 if any check reports `Error`. An ordinary `check` still returns 0 when its report contains errors. JSON consumers should inspect the severities and messages rather than infer success from the process code or label.

## Testing and release architecture

Unit tests cover parsing, registry behavior, precedence, check heuristics and scoring. Integration tests cover direct/transitive projects and CLI registry selection. Some network-dependent unit tests remain ignored by the default suite; CI explicitly exercises the two WASM compilation tests and the release smoke checks.

The tag-driven release workflow validates versions and source ancestry, tests packages, builds four native targets, runs a real WASM smoke check on each runner, and packages the binary with registry data and documentation. Cargo's workspace packaging verifies the library and CLI before upload. The library is published before the dependent CLI. A GitHub prerelease is published with checksummed archives only after validation and crates.io publication succeed.

## Extending the exercise

Implement `CrateCheck`, add the module, and add its invocation to `run_all_checks`. If a new check uses registry evidence, define explicit precedence and fallback behavior and test both paths. Keep terminal presentation out of the core. Prefer evidence-backed entries with reproducible version/feature examples.

The [contribution guide](../CONTRIBUTING.md) and [open issues](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues) provide student-sized follow-up work. The beta intentionally leaves feature-aware rules, stronger evidence states, final-WASM inspection and several user-facing commands unfinished.
