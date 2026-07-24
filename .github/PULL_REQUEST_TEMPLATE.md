## What

<!-- Brief description of the change -->

## Why

<!-- Link to the issue this PR addresses -->

Closes #

## How to Test

<!-- Steps for reviewers to verify this works -->

1. `cargo build`
2. `cargo test`
3. `cargo run -- check-deps --manifest fixtures/test-project/Cargo.toml --data-dir data/`
4. ...

## Checklist

- [ ] Code passes `cargo fmt --check`
- [ ] Code passes `cargo clippy -- -D warnings`
- [ ] Tests pass `cargo test`
- [ ] Project builds `cargo build`
