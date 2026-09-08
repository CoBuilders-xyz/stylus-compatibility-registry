# Release evidence

## Published beta

**Version:** `v0.1.0-beta.2` · **Published:** 8 September 2026 · **Source:** `59460476068542e5cb9e1fef88a2c51f3a2b7003`.

The [GitHub prerelease](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.2), [CLI crate](https://crates.io/crates/stylus-registry/0.1.0-beta.2) and [core crate](https://crates.io/crates/stylus-compat-core/0.1.0-beta.2) are public. The [release workflow](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/actions/runs/34268848187) succeeded on its second attempt after crates.io account email verification.

## Published components

| Component | Distribution |
|---|---|
| CLI | `stylus-registry 0.1.0-beta.2` on crates.io |
| Core library | `stylus-compat-core 0.1.0-beta.2` on crates.io |
| Native executables | Linux x86-64, macOS Intel/Apple Silicon and Windows x86-64 |
| Registry data | Standalone data archive and bundled with each native executable |
| Source and license | Tagged public repository under the MIT license |
| Documentation | Usage, architecture, check methodology, limitations and release operations |

## Project outcomes

[Fellowship outcomes](fellowship-outcomes.md) records the delivered capabilities, technical findings and development priorities for this release.

## Distribution evidence

The [publication record](reports/2026-09-08/publication.json) lists the public URLs, release commit, workflow result and SHA-256 values captured from GitHub and crates.io. The five original archives contain four native executables and one standalone data package. Their published `SHA256SUMS` file remains unchanged.

The [validation record](validation.md) explains which checks passed and what they establish.

## Documentation supplement

The expanded website and report supplement the existing beta. They do not change the executable version, move its tag or replace original release assets. Download the [documentation archive](downloads/stylus-registry-docs-v0.1.0-beta.2.tar.gz) and its [checksum](downloads/SHA256SUMS). `BUILD-INFO.json` records the documentation source commit separately from the beta source commit.

The Docs workflow validates and publishes the site from `main`. The same documentation archive can be attached as an additional asset to the existing beta release, with its documentation commit in the filename; see [publishing](publishing.md).
