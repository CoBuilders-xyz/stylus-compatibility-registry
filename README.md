# Stylus Crate Compatibility Registry

**Beta · educational tooling from the CoBuilders Stylus Fellowship · MIT licensed**

A Rust CLI and curated registry for reviewing dependencies before building an [Arbitrum Stylus](https://docs.arbitrum.io/stylus/overview) contract. It helps identify configuration issues and dependencies worth investigating; it does not certify that a contract can activate or execute on-chain.

## Install

Install the beta from crates.io:

```sh
cargo install stylus-registry --version 0.1.0-beta.2 --locked
rustup target add wasm32-unknown-unknown
stylus-registry check tiny-keccak --features keccak
```

Use a recent stable Rust toolchain. A prerelease version must be requested explicitly. Precompiled Linux, macOS and Windows executables are available in [GitHub Releases](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.2). They still need Cargo and the WASM target for compilation checks.

Cargo installs the executable, not the registry data. See the [installation guide](docs/usage.md) for the versioned data download, binary installation, and installation from GitHub.

## What the beta does

| Check | Evidence used |
|---|---|
| `no_std` | Curated registry flag, then a list of known std-dependent crates |
| `wasm_target` | Attempts `cargo check --target wasm32-unknown-unknown` in a temporary project; also uses a blocklist |
| `float_usage` | Curated flag, then a blocklist and optional source-text scan |
| `async_usage` | Curated flag, then known runtime names |
| `simd_usage` | A list of five crate names associated with SIMD |

The CLI analyzes individual crates, direct dependencies or a resolved transitive tree, and prints a readable report or JSON. Scores start at 100, subtract 30 for each error and 10 for each warning. The project score is its lowest crate score.

**Read the messages, not just the score.** Some unavailable checks currently fall back to `Pass`. A score of 100 is not proof of Stylus compatibility. Float/std classifications are project heuristics and curated assumptions, not an authoritative specification of the Stylus VM. See [known limitations and bugs](docs/limitations.md).

## Use the registry

After extracting a release bundle, use its `data/` directory:

```sh
stylus-registry check tokio --data-dir data/
stylus-registry check tiny-keccak --features keccak --data-dir data/
stylus-registry check-deps --manifest path/to/Cargo.toml --data-dir data/ --json
stylus-registry check-deps --manifest path/to/Cargo.toml --data-dir data/ --include-transitive
stylus-registry check-deps --manifest path/to/Cargo.toml --data-dir data/ --strict
```

`--strict` exits with status 1 when the project report contains errors. Warnings alone do not fail it. The individual `check` command reports check failures without changing its exit status; use its JSON report when scripting.

The beta includes 54 curated entries: 25 in `known-compatible.toml` and 29 in `known-incompatible.toml`. These names reflect the project's classifications, not 54 independently deployed and verified contracts. Registry lookup is by crate name and takes precedence over the std, float and async fallback checks.

## Build and contribute

```sh
git clone https://github.com/CoBuilders-xyz/stylus-compatibility-registry.git
cd stylus-compatibility-registry
cargo build --locked
cargo run --locked -- check tiny-keccak --features keccak --data-dir data/
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
```

The [mixed dependency fixture](fixtures/test-project/Cargo.toml) intentionally produces errors; `tiny-keccak` in that fixture has no hash feature configured. Use [the release smoke fixture](fixtures/release-smoke/Cargo.toml) for a passing example.

## Documentation package

[Browse the documentation site](https://cobuilders-xyz.github.io/stylus-compatibility-registry/) or start with the [documentation index](docs/README.md). The site includes methodology, a reproducible registry report and public release evidence.

- [Installation, usage and CI integration](docs/usage.md)
- [Architecture and extension points](docs/architecture.md)
- [Beta limitations and known issues](docs/limitations.md)
- [Release process and crates.io publishing](docs/releases.md)
- [Registry ecosystem and fellowship report](docs/fellowship-report.md)
- [Contributing](CONTRIBUTING.md) and [changelog](CHANGELOG.md)

Binary size estimation, automatic alternative suggestions, a searchable web registry, and broader ecosystem coverage remain student project opportunities. See the [open issues](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues).

## License

[MIT](LICENSE). Source, registry data and documentation are public in this repository.
