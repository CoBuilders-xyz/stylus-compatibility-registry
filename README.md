# Stylus Crate Compatibility Registry

A CLI tool and curated registry for evaluating **Rust crate compatibility** with [Arbitrum Stylus](https://docs.arbitrum.io/stylus/overview) contracts.

> **Will this crate work in my Stylus contract? Does it need `no_std`? Will it blow the 24KB limit?**
> This tool answers these questions before you waste hours on compilation errors.

## What It Does

| Feature | Description |
|---------|-------------|
| **Dependency Checker** | Analyzes your `Cargo.toml` and flags crates incompatible with Stylus |
| **Compatibility Score** | Rates each dependency 0-100 based on `no_std`, WASM target, floats, async |
| **Known Crates Registry** | Curated TOML database of pre-verified compatible and incompatible crates |
| **Alternative Suggestions** | Recommends Stylus-friendly replacements for incompatible dependencies |
| **CI Integration** | `--strict` mode exits with code 1 on errors — plug into your CI pipeline |

## Architecture

```
[Your Cargo.toml]
        │
        ▼
[Manifest Parser] ──► [Dependency List]
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
         [no_std Check] [WASM Check] [Float Check] ...
                │           │           │
                └───────────┼───────────┘
                            ▼
                  [Compatibility Score]
                            │
                            ▼
                   [Report (CLI / JSON)]
```

**Stylus constraints checked:**
- `no_std` — Stylus contracts must not use the Rust standard library
- `wasm32-unknown-unknown` — must compile to the WASM target
- No floating-point — `f32`/`f64` opcodes are disallowed for determinism
- No async runtimes — contracts execute synchronously in the Arbitrum VM
- Binary size — compressed WASM must fit within 24KB (uncompressed < 128KB)

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Language | Rust (stable) |
| CLI | clap v4 (derive) |
| Manifest Parsing | cargo_toml |
| Registry Data | TOML files |
| Output | Colored terminal tables, JSON |
| Testing | cargo test (unit + integration) |
| CI/CD | GitHub Actions |

## Getting Started

### Prerequisites

- Rust stable (see `rust-toolchain.toml`)
- cargo (comes with Rust)

### Setup

```bash
# Clone the repository
git clone git@github.com:CoBuilders-xyz/stylus-compatibility-registry.git
cd stylus-compatibility-registry

# Build the project
cargo build
```

### Usage

```bash
# Check a single crate by name
cargo run -- check tokio
cargo run -- check tiny-keccak

# Analyze all dependencies in a Cargo.toml
cargo run -- check-deps --manifest path/to/Cargo.toml --data-dir data/

# Strict mode (exits with code 1 if any errors found — for CI)
cargo run -- check-deps --manifest path/to/Cargo.toml --data-dir data/ --strict

# JSON output (for programmatic consumption)
cargo run -- check-deps --manifest path/to/Cargo.toml --json

# Try it on the included test fixture
cargo run -- check-deps --manifest fixtures/test-project/Cargo.toml --data-dir data/
```

### Running Tests

```bash
# Run all tests
cargo test

# Run only core library tests
cargo test -p stylus-compat-core

# Run only integration tests
cargo test -p stylus-compat-core --test check_no_std
```

### Linting & Formatting

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

## Project Structure

```
stylus-compatibility-registry/
├── crates/
│   ├── core/                    # Library: check logic + scoring
│   │   ├── src/
│   │   │   ├── checks/         # CrateCheck trait + implementations
│   │   │   ├── manifest.rs     # Cargo.toml parser
│   │   │   ├── registry.rs     # Known-crates TOML loader
│   │   │   ├── score.rs        # Compatibility scoring algorithm
│   │   │   └── types.rs        # Shared types (CrateInfo, CheckResult, etc.)
│   │   └── tests/              # Integration tests
│   └── cli/                    # Binary: user-facing CLI
│       └── src/
│           ├── main.rs         # Entrypoint
│           └── commands/       # check, check-deps subcommands
├── data/
│   ├── known-compatible.toml   # Pre-verified Stylus-compatible crates
│   └── known-incompatible.toml # Known incompatible crates with alternatives
├── fixtures/
│   └── test-project/           # Sample Cargo.toml for testing
├── .github/                    # CI workflows + templates
└── rust-toolchain.toml         # Pinned Rust toolchain
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on how to contribute, adding new checks, and the PR workflow.

## License

[MIT](LICENSE)
