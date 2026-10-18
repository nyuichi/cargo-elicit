# Change and benchmark audit after the Adler-32 iterations

Reviewed state: public commit `61c5a2ec6a5c765a35d8c9a8d7a15c65ccfbb940`, matching local tree `31a1d644cd28b2687dbe9c44857651d4e3f42bf7`. This review changes no production code, prompts, defaults or proof artifacts. It inventories all 26 existing commits, with the main behavioral comparison starting at the initial Aeneas implementation (`8c52919` locally). The machine-readable [complete commit/path inventory](adler2-change-audit/change-inventory.json) includes every changed path, including evidence and validation files, rather than treating evidence commits as product changes.

## Findings

The ordinary-arm 28-minute result is a real recorded authoring trial with valid saved proofs: **1,704.225159 seconds, or 28 minutes 24.225 seconds**. Its contemporaneous independent audit accepted all five selected API goals. This review freshly re-extracted the original Rust model, rebuilt the saved proof modules and checked all 52 recorded theorem roots again; all passed. That fresh re-audit took 123.20 seconds. It checks the saved result's correctness, not whether a new model run would finish in the same time.

There is **no observed Adler-specific answer or dispatch branch in production code**: a search of `src/` and Cargo configuration found no adler/checksum names or constants 65521, 5552 or 22208. Original runtime sources were unchanged. However, **absence of those strings does not establish absence of overfitting**. Product instructions, report-file handling and scheduling policies were repeatedly informed by one benchmark. Several concrete experiment couplings remain, and there is no held-out multi-crate authoring evaluation demonstrating transfer.

The final product example completed the same five mathematical goals in 89.36 minutes across two segments, 91.66 minutes including resumption preparation. That is shorter than the user's reported approximately three-hour reference, but longer than this retained ordinary trial. The current record establishes successful completion and repaired bookkeeping; it does not establish a general speed or efficiency benefit.

## What the 28-minute measurement actually covers

Evidence: [metrics](../../experiments/adler2/results/2026-10-17-goal-planning/without/metrics.json), [model/settings and usage](../../experiments/adler2/results/2026-10-17-goal-planning/without/telemetry-summary.json), [original audit](../../experiments/adler2/results/2026-10-17-goal-planning/without/audit/summary.json), [fresh re-audit](../../experiments/adler2/results/2026-10-18-change-audit/summary.json) and [measurement cross-check](../../experiments/adler2/results/2026-10-18-change-audit/measurement-recheck.json).

| Question | Confirmed result |
| --- | --- |
| Recorded elapsed time | 1,704.225159 seconds end to end; work 1,704.223989 seconds |
| Stop condition | One completed model turn plus the five-API completion marker; the marker alone is not acceptance |
| Actual event chronology | Local preserved telemetry starts 0.119 seconds after the measured start and ends at 1,702.468 seconds; it contains one completed turn |
| Model and configuration | Codex 0.159.2, gpt-6.1-sol/OpenAI, unspecified reasoning effort/tier, workspace-write, network enabled, on-request approvals |
| Recorded usage | 28,327,415 input tokens, including the 28,067,584 cached subset; 76,504 output tokens; 204 command items |
| Goal | Five named APIs of adler2 2.0.1, not every crate API or every possible platform |
| Mathematical extent | Original optimized computation, arbitrary representable states/slices, successful execution, empty-input normalization, original machine arithmetic and indexing |
| Extra results | Codec roundtrips, split/update and save/restore laws; concatenated-slice representability is explicit in secondary laws |
| Preservation | Original Rust and Cargo hashes match; the initial prepared Git tree contains no saved proof directory |
| Original evidence integrity | All 262 files in the published SHA-256 manifest still match |
| Fresh audit in this review | Generated model byte-identical; 1 generated module + 10 proof modules rebuild; 52/52 theorem roots pass the permitted transitive-axiom checks |
| Outside that measured interval | Tool installation, prepared standard models/Mathlib, environment/build preparation and the independent audit; the original audit took 123.05 seconds |

This is verification authoring time, not the runtime performance of the checksum crate. Cached model inputs are an accumulated usage subset, not evidence that earlier solutions were copied. Fresh initial cases and the common no-solution/no-hint rules are documented; the whitelisted public telemetry is not a full file-access trace and cannot prove every access the worker did not make. No new authoring run was performed for this review, so reproducibility of the 28-minute timing remains unmeasured.

