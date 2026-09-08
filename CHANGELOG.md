# Changelog

## 0.1.0-beta.1

First public beta of the CoBuilders Stylus Crate Compatibility Registry.

- Individual crate and project analysis, with optional transitive dependencies.
- Five checks: std requirements, WASM compilation, floats, async runtimes and known SIMD crate names.
- A curated registry of 54 entries, with std/float/async precedence shared by both commands.
- Feature selection, optional default features, text/JSON reports and strict project exit codes.
- Public Cargo packages and checksummed native release archives for Linux, macOS and Windows.
- Installation, architecture, release and fellowship documentation; tag-driven CI publication.

This is an educational beta. Scores do not certify deployment compatibility. Known compiler-output and invalid-feature fallback bugs (#35 and #45) remain open; see [limitations](docs/limitations.md). Binary size checks, automatic alternative suggestions and a web registry are not included.
