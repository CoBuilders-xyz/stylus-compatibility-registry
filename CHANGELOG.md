# Changelog

## 0.1.0-beta.2

- Publish the educational beta described below, including the CLI, core library, native binaries, registry data and documentation.
- Fix publication preflight to retain the verified Cargo archives before reading their checksums.
- Exercise the publication script in dry-run mode on pull requests, including the first-publication and retry checks.
- Preserve `v0.1.0-beta.1` as a failed publication attempt; no crates or GitHub release were published for that tag.
- The [known beta limitations](docs/limitations.md) still apply.

## 0.1.0-beta.1

First public beta of the CoBuilders Stylus Crate Compatibility Registry.

- Individual crate and project analysis, with optional transitive dependencies.
- Five checks: std requirements, WASM compilation, floats, async runtimes and known SIMD crate names.
- A curated registry of 54 entries, with std/float/async precedence shared by both commands.
- Feature selection, optional default features, text/JSON reports and strict project exit codes.
- Public Cargo packages and checksummed native release archives for Linux, macOS and Windows.
- Installation, architecture, release and fellowship documentation; tag-driven CI publication.

This is an educational beta. Scores do not certify deployment compatibility. Known compiler-output and invalid-feature fallback bugs (#35 and #45) remain open; see [limitations](docs/limitations.md). Binary size checks, automatic alternative suggestions and a web registry are not included.