The ordinary arm is “without cargo elicit”, not without any experiment runner: `baseline.py` launches Codex app-server, handles completion and records aggregate telemetry. Both arms share the mathematical task. Ordinary Codex uses selected-root extraction, omitting unrelated I/O; the product extracts more broadly and additionally maintains approvals, plans and ledger evidence. These differences are real overheads/boundaries, not a reduction of the five selected mathematical obligations.

## Complete behavioral change inventory

The following covers all production files changed after initial Aeneas support. The source delta is 14 production source files, 2,472 additions and 282 removals; 5 test/support files add another 1,939 lines and remove 11. Counts are Git textual diffs, not a measure of value. Cargo dependency versions were not changed during the Adler-specific iteration series.

| Change | Implementation | What changed and why | Assessment |
| --- | --- | --- | --- |
| Rich Aeneas contracts | `src/aeneas.rs`, `src/lean/ElicitAdapter.lean` | Replace scalar-only Rust-signature guessing with the actual translated Lean function telescope; support methods, self, structs, slices, generic/dictionary parameters and mutable-reference outputs | General backend capability, motivated by Adler's unsupported APIs |
| Original-function mapping | `src/aeneas.rs` | Match fresh extraction by source file/content/span and method name; require transparent local bodies, overflow checks and supported extraction options; reject ambiguous mappings | General correctness boundary; pinned output-format dependence |
| Helper scope and contract reuse | `src/aeneas.rs`, `src/lean/ElicitAdapter.lean` | Permit original local helper calls whose proof dependencies are checked; add contract-to-WP and WP-to-contract lemmas plus `elicit_spec` registration for callers | General Aeneas capability; not an Adler lemma library |
| Elaboration-aware specification identities | `src/aeneas.rs`, `src/lean/ElicitModelInputs.lean`, `src/model.rs` | Hash elaborated model declarations used by each contract; retain conservative whole-source fallback for unsupported inputs; distinguish proof selector changes from semantic contract changes | General mechanism; restricted fast-path applicability needs broader tests |
| Durable specification approval | `src/model.rs`, `src/cli.rs` | Save the explicitly approved semantic hash; recover readiness only when it returns exactly; changed models/contracts still need approval | General bug fix; no automatic approval of a new proposition |
| Unsupported targets and blockers | `src/model.rs`, `src/cli.rs`, `src/scheduler.rs`, `src/agent.rs` | Keep unsupported Aeneas trait-implementation targets visible but unscheduled; explicit method obligations remain separate; issue notes block retries until relevant changes or explicit retry | General backend limitation and retry fix |
| Relevant retry inputs | `src/model.rs`, `src/scheduler.rs` | Separate semantic context, target identity and proof inputs; changed diagnostic prose does not itself reopen work | General retry bookkeeping |
| Report-only invalidation filter | `src/fingerprint.rs` | Exclude root `PROGRESS.md` and `verification-inventory.json` only for statically screened simple crates; retain broad tracking for file reads, build scripts, custom imports/compiler hooks and other unsupported cases | **Direct benchmark-report naming coupling**; guarded, but not a general dependency tracker |
| Environment and installed-input fingerprints | `src/fingerprint.rs`, `src/cli.rs` | Canonical environment ordering; omit Aeneas-overridden LEAN_PATH and limited irrelevant Rustup selectors; track context-change causes; cache installed-file digests with inode/size/mtime/ctime guards | General reliability/performance mechanisms; platform/cache assumptions remain |
| Reconciliation and state coordination | `src/cli.rs`, `src/store.rs`, `src/analysis.rs` | Reuse persisted semantic analysis for identical complete inputs; wait on state transactions; add timestamped events and input-change details | General observability/concurrency fixes |
| Prepared-artifact checkpoints | `src/aeneas.rs`, `src/cli.rs`, `src/verifier.rs` | Check saved candidate proofs/lemmas before unrelated authoring; use checker-only refresh when possible; retain diagnostic cache and collected per-root outcomes | General preservation mechanism; declared missing roots can still cause work |
| Selected API goals | `src/plan.rs`, `src/cli.rs`, `src/model.rs`, `src/analysis.rs`, `src/lib.rs` | Add `--api`, exact/alias selector resolution and persistent selected roots without hiding the full inventory; add type/method aliases | General scope feature; changes the task denominator explicitly rather than proving the whole crate |
| Caller-first proof plans | `src/plan.rs`, `src/scheduler.rs`, `src/agent.rs` | Add Contract/Lemma/Integrate steps, prerequisite validation, root-delegation cycle checks, initial real Integrate attempts and blocked-plan repair; earlier mandatory planning-before-proof was replaced | **Benchmark-informed strategy**; can benefit this dependency shape, transfer not established |
| Persistent lemma checkpoints | `src/plan.rs`, `src/aeneas.rs`, `src/verifier.rs` | Check exact recorded propositions in Lemmas.lean, retain per-step input/hash evidence and separate lemma progress from API coverage | General mechanism with a fixed project file convention |
| Batch Lean root audits | `src/aeneas.rs`, `src/lean/ElicitAudit.lean` | Register each exact proposition/proof assignment synchronously with Lean; check transitive axioms; isolate individual invalid/missing roots; reject malformed or incomplete batch output | General checker optimization; safety regressions cover invalid roots, not just successful Adler proofs |
| Shared positive and negative results | `src/verifier.rs`, `src/cli.rs` | Persist every completed eligible root verdict; avoid repeating one pipeline for unrelated roots; negative results no longer poison requested successful work | General duplicate-check/result-scope fix |
| Productive-turn credits | `src/scheduler.rs`, `src/model.rs`, `src/cli.rs` | Count a new checked proposition as at most one credit against lifetime attempts; keep no-progress, same-input, task and time caps | **Benchmark-informed heuristic**; mathematical novelty is not a proof that a lemma helps the goal |
| Delegated-goal credit scope | `src/scheduler.rs`, `src/plan.rs`, `src/cli.rs` | Include directed callers of selected children, exclude siblings/unrelated goals, seed globally checked proposition identities before a turn to prevent association-only recredit; API scope unchanged | Generalizable graph bookkeeping fix demonstrated in Adler; no broad workload evidence |
| Worker context and instructions | `src/agent.rs`, `src/cli.rs` | Supply goal/plan, actual previous verification results, diagnostics and remaining time; instruct caller use and useful checkpoints rather than speculative helper enumeration | **Prompt strategy tuned from Adler failures**; see explicit benchmark-name coupling below |
| Worker deadline and final collection | `src/agent.rs`, `src/cli.rs`, `src/scheduler.rs` | Distinguish worker deadline from explicit user interruption; stop/join worker before checking; reserve `--finalize-seconds` default 60, bounded by one quarter; include initial sync/probes in the clock; handle lemma-only final edits | General defect fix; the 60-second default is benchmark-calibrated, not universally validated |
| Status and diagnostics | `src/cli.rs`, `src/model.rs`, `src/store.rs` | Show blocked targets, goal/plan, current checked lemmas, original cache/phase, task and pipeline events | General observability; conservative global derive guard still leaves selected-goal complete=false |

