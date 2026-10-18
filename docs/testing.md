# Validation

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo +1.86 check --locked
```

Regular tests cover snapshot read purity with absent tools and changed files, explicit refresh without proof compilation, independent review/evidence, incremental dependency reuse and corruption rejection, clean audit isolation, unsupported import rejection, proposition goals and configuration races. Deterministic verifier peers test command/evidence boundaries; they do not establish native kernel acceptance.

Opt-in native tests use real pinned verifiers and never invoke AI:

```sh
cargo test --locked --test native aeneas_incremental_audit_and_independent_semantic_review -- --ignored --nocapture
cargo test --locked --test native kani_original_contract_audit -- --ignored --nocapture
cargo test --locked --test native creusot_original_contract_audit -- --ignored --nocapture
```

The Lean test exercises actual API and proposition proofs, warm module reuse, proof/model edits, independent semantic review, project-axiom rejection, partial acceptance of an independent proposition, fresh audit and durable diagnostics after cache deletion. It prints its retained fixture location. Never rebuild the cargo-elicit executable while a native test is checking its tracked identity.

Historical experiment scripts under experiments/adler2 target v0.1. They remain preserved evidence rather than current product tests. A new authoring speedup is not established by these mechanism tests.
