# Check methodology

The beta combines curated flags, lists of crate names and a compilation attempt. Results describe the evidence used by this implementation. They do not certify that a Stylus contract can activate or execute.

## Unit of analysis

A single result concerns a crate name, an optional version requirement, selected features and the default-feature setting. A project report aggregates individual crate reports. Direct analysis reads declared dependencies; transitive analysis asks Cargo metadata for a resolved tree.

The registry matches **names only**. It cannot express a different rule for each version or feature combination. A plain version such as `0.4.3` is a compatible Cargo requirement, not an exact pin. Some version operators are rejected by this beta and can trigger a fallback; see [limitations](limitations.md).

## Evidence by check

| Check | What is inspected | What remains untested |
|---|---|---|
| `no_std` | Registry `requires_std`, then known std-dependent names | Full source analysis and the final contract's runtime requirements |
| `wasm_target` | A blocklist, then `cargo check` for a minimal dependency project targeting `wasm32-unknown-unknown` | Contract linking, activation, execution, size and gas |
| `float_usage` | Registry `has_float`, then known names and optionally downloaded Rust source text | Active feature expressions and emitted WASM instructions |
| `async_usage` | Registry `has_async`, then known runtime names | Complete runtime/async behavior |
| `simd_usage` | Five known crate names | Unlisted libraries, direct intrinsics and automatic vectorization |

The [architecture](architecture.md) documents precedence. For std, float and async, a matching record takes precedence even when its flag is `false`. A false flag is treated as a positive override; it does not mean “unknown”.

## Read the message before the score

| Message or evidence | Interpretation |
|---|---|
| `compiles for wasm32-unknown-unknown` | Cargo successfully checked the tested dependency configuration |
| Compiler error naming a missing hash feature | The tested configuration failed; selecting the required feature may fix it |
| “not in the … blocklist” after compilation was unavailable | Fallback evidence; compilation was not established |
| Source unavailable or not analyzed | No source-based conclusion was established |
| Registry classification | A curated assertion about a name; inspect notes and reproduce the configuration |

The same `Pass` severity can represent different evidence strength. CI release smoke checks explicitly require the successful compilation message so a fallback cannot satisfy those tests. Application consumers should make the same distinction.

## Scoring and exit status

Each crate starts at 100. An error subtracts 30 and a warning subtracts 10, with a floor of zero. Project score is the lowest crate score, or 100 for an empty project. Labels are Excellent (90–100), Good (70–89), Needs Review (50–69), and Incompatible (0–49).

One error can leave a score of 70 and a “Good” label. The label therefore cannot replace reading the errors. `check` exits successfully even when its report contains errors. `check-deps --strict` exits 1 for errors, but warnings or fallback passes do not fail it.

## Registry inventory measures

The [dated report](reports/2026-09-08/README.md) counts records in the two TOML files at the beta tag. File classification, boolean flags, notes and alternative fields are counted separately. Flag categories overlap and must not be added as though they were disjoint populations.

There is no denominator of all Rust crates or all Stylus dependencies in this dataset. “25 compatible-file records” cannot be interpreted as a compatibility success rate, ecosystem coverage percentage or deployment count. Notes and alternative names are stored assertions, not independently verified evidence.

## Reproducing an observation

Record the CLI version, full command, data snapshot, features, Cargo/Rust versions and report messages. Retain a project's lockfile when applicable. The temporary compilation check does not faithfully reproduce every workspace, path/git dependency, patch or target-specific configuration.

The frozen example reports include their commands and toolchain. Re-running them later can differ because dependency resolution and network state can change. Use the [official Stylus documentation](https://docs.arbitrum.io/stylus/overview) for actual contract requirements.
