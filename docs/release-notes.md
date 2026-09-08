# Release notes

## 0.1.0-beta.2 — 8 September 2026

First successfully published beta of the Stylus Crate Compatibility Registry.

- Installable CLI and reusable core library on crates.io.
- Single-crate and project dependency reports, including transitive analysis, feature selection, JSON and strict project mode.
- Five checks and 54 curated registry records.
- Linux x86-64, macOS Intel/Apple Silicon and Windows x86-64 native archives, with registry data and documentation.
- Tag-driven validation, native smoke checks, crates.io installation verification and checksummed GitHub release assets.

The publication preflight uses `cargo package` so verified archives exist before reading their checksums. The earlier `v0.1.0-beta.1` attempt did not publish crates or a GitHub release; its tag is preserved.

## Documentation supplement

The expanded documentation adds a searchable site with appearance controls, a page index and architecture diagram; a dated registry snapshot and example CLI reports; and dedicated methodology, roadmap and release-evidence pages.

This supplement leaves the published Cargo packages and native archives unchanged. Existing archives contain the original beta documentation; the new downloadable documentation package identifies its own source commit.

## Known limitations

Fallback passes, compiler output timeouts, name-only registry rules and incomplete source/SIMD detection remain documented in [limitations](limitations.md). Size estimation and automatic alternative suggestions remain planned features. A score of 100 is not deployment certification.

See the [source changelog](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/v0.1.0-beta.2/CHANGELOG.md) and [public release](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.2).
