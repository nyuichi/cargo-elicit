# Validation

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Regular tests cover native command execution, shared command deduplication,
per-goal overrides, durable logs, partial reuse, missing/corrupt receipts,
source/command/external-input invalidation, input races, nonzero exits, timeouts,
missing executables, pure saved reads and dependency planning. Analysis-cache tests cover Lean-only edits, changed Rust call targets, path
dependencies, features, workspace-glob additions, macro/build-script fallbacks,
nonstandard Rust module paths, corrupt/deleted caches and sources that need no
language-server queries. Structural graph
and fingerprint tests exercise advisory analysis and freshness independently
from mathematical proof admission, which no longer exists. Git-checkpoint tests
commit the actual records, clone into another directory without logs/caches,
recover historical success, require a new environment check, and then reuse it.
They also cover deleted snapshots, separate source/environment changes, immutable
records, tracked/ignored file boundaries and content-addressed record corruption.

One opt-in test uses real Lean, without any AI or full-crate authoring run:

```sh
cargo test --locked --test native native_lean_policy_is_owned_by_the_configured_command -- --ignored --nocapture
```

It confirms that ordinary Lean command success is recorded even with an unrelated
proposition, `sorry` and a custom axiom; native compilation errors still fail.
It also checks receipt reuse. Never rebuild the CLI while a native check is
observing the executable's input identity.

Historical benchmark scripts/reports use frozen older interfaces and are not
current product tests. These software checks do not measure authoring speed.
