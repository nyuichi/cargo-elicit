# Implementation and current scope

## Architecture

`analysis` manages Cargo metadata and the rust-analyzer stdio LSP connection, using `syn` to structure the original sources. It passes the selected package, features, target, and verifier cfg to rust-analyzer, waits for the workspace to finish loading, and queries definitions, calls, and explicit re-exports. A run reuses the same LSP connection and updates analysis when inputs change.

IDs combine the package, module path, definition kind, name, and a hash of the impl header. They distinguish impl self types and traits; line numbers are used only to locate definitions. Deletions and renames with unclear correspondence preserve earlier work and require review. Reclassifying an existing target as an auxiliary definition to shrink the progress denominator is also rejected.

Analysis is not a complete extraction of rustc HIR. Unresolved local calls, macro-generated definitions, glob re-exports, and build.rs output are reported as incomplete analysis and prevent overall completion. Analysis applies the selected target/features and verifier cfg, but does not fully reproduce arbitrary attribute macros or custom cfg from build scripts. Completion state is not inherited when the identity of a public name is ambiguous. Custom compiler flags, Cargo configuration that changes cfg, attribute-macro output, and workspace feature unification that broadens the selected configuration also require review.

`model` separates targets from official verification results. `verified` is derived from whether the specification is current, known specification dependencies have been reviewed, and every required unit has a proof without restrictions or unproved assumptions for the same broad input fingerprint. Types and traits have no independent `verified` status. Circular unreviewed specification dependencies cannot justify a proof.

`store` updates a single schema-1 `state.toml` using locks, fsync, and atomic replacement. Writes are checked against a revision. Separate locks protect runs, workers, and official verification. Worker turns cannot start during official verification, and before/after input checks also detect external edits. This is not a security boundary against a worker that can rewrite files under the same OS account.

`scheduler` prioritizes review, normally starts with type and trait specifications, and processes call-graph SCCs in callee-first, stable-ID order. In verification-required mode, verification of specified groups takes priority over further specification work. Failed tasks are suppressed on identical inputs. Input changes do not reset cumulative attempt and no-progress limits; only an explicit `--retry` resets cumulative limits.

`agent` implements the Codex 0.159.x stdio app-server protocol, including initialization, thread start/resume, turn start/completed/interrupt, and token usage. It does not override the model, reasoning effort, permissions, or authentication. Long shared instructions are stored as thread developer instructions; each turn receives the task, targets, related specifications, and revision. If a thread is lost, a new thread continues from the ledger. Prompt Cache hits are not guaranteed.

## Verifier adapters

| Aspect | Kani 0.68.x | Creusot 0.14.x |
|---|---|---|
| Specification processing | `--only-codegen`, function-contracts | `--only coma`, erasure check |
| Official verification | cargo kani with fresh `--export-json` output | Fresh Coma generation, why3find prove |
| Results | Schema-1.0 harness metadata, execution counts, individual checks | Coma mapped to original source positions and function names, VCs and proof trees in `proof.json` |
| Full-contract completion scope | Independent primitive free functions in the same file, explicitly typed `::kani::any()` inputs, one call to the original function | Independent primitive free functions, closed contracts, standard Creusot models and transformations only |
| Retained as unsupported scope | Methods, generics, custom models, helpers, input restrictions, stubs, loop contracts, external dependencies, etc. | Methods, generics, custom models, helpers, project axioms/assumptions, custom drivers, etc. |

Full-contract completion requires at least one `ensures` clause and a mapping to actual proof obligations. Empty contracts, mere process success, zero obligations, and body omission through `trusted` do not count as proof success. Even an explicitly typed harness does not prove an entire generic API by verifying one type. The name `stub_verified` in Kani is not sufficient evidence of a proof.

For Creusot, the adapter cleans only the selected package in cargo-elicit's dedicated build cache, using the Rust toolchain specified by Creusot, before generating Coma. It does not clean the user's target directory. Why3find and Why3 use the standard Creusot installation. The backend, compiler, standard models, and solvers are recorded as the trusted base, and tracked files are included in fingerprints. If the Kani installation cannot be identified, successful results are not marked reusable.

Automatically expanding arbitrary type invariants and trait laws into obligations for all implementations is not implemented. The worker may add models, laws, preservation harnesses, and similar definitions to original sources, but unsupported properties must remain reported as issues. Syntax and type checking can establish that an expression is processable; this is distinct from proving that the implementation satisfies the property.

## Changes and reuse

A specification hash change returns the target's specification and directly dependent specifications to review. If B references A and C references B, reviewing B after an A change can update only A's referenced hash without regenerating C's specification, provided B's specification contents do not change. While B awaits review, C's proof completion is also withheld. Once dependencies are restored, existing proof results can be reused if the inputs still match.

To avoid missing undeclared dependency changes, verification fingerprints broadly include source, proofs/models, package and local-dependency files, workspace and ancestor manifests and lockfiles, Cargo configuration, the selected configuration, cargo-elicit's own executable, compiler flags, toolchains, verifiers, standard models, and other inputs. The ledger, runtime files, caches, `.git`, Why3find's cache, and the actual target directory returned by Cargo metadata are excluded. Notes alone do not change the inputs. Symlinks in source inputs produce an error because those inputs cannot be tracked reliably.

Results with insufficiently tracked inputs, such as build scripts or external models, are not reused as full-contract completion. Results do not claim coverage of all features or all targets.

Runtime body and signature changes are detected, and unapproved changes during worker execution block completion. Known outer contract attributes are distinguished from runtime changes. Conservative diff checks may classify ghost code or loop annotations inside a body as implementation changes; human review is required in that case. The mechanism does not decide semantic equivalence of arbitrary Rust code.

## Worker CLI

`agent show [ID]` returns the current revision. Fetch it again after editing sources.

- `agent submit ID --revision N --note REASON [--depends ID] [--unit UNIT]`: Submit specification work or review after syntax and type processing. Preserve known dependencies and update referenced hashes. The contract unit cannot be removed to shrink the denominator.
- `agent depend ID DEPENDENCY --revision N`: Add a known specification dependency and return the target to review.
- `agent issue ID --revision N --reason REASON`: Mark a specification as requiring review even before it is fixed. Preserve historical results.
- `agent verify [ID ...] --revision N`: Run the official verification wrapper only while the worker is idle. During a turn, finish preparation and hand control back to the scheduler.

Raw conversations, stdout, and stderr are not recorded in the ledger. Error summaries and typed verification information are saved, avoiding unconditional persistence of logs that might contain credentials. Run the verifier directly for detailed failure diagnosis. Artifacts produced by native verifiers are retained in the cache.
