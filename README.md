# cargo-elicit

A Rust CLI for people and external AI tools to organize verification work, inspect obligations, and preserve evidence tied to the original crate.

**cargo-elicit never starts an AI.** It does not choose proof strategies, dispatch tasks, generate helpers, or require submission/checkpoint protocols. Use your editor, native verifier and preferred assistant normally.

## Workflow

```sh
cargo install --git https://github.com/nyuichi/cargo-elicit
cd your-crate
cargo elicit init --tool aeneas --api identity
cargo elicit list
cargo elicit show identity
cargo elicit plan identity
# Write contracts and proofs using normal native tools.
cargo elicit check
cargo elicit audit
cargo elicit status
```

Rust 1.86+ builds the CLI. Explicit discovery requires Cargo, rustc and rust-analyzer; install the chosen verifier for checks. [Aeneas/Lean setup and proof format](docs/aeneas.md) describes the pinned toolchain. Kani 0.68 and Creusot 0.14 retain their native evidence adapters and narrower supported coverage.

| Command | Behavior |
| --- | --- |
| `init` | Create configuration and explicitly discover the original Rust inventory |
| `notes` | Create/locate `.elicit/notes/` for progress, handoff and final reports outside verification inputs |
| `refresh` | Observe current scope, sources, declarations and tools; no proof compilation |
| `plan [API ...]` | Show reachable call graph collapsed into dependency-first SCCs, shared callees, contracts, evidence and unresolved boundaries |
| `status [--all]` | Show remaining work and a short candidate-SCC summary; hide audited and semantically reviewed items unless `--all` |
| `list`, `show` | Read all saved obligations or details; no processes, hashing, reconciliation or writes |
| `check [ID ...]` | Refresh inputs and validate obligations; Aeneas reuses matching checker-owned Lean products |
| `audit [ID ...]` | Refresh inputs, independently rebuild in a fresh workspace and audit exact proof coverage |
| `review [ID ...] --note REASON` | Record semantic judgment, separately from machine evidence |

[Dependency-first planning](docs/planning.md) explains candidate SCCs, caller requirements, proof reuse and the distinction between call edges and proof dependencies.

Read commands explicitly identify their snapshot and observation time. **They do not assert that files or tools still match it.** Use `refresh`, `check` or `audit` to observe later changes. Successful incremental checks and fresh audits are separate columns; semantic review does not gate either.

For frequent progress reads, `status` focuses on remaining work. A goal is hidden only when its saved fresh audit and semantic review are accepted and there is no current failed incremental check. Completed counts remain visible; `status --all` includes every goal. JSON applies the same filter and includes total/completed/remaining counts. Refreshed stale evidence or review makes a goal visible again. `list` and `plan` retain completed items so dependencies remain understandable.

## Declare what you want established

Edit `elicit.toml`; no revision tokens or submission commands are needed:

```toml
schema = 2

[verification]
tool = "aeneas"

[package]
features = []
all_features = false
no_default_features = false

[[obligations]]
id = "identity"
target = "identity"
description = "Returns its argument for every input."

[[obligations]]
id = "supporting-fact"
statement = "True"
theorem = "Proofs.fact"
description = "Replace this example with the mathematical fact you need."
```

A target selector may be an exact inventory ID, a unique function name or a method alias. A proposition obligation instead names an exact Lean statement and theorem. The example `True` is only a format demonstration, not a useful verification goal. Formal API predicates remain in native contract files. See [architecture and guarantees](docs/implementation.md).

Omitting declarations selects discovered public function/method/trait-implementation obligations. Unsupported targets remain explicit; they do not become proved because another method passes. `list --inventory` also shows types, private helpers and discovery limitations. Passing every declared obligation does not establish unspecified invariants, trait laws, other features or whole-package correctness.

`--package`, `--features`, `--all-features`, `--no-default-features` and `--target` select build scope at initialization. Edit the package section and refresh when changing scope. In a multi-package workspace, choose one package explicitly.

## Evidence and review

A backend exit code, an author's claim, and an accepted review cannot create a proof. API evidence requires original-source correspondence, complete native contract coverage and the adapter's scope/trust checks. Aeneas checks the actual translated signature and transitive axioms; only `propext`, `Classical.choice` and `Quot.sound` are allowed. Arbitrary proposition evidence is distinguished from Rust API coverage.

```sh
cargo elicit review identity --note 'The predicate captures the intended arbitrary-input behavior.'
cargo elicit review identity --verdict needs-work --note 'The specification needs clarification.'
```

Review binds to the specification and its human description. Changing editorial description can stale review without discarding a machine proof. Changing a contract/model or formal declaration invalidates the corresponding bindings. A proof-only edit invalidates input freshness until explicitly checked. No command edits the Rust implementation or approves an implementation change.

The versioned snapshot is `.elicit/snapshot.json`. Durable receipts, phase logs and diagnostic sources are under `.elicit/evidence/`; paths in receipts are project-relative. `.elicit/cache/` and `.elicit/runtime/` are regenerable and should be ignored by Git. Cache deletion preserves saved evidence and review summaries. Evidence is an audit trail, not a tamper-proof store against someone who can rewrite it under the same OS account.

Read commands exit 0 on a successful read. `check`/`audit` exit 0 for successful requested evidence at that command's level, 1 for unresolved work, 2 for configuration/environment errors, and 130 for interruption. `--json` exposes the same saved facts.

## Use with an AI assistant

The optional [cargo-elicit skill](skills/cargo-elicit/SKILL.md) teaches external assistants these commands. It leaves planning and authoring to the assistant. Humans need no skill or AI credentials. There is no `run` command or built-in Codex integration.

This is an incompatible 0.2 redesign. Old ledgers/configuration are not migrated. Historical experiments are preserved in [the experiment index](docs/experiments/README.md); they describe older products and do not demonstrate that this version speeds up proof authoring.

Licensed under MIT OR Apache-2.0.

## Operational records and final audit

Use `cargo elicit notes` to create/locate `.elicit/notes/` (also created by `init`). Keep progress, handoff notes and final reports there. `.elicit/` was already excluded from fingerprints; this is an explicit workflow convention, not a broader exclusion rule. Other project files, including arbitrary Markdown and root-level reports, stay tracked.

Never import or read operational notes from Rust, build scripts, proc macros, Lean or verifier code. Such dependencies are outside the reproducibility guarantee. Keep all specifications, models and proofs in tracked input locations. Archive notes explicitly when publishing/moving work: `.elicit` is normally Git-ignored.

Save semantic inputs and reproduction scripts, run the final `audit`, then write the report in `.elicit/notes/`. Notes alone do not require another audit. After `refresh`, `status --all` identifies the current audit receipt and, for stale evidence, compares saved audit input hashes with the newly observed input hashes. `--json` exposes `audit_validity`, changed paths, formal-binding changes and whether a fresh audit is needed at the saved snapshot. Reasons survive repeated refreshes; older evidence without saved input hashes reports that details are unavailable. A plain `status` does not inspect current files. Explicit `audit` continues to rebuild from scratch every time.
