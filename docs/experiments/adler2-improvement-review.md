# Astra review: improvements suggested by the latency analysis

Status: design review only. No production changes, new authoring trials or speedup claims. The user requested preservation of the record and consultation with Astra after the [latency analysis](adler2-latency-analysis.md). Astra reviewed that analysis, the [complete change audit](adler2-change-audit.md), and the current worker, plan, scheduler and Aeneas adapter implementation.

The record supports several general design improvements. It does not establish how many minutes each would save. Continue to preserve original Rust, arbitrary-input contracts, explicit semantic approvals, kernel/transitive-axiom audits, partial results and resumability.

## 1. Test goal-level continuous work without requiring a return after every checkpoint

Current worker instructions tell the model to finish a turn after a useful checkpoint for official checking. Plans then map individual contract/lemma/integration steps into subsequent assignments. The latest product trajectory had fifteen normal worker completions versus one ordinary turn; it had no Plan tasks and no worker deadline cutoffs. The candidate issue is therefore repeated **normal checkpoint handoffs**, not forced deadline failure or loss of the resumed conversation.

A minimal comparison would change only the instruction that determines when to return: continue working through multiple supporting lemmas and the caller within the same goal; save useful intermediate files without necessarily ending the turn; return on goal completion, a concrete blocker, or the existing budget/deadline. Retain the same approval and final acceptance rules. Plans remain available as assistance instead of making fine subdivision a required deliverable. Do not simultaneously replace the scheduler, plan format and timing defaults, or the effect becomes impossible to isolate.

This is the largest plausible workflow opportunity because the observed increase inside workers was 47.29 minutes. That number is not an estimate of the benefit of changing this instruction: it mixes extra proof exploration, protocol operations, compiler calls and latency. A controlled comparison is needed before saying this change is faster. Intermediate status should still distinguish saved/uncollected work from current kernel-checked evidence.

Relevant code: src/agent.rs instructions and task payload; src/plan.rs next_root; src/cli.rs worker/checkpoint boundaries.

## 2. Generalize local Lean module handling, then separately design safe official reuse

The current adapter copies and builds a fixed Specs/Lemmas/Proofs arrangement. Task input filtering also names Lemmas.lean and Proofs.lean specifically. The product's completed update proof costs approximately 150 seconds to rebuild; adding the decoder to the same module repeats it. Small independent proofs should be able to live in separate tracked local modules, with their import dependencies managed consistently across local development, task inputs and official acceptance.

**Splitting files alone does not reduce current official verification cost.** Each official invocation uses a fresh cache and rebuilds its modules. Two distinct improvements must be reviewed separately:

1. Flexible local source modules and a dependency graph, allowing local incremental development and avoiding unnecessarily monolithic proof editing.
2. Official compiled-module reuse, only when provenance validates the original Rust/generated model, contracts, module source, transitive imports, toolchain/binary/configuration and compiled outputs. The existence of an old .olean is insufficient.

Retain an independent fresh rebuild for final acceptance/reproducibility checks. Do not copy arbitrary user-supplied compiled objects into trusted evidence. Tiny independent-goal fixtures can first verify that unrelated edits do not rebuild a local peer, while referenced model/import/tool changes invalidate all affected evidence. That is a mechanism check, not an authoring speed benchmark.

Relevant code: src/aeneas.rs lean_command and execute; src/plan.rs task_input; src/fingerprint.rs import/input tracking. There is no Adler-specific proof or tactic recommendation here.

## 3. Measure protocol costs explicitly and make resumed usage accounting robust

The disjoint timing analysis separates outside-worker time from command-active and other worker time, but it cannot classify all worker activity into proof search versus protocol administration. Do not label the entire 28.02-minute no-command increase as wasted planning, or the entire 13.70-minute outside-worker interval as a single checker cost.

Record observable reasons and phases: task return reason, official verification trigger, changed inputs, CLI operation kind and reconciliation duration. Keep the interval accounting non-overlapping and avoid collecting private reasoning content. Use those measurements to target actual repeated operations rather than infer them from task names.

The cumulative-token mistake should also be prevented in general tooling: identify threads, retain cumulative totals separately from segment increments and do not sum a resumed thread's counters twice. The preserved initial-final/resume-first equality is a concrete regression fixture. A tiny synthetic resumed-stream case can verify this without another AI trial.

Even eliminating all observed outside-worker overhead leaves about 75.66 minutes in this trajectory. Checker optimization by itself cannot close the recorded gap to 28.40 minutes.

## 4. Remove experiment coupling and report selected-goal completion separately

- Replace the common prompt's literal BENCHMARK_TASK.md handling with ordinary explicit user requirements. This is cleanup of actual experiment-to-product coupling, not a speed result.
- Review the PROGRESS.md / verification-inventory.json special cases as an input/output classification design. Do not simply ignore every file called a report: compiler/proof file reads can make it a semantic input, so conservative fallback remains necessary.
- Expose selected_goals_verified separately from inventory_complete. Current selected proofs can all be accepted while conservative excluded-derive discovery keeps complete=false and exit 1. Clear reporting can resolve that ambiguity without whitelisting derive names or pretending whole-crate coverage. Any change in exit-code semantics needs a separately stated specification.

These changes improve generality and clarity; there is no measured speed benefit to promise for them.

## Recommended sequence and validation boundaries

Astra recommends cheap measurement/coupling cleanup, then general module handling, followed by a narrowly controlled goal-level workflow comparison. Treat the goal-level instruction as the leading performance hypothesis, not an already established fix. Keep each mechanism independently reviewable; official cache reuse is a later, distinct correctness-sensitive step.

Use small, unrelated fixtures for the underlying mechanisms. For actual authoring performance, use previously unused problems with different proof shapes and identical contracts/settings/acceptance rules. Do not continue selecting instructions or defaults solely on Adler. Test one proposed workflow factor at a time. No new experiment was started as part of this review.

## What Astra recommends keeping unchanged

- The original Rust implementation and required arbitrary-input semantics.
- Approval identity checks and exact theorem/contract correspondence.
- Lean kernel and permitted transitive-axiom checks.
- Preservation and resumption of partial work.

Do not increase timeouts/retry limits/finalization margins or complicate proposition credits merely because this trial was slow. Do not add Adler-specific mathematics, invariants or tactic strategies to the product. The previous fixes established better preservation and continuation, not more efficient proof discovery.
