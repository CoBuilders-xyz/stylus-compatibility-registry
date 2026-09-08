# Stylus registry: ecosystem and fellowship report

**Release:** `v0.1.0-beta.2`

**Prepared:** 2026-09-08

**Scope:** the Stylus Crate Compatibility Registry repository only.

This document is the registry's contribution to the fellowship's public release and documentation deliverables. It does not evaluate the separate ecosystem dashboard, another tooling repository, or the fellowship's overall grant completion.

## Problem addressed

Rust developers evaluating dependencies for a Stylus contract need to understand target compilation, runtime assumptions and feature configuration. Library names and a successful native build are not enough to answer these questions. Knowledge of possible incompatibilities is also useful to students learning Rust tooling, WASM and open-source contribution workflows.

The project provides an executable starting point: a CLI, a reusable core library, version-controlled classifications and reports that point developers toward further investigation. It intentionally remains an educational beta with an open roadmap.

## Public outputs

| Output | Evidence |
|---|---|
| Open-source implementation and license | [Repository](https://github.com/CoBuilders-xyz/stylus-compatibility-registry), [MIT license](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/v0.1.0-beta.2/LICENSE) |
| Versioned public distribution | [Beta release](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.2), [CLI crate](https://crates.io/crates/stylus-registry), [core crate](https://crates.io/crates/stylus-compat-core) |
| Usage guidelines | [Installation, examples and CI integration](usage.md) |
| Architecture | [Component responsibilities, evidence precedence and extension points](architecture.md) |
| Transparent limitations | [Beta limitations and known issues](limitations.md) |
| Repeatable publication | [Release process](releases.md), [release workflow](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/v0.1.0-beta.2/.github/workflows/release.yml) |

The release links identify the deliverable produced by the publication workflow. GitHub source archives alone are supplemented with four native executables, versioned data, checksums and the Cargo installation path.

## Implemented capabilities

- Single-crate analysis with dependency version/features and optional default features.
- Direct project dependency analysis and an optional resolved transitive tree.
- Five checks: std requirements, WASM compilation, float usage, async runtimes and known SIMD crate names.
- Shared registry precedence for std, float and async decisions in both CLI commands.
- Text/JSON reports, per-crate and minimum project scores, and strict project exit codes.
- A curated snapshot with **54 unique crate names**: 25 in the compatible file and 29 in the incompatible file.

These counts describe registry records, not contracts deployed or independently verified on-chain. See [the data files](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/tree/v0.1.0-beta.2/data/) and [scoring/evidence rules](architecture.md).

## Evidence from repository development

The repository was created on 2026-07-24. Its pre-release history includes the initial scaffold (#1), WASM compilation (#32), float source analysis (#33), transitive dependency analysis (#34), CLI features/output (#40), registry precedence (#42), single-crate registry loading (#44), and the SIMD check (#41). These pull requests make the technical progression inspectable.

The pre-release API reported contributions from [nachfq](https://github.com/nachfq) and [mariano-aguero](https://github.com/mariano-aguero). Attribution remains available in the [commit history](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/commits/main/) and pull requests. GitHub account counts and squash commits are not a measure of attendance, student count, hours worked or individual learning outcomes.

The integration reviewed before release preparation had 55 passing Rust tests on Linux, with four tests ignored by the default suite. The release pipeline additionally executes the two real WASM compilation tests, verifies package builds and checks native artifacts on four platforms. Test success validates the implemented behavior, including its documented limitations; it does not establish universal crate compatibility.

## Stylus adoption insights from the tooling exercise

These are technical observations from development and local evaluation, not a market-adoption study:

1. **Configuration is part of compatibility.** `tiny-keccak` needs a selected hash feature. Exposing feature flags removed an inability to test that valid configuration. A crate name alone was insufficient evidence.
2. **Curated knowledge needs an executable path.** Parsed float/async flags originally did not affect checks. Wiring them into the common pipeline made updates to the registry observable through the CLI.
3. **Direct dependencies do not describe every runtime assumption.** A transitive tree exposes lower-level dependencies that a manifest-only review misses. It also increases analysis cost and requires careful interpretation of target and feature resolution.
4. **Compilation and activation are separate questions.** A dependency can pass Cargo's WASM check while other contract constraints remain untested. The SIMD name warning illustrates the gap between heuristic detection and inspecting a final artifact.
5. **Missing evidence needs clearer representation.** Current fallbacks can report `Pass` when compilation or scanning was unavailable. Issues #35 and #45 provide concrete follow-up work on reliability and user communication.

These observations suggest that accessible setup, reproducible examples, visible evidence and configuration-aware rules can reduce friction for developers exploring Stylus. The repository alone cannot quantify how much adoption increased.

## Fellowship outcomes and recommendations

The observable outcome is a public, extensible codebase connecting Rust workspace design, package management, WASM compilation, parsing, testing, CLI design, data curation and CI releases. The issue/PR workflow provides bounded assignments for further cohorts. The architecture separates check logic from presentation so students can add a check without redesigning the whole tool.

Recommended next exercises, in order of practical value:

1. Make unavailable evidence explicit and fix compiler pipe handling and rejected-feature fallbacks (#35, #45).
2. Model registry conditions by features and versions, with evidence links and reproducible crate examples (#6, #11).
3. Expand evaluated scenarios and representative Stylus dependency coverage (#29–#31).
4. Add user-facing capabilities such as alternative suggestions, size estimation or registry exploration as independent contributions (#5, #7, #14, #15).

Completing every issue is not a prerequisite for this beta. Follow-up work should improve specific observable behavior, with tests and honest documentation, rather than inflate the compatibility score's meaning.

## Measurement boundaries

This release includes no telemetry and this report makes no claim about production users, downloads, activated contracts, deployment counts or measured student skill gains. Download statistics and future user feedback may support later reports, but cannot be inferred from stars, test counts or registry size. Cohort attendance, student reflections and program-wide outcomes belong in the fellowship's broader report if separately collected.

## Sources and reproducibility

The inventory is reproducible from `Cargo.toml`, the two `data/*.toml` files, the CLI help, repository history and the linked workflows. Review the versioned source at [v0.1.0-beta.2](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/tree/v0.1.0-beta.2) rather than assuming future `main` has the same behavior.

The underlying platform documentation is [Arbitrum Stylus](https://docs.arbitrum.io/stylus/overview); the registry is a fellowship tool and does not replace that specification or the actual contract validation process.
