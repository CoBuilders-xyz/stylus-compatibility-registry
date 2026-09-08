# Release evidence and deliverables

## Published beta

**Version:** `v0.1.0-beta.2` · **Published:** 8 September 2026 · **Source:** `59460476068542e5cb9e1fef88a2c51f3a2b7003`.

The [GitHub prerelease](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/releases/tag/v0.1.0-beta.2), [CLI crate](https://crates.io/crates/stylus-registry/0.1.0-beta.2) and [core crate](https://crates.io/crates/stylus-compat-core/0.1.0-beta.2) are public. The [release workflow](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/actions/runs/34268848187) succeeded on its second attempt after crates.io account email verification.

## Deliverable mapping

| Phase commitment | Registry evidence | State |
|---|---|---|
| Public release of the second Stylus tooling initiative | Cargo packages and four native archives | Published beta |
| Public open-source release, registry portion | [Repository](https://github.com/CoBuilders-xyz/stylus-compatibility-registry), [MIT license](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/v0.1.0-beta.2/LICENSE), tagged source | Published |
| Documentation and usage guidelines | [Usage](usage.md), [methodology](methodology.md), [limitations](limitations.md) | Included |
| Architectural documentation | [Architecture](architecture.md), [release operations](releases.md) | Included |
| Final documentation package | [Documentation index](README.md), downloadable supplement and original release archives | Included; site publication tracked by the Docs workflow |
| Public ecosystem report | [Registry inventory and technical findings](reports/2026-09-08/README.md) | Registry-specific contribution; no adoption measurement |
| Fellowship outcomes and recommendations | [Outcomes](fellowship-outcomes.md), [original report](fellowship-report.md) | Repository-specific assessment |
| Public dashboard deployment | Separate dashboard repository | Outside this repository's scope |

This mapping covers this repository only. It does not claim completion of deliverables belonging to other projects or measured program-wide outcomes.

## Distribution evidence

The [publication record](reports/2026-09-08/publication.json) lists the public URLs, release commit, workflow result and SHA-256 values captured from GitHub and crates.io. The five original archives contain four native executables and one standalone data package. Their published `SHA256SUMS` file remains unchanged.

The [validation record](validation.md) explains which checks passed and what they establish.

## Documentation supplement

The expanded website and report supplement the existing beta. They do not change the executable version, move its tag or replace original release assets. Download the [documentation archive](downloads/stylus-registry-docs-v0.1.0-beta.2.tar.gz) and its [checksum](downloads/SHA256SUMS). `BUILD-INFO.json` records the documentation source commit separately from the beta source commit.

The Docs workflow validates and publishes the site from `main`. The same documentation archive can be attached as an additional asset to the existing beta release, with its documentation commit in the filename; see [publishing](publishing.md).
