# Validation and evidence

## Published executable

The [beta.2 release workflow](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/actions/runs/34268848187) completed successfully on 8 September 2026. Its source commit and public package checksums are captured in the [publication record](reports/2026-09-08/publication.json).

| Check | Evidence and result |
|---|---|
| Rust validation | Formatting, Clippy and the default Rust suite passed; the pre-release baseline was 55 passing tests with four ignored |
| Real WASM compilation | CI explicitly ran the two ignored compilation tests and native smoke checks |
| Publication preflight | Both packaged crates built; checksums and existing-version checks passed; nine Python tests passed |
| Native distribution | Linux x86-64, Windows x86-64 and both macOS architectures built and passed smoke/extracted-archive checks |
| Public Cargo installation | CI installed `stylus-registry 0.1.0-beta.2` from crates.io and smoke-tested that executable |
| Published downloads | All five original archive checksums matched; the downloaded Linux executable passed real WASM success/failure, feature, registry, JSON and strict-project smoke checks locally |

These checks validate implemented behavior and distribution. They do not establish compatibility for every crate or prove a deployed Stylus contract works.

## Documentation supplement

The Docs workflow runs `scripts/docs.py check` and `scripts/registry_report.py --check` before building the static site and downloadable archive. It checks local links, navigation/search coverage, embedded source hashes and exact inventory reproduction. PRs prepare a preview artifact; only `main` deploys to Pages.

The [example reports](reports/2026-09-08/examples.json) record two real runs of the published executable: a successful `tiny-keccak` configuration and its missing-feature compilation failure. The frozen inventory contains 54 unique names and reproduces the [reported metrics](reports/2026-09-08/metrics.json).

Browser verification covers page navigation, search, evidence downloads, the architecture diagram, light/dark appearance and mobile navigation. The source PR and Docs workflow record the validation of this documentation revision separately from the beta's executable tests.

## Reproduce locally

```sh
python3 scripts/registry_report.py --check
python3 scripts/docs.py check
python3 scripts/docs.py build
python3 -m http.server 8043 --directory target/docs-site
```

Open `http://localhost:8043`. The packaged Markdown and JSON remain readable without the viewer. Docsify and Mermaid load from pinned CDN URLs, so the interactive viewer needs network access and JavaScript.
