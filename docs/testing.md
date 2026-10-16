# Testing

The regular `cargo test --locked` suite covers the ledger, stale propagation, coverage, retries, SCC ordering, locks, revision conflicts, CLI regressions using actual Cargo and rust-analyzer, and result parsing using actual Kani 0.68.0 JSON output. On Unix, process termination tests check timeouts for child processes, including children in separate process groups on Linux. CLI tests require Python 3.

`tests/support/codex.py` simulates the stdio app-server and calls the actual cargo-elicit agent CLI. It reproduces completion notifications without accepted work, failed turn status, unattended approval requests, body edits, interruption, and thread resumption. `tests/support/cargo.py` replaces verifier commands and delegates ordinary Cargo operations, such as metadata, to actual Cargo. The product has no test-only environment variables or fake success paths.

Native tests are ignored by default. They run actual verifiers through the same wrapper used in the product. The Codex side uses a deterministic protocol peer; these tests do not assess AI generation quality or execute live AI turns.

```sh
cargo test --release --locked --test native -- --ignored --test-threads=1
```

Each test copies the entire fixture package into a temporary directory and verifies its original functions. It does not duplicate implementation functions into a separate crate for verification. Tests cover interruption and resumption, specification mode without proof search, verification-required mode, and reuse of proof results after deleting the cache and running `init` again. Temporary directories are retained on failure for diagnosis.

During reconstruction, all 39 regular tests and both native tests passed on Linux x86_64. Formatting, Clippy, and the Rust 1.86 build check also passed.

Tool versions used during reconstruction:

- Rust stable 1.99.1 / MSRV 1.86.0
- Codex app-server schema 0.159.2
- Kani 0.68.0 / CBMC 6.11.0 / nightly-2026-08-21
- Creusot 0.14.0 / nightly-2026-08-03
- Why3 `05bf4ddf55cb3042f34c3bc0ed3aff56a226978a` (1.8.2+git)
- Why3find `0f054b93c86ac7ba20bca7df5d1563a9a4434a30` (1.3.0+dev)
- Alt-Ergo 2.6.4

CI runs formatting, Clippy, regular tests, and the MSRV check. It does not install native verifier toolchains or execute live AI turns. Consult the repository's CI status and commit validation records for execution results.
