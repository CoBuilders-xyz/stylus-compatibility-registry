# Fellowship outcomes — registry contribution

## Scope

This repository contributes developer tooling to the fellowship's public release and documentation phases. The [dashboard's documentation](https://cobuilders-xyz.github.io/stylus-dashboard/) covers the separate observability project. This page assesses the registry's observable outputs, not overall grant completion.

## Delivered capabilities

| Outcome | Evidence |
|---|---|
| Installable public tooling | [CLI](https://crates.io/crates/stylus-registry/0.1.0-beta.2), [core library](https://crates.io/crates/stylus-compat-core/0.1.0-beta.2), native archives |
| Dependency triage | Single-crate, direct and transitive analysis; [usage](usage.md) |
| Inspectable curated knowledge | 54 unique records and a [frozen inventory](reports/2026-09-08/README.md) |
| Extensible Rust design | Separate core and CLI, five checks, structured reports; [architecture](architecture.md) |
| Automated public releases | Tag validation, four native builds, Cargo publication and installation checks; [release automation](releases.md) |
| Public handoff material | Searchable documentation, check methodology, known issues, source evidence and downloadable documentation |

## Engineering lessons

**Configuration belongs in the example.** `tiny-keccak` needs a hash feature. The dated report includes both the working configuration and the actual compiler failure without it.

**Unavailable evidence needs its own meaning.** The current fallback behavior can return `Pass` without compilation. Tests that only assert a passing score would miss this. Release smoke checks require evidence of real WASM compilation.

**Curated data must affect the shared pipeline.** Std, float and async flags now have the same precedence across the individual and project commands. Incorrect flags can also hide findings, which makes reproducible curation a useful next exercise.

**Test publication beyond compilation.** The first tag failed because the publication script expected an archive that Cargo's dry run did not retain. The corrected preflight now creates the packages and reads their checksums in PR CI. Initial account setup also required email verification; the final retry published both crates and verified installation.

## Recommended student exercises

1. Fix unavailable-evidence handling and compiler pipe draining ([#35](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/35), [#45](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/45)). Verify both the message and severity.
2. Add version/feature conditions and executable evidence to registry entries. Start with one well-understood crate and paired success/failure examples.
3. Expand representative dependency scenarios. Define what coverage means before counting new records.
4. Implement alternative suggestions, size estimation or registry exploration as separate contributions with explicit limits.

The [original beta report](fellowship-report.md) provides development history and additional issue references. An incomplete roadmap is compatible with this educational beta.

## Impact still to measure

This release records no telemetry, user interviews, cohort attendance or measured learning gains. It cannot establish that the tooling caused Stylus adoption. A later fellowship retrospective can add participant reflections, actual integrations and stakeholder feedback as separately collected evidence.
