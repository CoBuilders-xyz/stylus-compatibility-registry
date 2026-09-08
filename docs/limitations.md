# Beta limitations and known issues

The `0.1.0-beta.2` release is educational, experimental tooling. Its reports combine a curated database, name-based heuristics and a compilation attempt. They are not an audit, a proof of VM compatibility, or evidence of on-chain deployment.

## Interpret the checks accurately

- **Registry authority:** std, float and async flags override fallback evidence, including a `false` flag overriding a blocklist or bypassing a source scan. Incorrect entries can hide problems. Lookup uses only the crate name; `max_version` is currently not enforced and configuration notes are not executable rules.
- **Compilation:** the tool invokes `cargo check` on an empty library with the requested crate as a dependency. It does not build/link your actual contract, inspect its final WASM, measure size or gas, activate it, or execute transactions. Path/git dependencies are not reproduced faithfully by a crates.io name/version check.
- **Unavailable evidence:** missing Cargo/target, network failures, timeouts or rejected input may produce a passing blocklist fallback. Float source download failures also currently return `Pass`. Read messages such as “not in the ... blocklist”, “source could not be analyzed”, or “source was not analyzed” as missing evidence.
- **Float scan:** it searches Rust source text for float types/literals and approximates conditional compilation. It does not evaluate active Cargo features or inspect emitted instructions. A missing version skips the scan; version requirements are not resolved into an exact source version by this scan. The compilation validator also rejects operators such as `=`, `^`, `~` and `*` in supplied version strings and may fall back to `Pass`; use plain versions such as `0.4.3` for these beta examples.
- **Std/async:** registry entries and known crate names are not a complete feature-aware analysis of library code or runtime behavior. The beta's policy and wording should not be treated as the current Stylus protocol specification.
- **SIMD:** five crate names are recognized. Direct intrinsics, `core::simd`, automatic vectorization and unlisted crates are not detected. A matching name is a warning regardless of actual compiler flags or emitted code.
- **Scoring:** every error costs 30 points and every warning 10. An error can still leave a label such as “Good”. `--strict` fails on errors, not on warnings or unavailable evidence that was classified as `Pass`.

For actual SDK and execution requirements, consult [Arbitrum's Stylus documentation](https://docs.arbitrum.io/stylus/overview) and validate the complete contract with the appropriate Stylus toolchain.

## Known bugs retained in this beta

| Issue | Behavior | Workaround |
|---|---|---|
| [#35](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/35) | Large compiler output can fill a pipe, cause the 120-second timeout and then return a passing fallback | Treat a slow fallback as inconclusive; run the dependency/contract compilation directly |
| [#45](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/45) | Some rejected feature strings silently skip compilation and can score 100 | Use valid feature names, comma-separated without spaces or repeated flags; verify the result explicitly says it compiled |
| Registry path handling | A missing directory or one without the expected TOML filenames is treated as an empty registry | Check the extracted files and pass the correct directory; verify registry notes appear |

These are known limitations, not successful compatibility validations. Do not deploy solely on the tool's score.

## Intentionally unfinished scope

Binary size estimation (#5), feature-aware registry conditions (#6), automatic alternative suggestions (#7), full source verification for `no_std` (#11), search/info commands, configurable scoring, a database and a web frontend are not part of this release. `alternative` values are present in the data but are not printed as automatic recommendations by the CLI.

Coverage consists of 54 curated entries. The “compatible” and “incompatible” filenames are classifications inherited from the educational registry, not independently verified deployment results for every version and configuration.

## Reproducibility and external effects

Dependency analysis may download code and run dependency build scripts through Cargo. The lockfile pins the tool's own dependencies, not every temporary crate check. Fully specified version arguments and project lockfiles improve repeatability, but the tool does not reproduce every build flag, patch, target-specific setting or workspace configuration.

Report a new limitation through the [issue tracker](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/issues/new/choose), including the command, tool/dependency versions, relevant feature flags, actual result and expected result. Remove secrets from logs before sharing them.
