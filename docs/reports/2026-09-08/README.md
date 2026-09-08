# Registry coverage report · September 2026

**Snapshot:** 8 September 2026 · **Tool:** `v0.1.0-beta.2` · **Source:** `59460476068542e5cb9e1fef88a2c51f3a2b7003`.

This report describes the registry's curated inventory and two observed CLI configurations. Its counts describe this versioned dataset, not all Rust crates or contracts deployed on-chain.

## Findings

| Measure | Count | Meaning |
|---|---:|---|
| Unique crate names | 54 | Distinct names across both source files; no duplicates in this snapshot |
| Compatible-file records | 25 | The source file's classification, not verified deployments |
| Incompatible-file records | 29 | The source file's classification, not failures of every possible configuration |
| `requires_std = true` | 24 | Curated std assertions |
| `has_float = true` | 9 | Curated float assertions |
| `has_async = true` | 9 | Curated async assertions |
| Records with notes | 54 | Nonempty explanatory text; not necessarily reproducible evidence |
| Records with an alternative | 6 | Stored suggestions, not automated CLI advice |
| Records with `max_version` | 0 | No populated version ceiling; the beta does not enforce this field anyway |

Flags overlap. Their counts cannot be added to derive an incompatible total. There is no basis here for calculating coverage of the Rust or Stylus ecosystem.

## What the inventory tells us

The data provides an inspectable starting point for dependency review. Every record has notes, but feature conditions remain prose and lookup is by name. Registry entries need reproducible version/feature cases to make their conditions testable.

The six stored alternatives show available curation work that is not yet exposed as automatic recommendations. Implementing that feature needs a clear distinction between a suggested library and a tested replacement.

The TOML headers contain historical assertions about verification. This report preserves the source verbatim in its snapshot but does not adopt those assertions as independent evidence of contract deployment.

## Observed configuration example

The [captured CLI reports](examples.json) were produced using the Linux executable downloaded from the public beta release, with the beta's data and the toolchain recorded in that file.

```sh
stylus-registry check tiny-keccak --version 2.0.2 --features keccak --data-dir data/ --json
stylus-registry check tiny-keccak --version 2.0.2 --data-dir data/ --json
```

The first configuration passed the real WASM compilation check. The second produced a compiler error requiring a hash function feature. Both commands returned process status 0 because the single-crate command reports check errors inside its JSON. This is why the [methodology](../../methodology.md) asks readers to inspect messages and severities.

This pair demonstrates configuration sensitivity for one dependency. It cannot establish broader compatibility rates. The tool's version argument is a Cargo-compatible requirement; these commands do not independently lock every dependency resolution.

## Reproduce the inventory offline

From a checkout containing this documentation supplement, run:

```sh
python3 scripts/registry_report.py --check
```

Python 3.12 and its standard library are sufficient. The analyzer checks each embedded TOML source hash, rejects duplicate names and reproduces `metrics.json` byte for byte. It reads the frozen snapshot, not changing data on `main`.

The snapshot SHA-256 is `3a23f043054537df3be4f29c20227d981037f8708f8f9ba351ea156143d0988b`. The source data can also be compared with `git show v0.1.0-beta.2:data/known-compatible.toml` and its incompatible counterpart.

## Download the evidence

- [Frozen snapshot with original TOML and source hashes](snapshot.json)
- [Reproduced inventory metrics and all crate names](metrics.json)
- [CLI commands, toolchain and reports](examples.json)
- [Public release, workflow and crates.io record](publication.json)

These records are dated observations. Registry changes and future downloads do not update this snapshot automatically. See [validation](../../validation.md) for the checks performed and [roadmap](../../roadmap.md) for recommended follow-up work.
