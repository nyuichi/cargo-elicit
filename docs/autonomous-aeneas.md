> Historical design document. Superseded by the native-command interface in README.md; not current CLI behavior.

> Historical v0.1 document. The incompatible v0.2 product is documented in [README](../README.md) and [architecture](implementation.md). This page does not describe the current CLI.

# Autonomous Aeneas authoring and artifact collection

The default Aeneas workflow gives one Codex author the entire declared goal set. The author chooses proof order, helper lemmas, and local Lean modules. cargo-elicit manages stable artifact acceptance, semantic review identities, current evidence, and recovery. It does not require a plan, a return after each lemma, or a separate submission and syntax pipeline for each API. Kani and Creusot retain the planned workflow.

```sh
cargo elicit init --tool aeneas --api identity --api other
# Set verification.required = true in elicit.toml for proof completion.
cargo elicit run --max-seconds 3600
cargo elicit status --json
```

The whole-package goal remains available by omitting API selectors or explicitly using `init --whole-package`. It includes unresolved type/trait obligations and semantic-analysis limitations. Selected-goal completion requires current reviewed contracts and full proofs for the selected APIs; unrelated discovery/inventory limitations remain visible in `inventory_complete`. This exception applies only to Aeneas verification-required mode; scope/configuration issues and unknown analysis issues still block completion. Removed or changed original runtime APIs still block completion. `selected_goals_verified` reports proof coverage separately from that broader completion decision.

## Review once, collect stable files

Saved proof sources are not approval or proof evidence. Before returning, the author explicitly reviews the meaning of the contracts:

```sh
cargo elicit agent stage --all --note "These postconditions express the requested arbitrary-input behavior"
# Or supply exact ledger IDs, including non-function targets when reviewing them:
cargo elicit agent stage 'my-crate::identity::Function' --note "Reviewed identity semantics"
```

`--all` stages supported contracts declared in `contracts.toml`; it does not approve every type or unsupported trait in the inventory. Staging checks identities and saves intent, without running a formal pipeline per API. The receipt binds the original runtime/configuration/tool context and each contract/model identity. Proof terms may change afterward; changing a contract, its model, or runtime context requires a new explicit review. No inference from a proof's existence substitutes for this review. The reviewer remains responsible for specification adequacy.

After the worker stops, the collector runs one fresh pipeline for the staged contracts and existing reviewed declarations needing evidence. It validates the exact translated Rust correspondence, contract types, kernel proofs and transitive axiom allowlist. Only individually mapped and successfully elaborated contracts receive the recorded review; unprocessed staged contracts remain pending. only valid unrestricted full-contract proofs receive verified evidence. A well-typed contract may be reviewed while its proof remains unresolved. Known specification dependencies are retained and rebound during joint review; required contract units are preserved. Environment errors, interruptions, changed inputs and stale reviews do not silently become accepted proofs.

Saved work can be accepted without any AI worker:

```sh
cargo elicit collect --seconds 600
cargo elicit status --json
```

Repeated collection reuses current successful evidence and retries unsuccessful checks. `collect` requires the worker to be idle and uses the same completion exit codes as `run`. It rejects a stale review until explicitly restaged, preserving the files. JSON status distinguishes saved Lean module count and pending/stale/invalid semantic review from current checked results. Status reconciliation does not silently compile or approve saved proofs; it is safe to inspect during authoring.

## Local modules

Keep persistent UTF-8 `.lean` sources under `elicit-proofs/aeneas`. `Specs.lean` may import local model modules; `Proofs.lean` is the proof entry point and may import completed proof modules. Ordinary single-line import headers, nested directories and comments are supported. For example:

```text
Specs.lean             imports Models.Values
Models/Values.lean     imports Aeneas
Proofs.lean            imports Api.Identity and Api.Other
Api/Identity.lean      imports ElicitContracts
Api/Other.lean         imports ElicitContracts
```

The collector copies tracked sources into a fresh generated project and compiles imports in dependency order. Missing local sources, cycles, symlinks and shadowing generated/toolchain modules are rejected. Names currently use ASCII identifiers. Do not import local compiled objects or put unfinished Lean experiments in this tree; use `.elicit/cache`. Imported model sources participate in semantic review identity; all proof sources participate in evidence freshness. Custom model graphs use conservative source identity rather than claiming fine-grained elaborated dependency reuse.

This change does not introduce cross-invocation `.olean` reuse. Each official invocation still freshly extracts, translates and checks the proof graph. Splitting modules provides ordinary Lean organization, without promising a compiler speedup.

## Budgets, recovery and compatibility

The autonomous author uses the remaining overall authoring budget, reserving finalization time when verification is required. The per-task and short-plan deadlines apply only to `--workflow planned`. Verification uses the remaining overall budget capped by `--verify-seconds`. A task deadline stops the author and its process group before collection; external interruption preserves active work and returns 130 without accepting new evidence. The next invocation reconciles saved files and resumes without requiring `--retry` solely because of interruption/deadline exhaustion.

Completed failed attempts remain bounded by cumulative attempt/no-progress limits. After changing approach, `--retry` explicitly resets these. Cumulative token counters are stored per unique thread and merged by maxima, so resumed thread totals are not added twice. The existing current-thread counters remain available for old ledgers.

Use `cargo elicit run --workflow planned` for the previous contract/lemma/integration dispatcher and its optional checked plan checkpoints. Its documentation is in [goal-directed verification](goal-directed-aeneas.md).

No end-to-end authoring speedup is claimed from this implementation alone. Fixed-artifact acceptance measurements bound added checking cost; fresh matched authoring experiments are a separate step.
