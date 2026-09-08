# Releasing the registry

## Distribution channels

A release publishes two Cargo packages and a GitHub release:

1. `stylus-compat-core` on crates.io.
2. `stylus-registry` on crates.io, with an exact dependency on the matching core version.
3. Native archives for Linux x86-64, macOS Intel, macOS Apple Silicon and Windows x86-64.
4. A standalone registry-data archive, documentation, release notes and `SHA256SUMS`.

`v0.1.0-beta.1` was a failed publication attempt: no crates or GitHub release were published. Its tag is preserved.

The CLI's first public version is `0.1.0-beta.2`; its tag is `v0.1.0-beta.2`. Prerelease tags produce GitHub prereleases. Rust crate versions are immutable once published; release versions and tags must not be reused for different source.

Native artifacts are built on Ubuntu 22.04, macOS 15 Intel/ARM and Windows Server 2022. They are not code-signed or notarized by this project. Use a source installation if local platform policy requires it.

## One-time crates.io setup

A maintainer needs a crates.io account with a verified email and permission to publish both names. For the initial publication, create an API token that permits publishing `stylus-compat-core` and `stylus-registry`, including creating these crates. Store it as the repository Actions secret `CARGO_REGISTRY_TOKEN`.

Configure the secret through GitHub repository settings or `gh secret set CARGO_REGISTRY_TOKEN`. Never commit the token. A local `.env` is ignored by Git and is not read by release scripts or included in archives. Rotate credentials through the same secret without changing code.

The workflow uses the normal GitHub Actions token to create the GitHub release; it does not require a separate personal GitHub token. After bootstrapping, maintainers can migrate to [crates.io Trusted Publishing](https://crates.io/docs/trusted-publishing), configuring both crates and updating the auth step. The beta workflow uses the repository secret so the first publication works too.

## Prepare the next release

1. Update `[workspace.package].version` in `Cargo.toml`, for example to `0.1.0-beta.3`.
2. Update `[workspace.dependencies].stylus-compat-core.version` to the same exact version, e.g. `=0.1.0-beta.3`.
3. Run `cargo check --workspace` to update the workspace entries in `Cargo.lock`.
4. Add the matching `## 0.1.0-beta.3` entry in `CHANGELOG.md`. Update versioned installation examples and links in the READMEs, usage guide and example workflow.
5. Run the validation below, open a PR and merge it into `main` after the checks pass.

```sh
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
cargo test --locked -p stylus-compat-core compile_check_ -- --ignored
python3 scripts/publish_crates.py --dry-run
python3 -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/release.py validate --tag v0.1.0-beta.3
```

The publication preflight requires a clean checkout. For package-only verification before committing, use `cargo package --workspace --locked --allow-dirty`.

## Trigger publication

After the preparation PR is merged, update local `main` and tag that commit:

```sh
git switch main
git pull --ff-only origin main
git tag -a v0.1.0-beta.3 -m 'Stylus registry 0.1.0-beta.3'
git push origin v0.1.0-beta.3
```

Creating the tag is the deliberate version-selection step. Everything after the tag push is performed by [`.github/workflows/release.yml`](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/.github/workflows/release.yml):

- Validate the tag, workspace/core versions, lockfile and changelog. The tagged commit must belong to `origin/main`.
- Run formatting, Clippy, Rust tests, real WASM compilation tests, release-script tests and the publication script in dry-run mode, including package creation and existing-version checksum checks.
- Build each native target, run smoke checks that require actual WASM compilation, then exercise the executable and registry from its extracted archive.
- Publish the core crate before the CLI. Verify `cargo install stylus-registry --version <version> --locked` from crates.io and smoke-test that installed binary.
- Require all four native archives and the registry archive, calculate checksums, and upload the complete set to a draft GitHub release before publishing it.

Pull requests touching release inputs run `scripts/publish_crates.py --prepare-only`: packages are built and their archive checksums read, without comparing PR commit metadata against an already published version. All four platforms are exercised without a publishing secret or uploads. Tagged releases use `--dry-run` and the publication step to enforce existing-version checksum identity. The normal CI workflow also explicitly runs real WASM checks; a blocklist fallback cannot satisfy the smoke test.

The stable Rust channel and the Cargo lockfile are used for the beta. `BUILD-INFO.json` records the exact compiler, source commit, version and native target in each archive. The process does not claim bit-for-bit reproducibility across different compiler versions.

## Inspect and retry

Inspect the **Release** workflow in GitHub Actions. If a transient failure occurs, rerun failed jobs. You can also run the workflow manually with its existing `tag` input; this does not create a new tag or skip validation.

A partial crates.io publication is resumed only if an existing version has the same checksum as the locally prepared `.crate` file. Different bytes fail explicitly. If source or packaging must change after a crate was published, bump the version; do not move its tag.

A draft GitHub release can have assets replaced during a retry. Once public, the workflow refuses to overwrite its assets. Use a new release for fixes. A missing token or failed platform build leaves publication incomplete rather than silently omitting a promised distribution channel.

After a successful run, check the release page, install with Cargo in a clean location, and download an archive to confirm its checksum. The workflow automates the installation and archive checks, while the release page remains the public evidence for the fellowship deliverable.

## Relevant documentation

- [Cargo publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Cargo installation and version selection](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository)
- [Known beta limitations](limitations.md)