The actual post-Aeneas source/test counts and per-file figures are authoritative in the machine-readable inventory. No change is hidden by folding a failed experiment into a later successful one.

## Experiment, validation and documentation changes

These are separate from product behavior, but matter for the overfitting question.

- `experiments/adler2/prepare.py` pins the release/archive and creates fresh with/without cases from identical original sources. `TASK.md` defines five APIs, arbitrary-input semantics, preserved original computation and excluded scope. API names, Adler constants and the independent scalar model belong here and in saved solutions, not in production.
- `baseline.py` drives the ordinary Codex arm. Its later continuation policy requests continued unresolved work, with a five-API completion marker subject to independent audit. The 28-minute result finishes in one turn; the extra continuation turns were not used. This is a changed stopping policy compared with the older partial ordinary control, which must not be silently pooled with it.
- `codex-proxy.py` and `summarize.py` collect whitelisted resolved settings, per-thread usage and event kinds. Cached input remains a subset, and missing final updates on interruption remain possible.
- `run-arm.py` measures initialization/work/observations, executes actual intermediate status/text/JSON/list commands and preserves stopped state/events. Later fixes include terminal observation in reported totals and pass the actual remaining inner budget after initialization, with a declared two-second shutdown margin.
- The final continuation uses a preserved **external experiment script**, now published under the continuation evidence's `protocol/`. It skips init, reuses the same case and frozen binary, recomputes a cumulative three-hour evaluation cap, reserves 300 seconds for terminal observation and supports an explicitly declared retry. Actual retry usage was zero. This is experiment-specific setup, not a new product default.
- Independent audit scripts freshly extract original bodies, compare generated bytes, rebuild saved Lean modules and inspect theorem types/transitive axioms. Their subject-specific constants and API lists are expected; they are acceptance code for this benchmark, not runtime dispatch.
- Native fixtures test a small Counter/identity family, methods, slices, helper reuse, cache deletion, semantic invalidation, batch rejection and deadline/interruption behavior. They establish mechanism checks beyond literal Adler target IDs, but **not** general proof-authoring effectiveness on other real crates.
- `tests/ledger.rs` includes iteration 8's actual frozen Adler state and an exact write_slice target ID. This is intentional regression replay and does not enter production `include_str!` paths. It nevertheless couples the test suite to a benchmark evidence layout; the same mechanism also has synthetic caller/child/sibling regressions.
- CI adds formatting, Clippy, regular tests, MSRV and Python budget/initialization harness tests. Documentation and evidence commits preserve failures, source/binary freezes, proof artifacts, audits, hashes and changing protocols. They are not additional production fixes.
- All earlier foundational work is also in the commit inventory: initial Rust CLI/ledger/single-worker adapters; conservative Kani/Creusot evidence handling, recovery and scope checks; English documentation; and initial pinned Aeneas support. The only Cargo manifest delta after the initial reconstruction optimizes sha2/digest in development builds. That predates the Adler comparison and does not inject any subject-specific mathematics.

