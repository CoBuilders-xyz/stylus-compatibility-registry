# stylus-compat-core

Experimental Rust library behind the Stylus Crate Compatibility Registry, an educational project from the CoBuilders Stylus Fellowship.

It parses Cargo manifests and dependency trees, loads a curated TOML registry, runs five compatibility heuristics, and produces serializable reports and scores.

```rust,no_run
use std::path::Path;

let report = stylus_compat_core::analyze_project(
    Path::new("Cargo.toml"),
    Some(Path::new("data")),
)?;
println!("{}", report.overall_score);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Checks may invoke Cargo and download crate source. A score of 100 is not proof that a contract can activate or execute on Stylus. Registry flags are curated assumptions, not an on-chain verification result.

- [Usage and installation](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/docs/usage.md)
- [Architecture](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/docs/architecture.md)
- [Known limitations](https://github.com/CoBuilders-xyz/stylus-compatibility-registry/blob/main/docs/limitations.md)
- [Repository and registry data](https://github.com/CoBuilders-xyz/stylus-compatibility-registry)

License: MIT.
