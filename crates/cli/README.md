# stylus-registry

Experimental CLI from the CoBuilders Stylus Fellowship for reviewing Rust dependencies before building Arbitrum Stylus contracts.

```sh
cargo install stylus-registry --version 0.1.0-beta.2 --locked
rustup target add wasm32-unknown-unknown
stylus-registry check tiny-keccak --features keccak
stylus-registry check-deps --manifest Cargo.toml --json
```

The CLI checks standard-library requirements, WASM compilation, float usage, async runtimes and known SIMD crate names. It emits human-readable or JSON reports. `check-deps --strict` fails on reported errors, not warnings.

The WASM compilation check still needs Rust, Cargo, the WASM target and network access. Cargo installs the executable; download the versioned registry data separately to use `--data-dir`.

This is a beta heuristic checker, not a deployment validator. Read the result messages: some checks can fall back to a passing blocklist result when compilation or source analysis could not run.

- [Installation, registry download and examples](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/docs/usage.md)
- [Known limitations](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/docs/limitations.md)
- [Releases](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases)

License: MIT.
