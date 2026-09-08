# Fellowship outcomes

The Stylus Crate Compatibility Registry delivers a public CLI, reusable Rust library and curated dependency registry. This page records the project's outputs for the `v0.1.0-beta.2` release and the priorities for further development.

## Delivered capabilities

| Capability | Evidence |
|---|---|
| Public CLI distribution | [stylus-registry on crates.io](https://crates.io/crates/stylus-registry/0.1.0-beta.2) and native executables for four platforms |
| Reusable analysis library | [stylus-compat-core on crates.io](https://crates.io/crates/stylus-compat-core/0.1.0-beta.2), separate from CLI presentation |
| Dependency analysis | Individual crates, direct project dependencies and resolved transitive dependencies, with version and feature options |
| Compatibility reports | Five checks, explanatory messages, JSON output, per-crate scores and strict project mode |
| Curated registry data | [54 unique crate names](reports/2026-09-08/README.md), with a frozen snapshot and reproducible inventory |
| Automated releases | Tag validation, package verification, four native builds, crates.io publication and installation checks |
| Maintainer documentation | Usage, architecture, methodology, limitations, contribution guidance and release operations |

The [release record](release.md) links the public artifacts and successful publication workflow. The [validation guide](validation.md) describes the checks performed on the packages and downloaded executable.

## Technical findings

**Dependency configuration changes the result.** The captured `tiny-keccak` reports show a successful WASM check with the `keccak` feature and a compiler error without a hash feature. Reproducible registry entries need to record configuration alongside the crate name.

**Evidence strength must be visible.** Compilation results and name-based fallbacks answer different questions. Current fallback handling can return `Pass` without compilation; release smoke checks explicitly require evidence of a real WASM build.

**Registry rules need shared precedence.** Std, float and async flags now affect the common check pipeline used by both CLI commands. This makes data updates observable and places responsibility on each entry's accuracy.

**Distribution needs end-to-end validation.** The release pipeline verifies packaged source, native archives and an installation from crates.io. Publication retries check existing package identity before continuing.

## Development priorities

1. Make unavailable evidence explicit and fix compiler pipe draining and rejected-input fallbacks.
2. Introduce version/feature conditions and reproducible evidence for registry entries.
3. Improve workspace, target and transitive-dependency analysis.
4. Add size estimation, alternative suggestions and registry exploration with clearly defined behavior.

The [roadmap](roadmap.md) and [issue tracker](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues) track these improvements. The beta's current behavior is documented in [limitations](limitations.md).
