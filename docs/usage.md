# Installation and usage

This guide targets `v0.1.0-beta.1`. Consult [limitations](limitations.md) before treating a report as an integration decision.

## Prerequisites

Use recent stable Rust and Cargo, installed with [rustup](https://rustup.rs/):

```sh
rustup update stable
rustup target add wasm32-unknown-unknown
```

The CLI itself is a native executable. Its WASM check launches Cargo and builds a temporary dependency project. Network access is needed to resolve/download dependencies and, when requested, inspect crate source. Local build-tool requirements are the same as a normal Rust development environment (including the platform linker).

A downloaded native binary does not remove these requirements. If compilation cannot run, the beta may fall back to a blocklist and still report `Pass`.

## Install with Cargo

```sh
cargo install stylus-registry --version 0.1.0-beta.1 --locked
stylus-registry --version
```

Expected version: `stylus-registry 0.1.0-beta.1`. Put Cargo's binary directory on `PATH` if needed (`~/.cargo/bin` on Unix; `%USERPROFILE%\.cargo\bin` on Windows).

The version is explicit because Cargo does not normally select prereleases. The library is published separately as `stylus-compat-core` for Rust consumers; installing the CLI resolves it automatically.

Cargo does not install external registry TOML files. Download the matching data bundle:

```sh
curl -fLO https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/download/v0.1.0-beta.1/stylus-registry-data-v0.1.0-beta.1.tar.gz
tar -xzf stylus-registry-data-v0.1.0-beta.1.tar.gz
stylus-registry check tokio --data-dir stylus-registry-data-v0.1.0-beta.1/data
```

On Windows, download the same file from the release page and extract it with `tar -xzf` or an archive manager. Pass the extracted data directory with `--data-dir` (or `-d`). No registry is loaded automatically without that flag.

## Download a native executable

Choose an archive from the [beta release](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.1):

| Platform | Archive suffix |
|---|---|
| Linux x86-64 (glibc; built on Ubuntu 22.04) | `x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `aarch64-apple-darwin.tar.gz` |
| Windows x86-64 | `x86_64-pc-windows-msvc.zip` |

Each archive contains `stylus-registry` (or `.exe`), `data/`, documentation, fixtures, license and build information. Linux ARM and Alpine/musl binaries are not provided in this beta; use a source build there.

Extract the archive, change into its directory and run:

```sh
./stylus-registry --version
./stylus-registry check tiny-keccak --features keccak --data-dir data/
```

On Windows, use `./stylus-registry.exe`. You can move the executable onto `PATH`; keep the registry files wherever convenient and pass their path explicitly.

`SHA256SUMS` on the release page covers the downloadable archives. On Linux use `sha256sum --check SHA256SUMS --ignore-missing`; on macOS compare `shasum -a 256 <archive>`; on PowerShell compare `(Get-FileHash <archive> -Algorithm SHA256).Hash` with the matching line.

## Install from GitHub or a local checkout

```sh
cargo install --git https://github.com/CoBuilders-xyz/stylus-compatibility-registry.git --tag v0.1.0-beta.1 --locked stylus-registry
```

Or build from source, including the data and examples:

```sh
git clone --branch v0.1.0-beta.1 https://github.com/CoBuilders-xyz/stylus-compatibility-registry.git
cd stylus-compatibility-registry
cargo install --path crates/cli --locked
stylus-registry check-deps --manifest fixtures/release-smoke/Cargo.toml --data-dir data/ --strict
```

## Individual crates

```sh
stylus-registry check hex --version 0.4.3 --features serde --no-default-features --data-dir data/
stylus-registry check tiny-keccak --version 2.0.2 --features keccak --data-dir data/ --json
stylus-registry check tiny-keccak --features keccak,sha3
```

Features can also be repeated: `--features keccak --features sha3`. Do not insert whitespace around comma-separated feature names in this beta; see issue #45 in the limitations guide.

`--version` selects the dependency version; root-level `stylus-registry --version` prints the tool version. Without a dependency version, Cargo resolves an available version and the source float scan does not run. Registry lookup is not version-aware. Use a plain version such as `0.4.3` in these beta examples. Cargo interprets this as a compatible version requirement, not an exact pin; the checker does not faithfully enforce every version expression.

Expected examples when Cargo and the target are available:

- `tiny-keccak --features keccak`: compilation passes; typically 100/100.
- `tiny-keccak` without features: compilation fails because a hash function must be selected.
- `wide` without a registry: SIMD warning; typically 90/100.
- `wide --data-dir data/`: SIMD and float warnings; typically 80/100.
- `tokio --data-dir data/`: registry reports std and async errors; compilation depends on selected features.

These examples describe this beta and can change with dependency versions and the build environment.

## Projects and transitive dependencies

```sh
stylus-registry check-deps --manifest ./Cargo.toml --data-dir ./registry-data --json
stylus-registry check-deps --manifest ./Cargo.toml --data-dir ./registry-data --include-transitive --strict
```

Without `--include-transitive`, the tool parses declared dependencies. With it, Cargo metadata resolves the dependency tree and marks transitive entries in the report. Build features and version resolution can change which dependencies appear. For workspace inheritance, path/git dependencies and complex manifests, review the limitations of the parser and crates.io-based compilation check.

## Exit codes and reports

| Invocation | Exit behavior |
|---|---|
| `check` | 0 even when a check reports `Error`; inspect JSON |
| `check-deps` | 0 unless command execution fails |
| `check-deps --strict` | 1 if the report contains any `Error`; warnings alone return 0 |
| Invalid arguments | Clap reports an argument error and returns nonzero |
| Manifest/registry parse failure | Command reports an error and returns 1 |

`--json` writes a structured report to stdout. Per-crate reports contain `crate_info`, `results` and `score`; project reports also contain `crate_reports`, `overall_score`, `error_count` and `warning_count`. The score is a triage aid. A fallback `Pass` is not the same as successful compilation.

## Use in another project's CI

Copy [the GitHub Actions example](../.github/examples/stylus-check.yml) into that project's `.github/workflows/`. It installs a versioned CLI, downloads the matching registry data, installs the WASM target, and runs `check-deps --strict`. It works without publishing keys.

Use this beta as advisory tooling where false positives or missed findings are acceptable. A passing job is not a replacement for building and validating the actual Stylus contract.
