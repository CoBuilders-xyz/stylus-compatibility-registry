# Roadmap

The beta implements single-crate and project dependency analysis, five checks, curated registry data and text/JSON reports. The following improvements address its current limitations.

## Evidence handling

Make unavailable compilation and source evidence explicit instead of representing a fallback as `Pass`. Fix compiler pipe draining so large diagnostics cannot turn a failed compilation into a timeout and passing fallback.

Track [#35](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/35) and [#45](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/45). Validation should distinguish successful compilation, a real compiler error and an unavailable check.

## Registry rules

Add version and feature conditions, evidence links and reproducible configurations to records. Define what happens when a condition is unknown or does not match. Current lookup uses only the crate name, and `max_version` is not enforced.

Registry changes should include a command showing the relevant behavior. The [coverage report](reports/2026-09-08/README.md) provides the current baseline.

## Dependency analysis

Improve workspace inheritance, path/git dependency handling, target-specific resolution and source analysis. Final-WASM inspection would provide evidence that name-based SIMD warnings and source-text float checks cannot establish.

## Planned commands

Binary size estimation, automatic alternative suggestions, registry search/info commands and configurable scoring are not implemented in this beta. Stored `alternative` values are data, not tested replacement recommendations.

See the [issue tracker](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues) for individual changes and the [contribution guide](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/CONTRIBUTING.md) for development instructions.