## Concrete coupling and transfer risks

| Finding | Evidence | Conclusion / review recommendation |
| --- | --- | --- |
| Common prompt names `BENCHMARK_TASK.md` | `src/agent.rs:20` | **Actual experiment-to-product leakage.** Replace with normal user task/requirements handling in a separately reviewed cleanup; not needed for the proof checker and not justified as a general product convention |
| Two exact report basenames get special treatment | `src/fingerprint.rs:118` | **Actual benchmark-format coupling.** The conservative checks and fallback reduce correctness risk; arbitrary report names receive no equivalent benefit. Consider a documented report/output model rather than accumulating names |
| Caller-first/checkpoint/continuation strategy was repeatedly selected on one crate | `src/agent.rs`, `src/plan.rs`, `src/scheduler.rs` and the iteration reports | **Plausible statistical overfitting.** New checked text may be irrelevant, root order is deterministic, and long proof chains may behave differently on other crates; no held-out evaluation establishes a general benefit |
| New time defaults were chosen around observed work | plan_seconds=180; finalize_seconds=60 with one-quarter cap | **Unvalidated defaults, not Adler constants.** The 60-second choice used preceding 26–29 second checks; a large slow proof can exceed that margin. Both values are configurable |
| Exact Adler state in a regression test | `tests/ledger.rs:1028` | **Test coupling, not solution injection.** Prefer a compact generic fixture for long-term maintenance; preserve the original evidence separately |
| Only a narrow subset gets fine invalidation | `reports_are_independent`, elaborated Specs model collector | **Uneven applicability.** Build scripts, custom imports/metaprograms and dependency layouts can retain broad invalidation and cost; performance on a simple dependency-light crate need not transfer |
| Fixed persistent module names | Specs/Lemmas/Proofs/ElicitContracts | **Current Aeneas project format, not Adler names.** A convention can be legitimate, but monolithic module rebuilds and restrictive custom-import handling may penalize larger projects |
| Pinned tools/output formats | Codex 0.159.x, exact Aeneas/Charon pins, Lean 4.35.0 | **Compatibility limitation, not benchmark answer leakage.** Broader tool-version support is not established |
| No Adler answers in shipping code | No matching subject names/constants in `src/`; only generic adapter/audit Lean files are embedded | No observed literal subject-specific solver. This does not validate scheduler/prompt generality |
| Audit/subject original bytes | Both fresh auditors and original source hashes | No observed replacement of optimized Rust with an easier scalar implementation or input-bound weakening in the accepted five goals |

The conservative selected-goal status gate remains another known limitation: excluded derive analysis keeps CLI exit 1 / complete=false even when the selected five proofs are current. This review did not whitelist derives, change the gate or retrospectively alter any trial.

## Interpretation and scope of this audit

The last two fixes were only the latest slice of a larger change set. It would be misleading to describe the full benchmark-driven development as just two small fixes. It includes significant new goal planning, prompt instructions, semantic fingerprinting, checker batching and retry heuristics, plus experiment changes.

There is enough evidence to retain the concrete correctness/recovery repairs as individually motivated mechanisms. There is not enough evidence to claim that all new strategy and default choices improve typical Rust verification. A useful next decision is a code-scope cleanup review and a small held-out set with distinct proof shapes, not more Adler-specific tuning. This document proposes that direction; it does not implement changes or start more authoring experiments.

The 28-minute ordinary result should remain visible as unfavorable evidence for any claimed general speed advantage. A fresh controlled comparison or multiple untouched workloads would be needed to estimate one. Neither a single fast ordinary result nor the product's later 92-minute success establishes a runtime distribution, causal attribution, or freedom from overfitting.
