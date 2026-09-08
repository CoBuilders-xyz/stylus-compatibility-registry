# Stylus Registry

Review Rust dependencies before building a Stylus contract. Install the beta, understand what each check means, and inspect the evidence behind the release.

[Get started](usage.md) · [Download beta.2](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.2) · [Source code](https://github.com/CoBuilders-xyz/stylus-compatibility-registry)

## Start here

| I want to… | Read |
|---|---|
| Install the CLI and check a dependency | [Installation and usage](usage.md) |
| Understand a score or a passing result | [Check methodology](methodology.md) |
| Understand the Rust workspace and data flow | [Architecture](architecture.md) |
| Evaluate whether the beta fits my project | [Limitations and known issues](limitations.md) |
| Publish the next version | [Release automation](releases.md) |

## Try the beta

```sh
cargo install stylus-registry --version 0.1.0-beta.2 --locked
rustup target add wasm32-unknown-unknown
stylus-registry check tiny-keccak --features keccak
```

The [usage guide](usage.md) explains how to download the registry data and pass `--data-dir`. Cargo installs the executable; native release archives also include the data and documentation.

A score is a triage aid. The beta can report `Pass` when a check falls back without compiling. Read the result messages and validate your complete contract with the Stylus toolchain.

## Registry data

The [September 2026 registry report](reports/2026-09-08/README.md) provides a frozen inventory, reproducible counts and examples of actual CLI reports. It describes curated coverage and configuration pitfalls.

The [roadmap](roadmap.md) describes planned improvements to evidence handling, registry rules and dependency analysis.

## Release package

- [Publication evidence](release.md)
- [Fellowship outcomes](fellowship-outcomes.md)
- [Release notes](release-notes.md)
- [Validation and reproducibility](validation.md)
- [Maintaining this site](publishing.md)
- [Download the documentation supplement](downloads/stylus-registry-docs-v0.1.0-beta.2.tar.gz) · [SHA-256](downloads/SHA256SUMS)

The website follows `main`. The published binaries and Cargo packages remain `v0.1.0-beta.2`; the documentation supplement records its own source commit. Existing release archives retain their original documentation.
