# Planning a path toward ordinary-Codex authoring time

Status: quantitative design plan reviewed with Astra; no implementation or new authoring benchmark. The user asked for a plausible path to near-parity before starting further work, rather than another sequence of Adler-specific adjustments.

## Decision

A path toward approximately ordinary-Codex authoring time is conceivable, but **small optimizations that preserve the current fine-grained authoring controller do not justify that expectation**. The candidate must allow essentially ordinary autonomous authoring over the user's selected goal set, with verification, approvals and evidence management added around it. Keep the checker and ledger; allow the authoring controller to change substantially.

The proposed design has a falsifiable cost budget. It is not a forecast that a fresh model run will take a particular number of minutes. The 28.40-minute ordinary reference is one recorded successful trajectory, not a population mean.

## Backsolve the required improvement

For planning only, define near-parity as no more than 10–20% added elapsed time relative to a matched ordinary control. Against this historical reference, those targets are **31.24–34.08 minutes**. A looser “30–35 minute” phrase is an approximate range, not exactly the same target band.

The [measured disjoint accounting](adler2-latency-analysis.md) gives:

- Ordinary elapsed B = 28.40375 minutes.
- Product authoring-window increase E = 47.28887 minutes: additional command-active plus other worker activity. This is not a claim that all E is waste caused by the harness.
- Current product time outside worker windows = 13.69973 minutes, versus 0.03349 ordinary minutes.

A useful requirements model is:

**Candidate elapsed ≈ B + E × rho + H**

Here rho is the fraction of the observed extra worker activity remaining in the candidate; it is unknown. H is the **net additional critical-path cost** of required verification, status, approval/protocol and related work versus what the ordinary authoring run already pays. Subtract replaced work, do not double-count overlapping work, and do not omit status to hit a target. Work inside a future worker must not escape the budget merely by moving between categories: H and rho must be defined so each extra interval is counted once.

The table is a set of conditional scenarios, **not estimates of likelihood or confidence intervals**.

| Assumed remaining worker increase | Assumed H | Resulting minutes | Relative to 28.40 |
| --- | ---: | ---: | ---: |
| None | 2 min | 30.40 | 1.07x |
| 5% of E | 3 min | 33.77 | 1.19x |
| 25% of E | 3 min | 43.23 | 1.52x |
| 50% of E | 4 min | 56.05 | 1.97x |

For the +20% target, H of 2 / 3 / 4 minutes requires eliminating approximately **92.2% / 94.3% / 96.4%** of the observed worker increase. That is close to restoring the ordinary authoring workload, not merely improving it somewhat. Eliminating 75% still leaves approximately 43 minutes when H=3.

For +10%, only 2.84 additional minutes are available in total. H=3 already exceeds that budget even if the candidate authoring takes exactly as long as the ordinary reference. Conversely, arithmetic does not rule out +20% if authoring is near the ordinary workload and H is about 2–4 minutes. Those H values are an engineering budget to test, not a proven unavoidable cost or guarantee.

Optimizing outside-worker work alone leaves approximately 75.66 minutes even if that work is free. Even reducing it to 2–4 minutes while leaving authoring unchanged gives approximately 78–80 minutes. That is not a near-parity plan.

## Candidate architecture needed for the optimistic scenarios

### Author the user's goal set continuously

Use the user's actual selected set, such as all five selected APIs, as the autonomous assignment. Let the worker choose proof order, supporting lemmas and source module decomposition. Keep explicit plans available, but remove fine plan installation, per-lemma returns and per-API reassignment as mandatory paths to progress. Save useful checkpoints while continuing work. Return when the goal set is complete, a concrete blocker needs a decision, the overall authoring budget ends or the user interrupts.

A short comparison that removes only the checkpoint-return instruction can diagnose one possible cause. **It is not the complete near-parity design.** The present worker budget is min(task_seconds=600, remaining authoring time), whereas the ordinary reference took about 1,702 seconds in one turn. Keeping that cap while removing normal checkpoint returns would introduce deadline interruptions that did not occur in the latest product run. The near-parity candidate must explicitly make the user goal set and remaining overall budget the execution unit, with finalization time reserved, rather than treating a longer task timeout as another subject-specific tuning parameter. Compare equal total budgets and interruption behavior.

### Make the ledger consume evidence without repeatedly directing proof exploration

Keep semantic contract approval, exact target mapping and required units. Support concise or batched metadata/submission operations where they can share stable-input processing while preserving explicit per-contract identities and approvals. The worker should not need repeated prose planning, revision queries and complete syntax pipelines simply to maintain bookkeeping for already saved stable work. Measure such costs before implementing an elaborate new protocol.

Status must remain useful during a long goal-level turn. Distinguish saved but uncollected work, evidence checked for a particular immutable input, and evidence current for the latest input. Never promote author claims or stale evidence to current success. Official acceptance needs stable inputs; if checking during authoring is supported later, it must use a well-defined immutable snapshot or an explicit stable checkpoint. This plan does not require introducing a new concurrent verification subsystem before testing the basic authoring hypothesis.

### Remove unnecessary proof-layout constraints

Track local Lean source modules and their import graph instead of forcing every completed caller into one persistent Proofs.lean. Local incremental development should not re-elaborate a completed heavy caller for every independent decoder edit. Original Rust/model/contracts, imported source and tool changes must still invalidate affected work.

Separate this from official cache reuse. Current official verification rebuilds a fresh directory, so splitting source files alone does not avoid rebuilding unchanged modules; no official speedup from compiled-module reuse is assumed. Add compiled-module reuse only if a measured mandatory-cost problem requires it, with validated provenance for source/model/contract/import/tool/configuration inputs and compiler outputs. Mere presence of a .olean is insufficient. Retain an independent final fresh rebuild to validate correspondence and acceptance.

### Preserve the actual trust boundary

Do not alter original runtime implementations, restrict arbitrary-input contracts, reduce required API coverage, reuse proof results across changed semantic inputs, relax specification approval or weaken Lean kernel/transitive-axiom validation. Remove experiment-specific prompt/report coupling as general cleanup; do not substitute Adler mathematics or a preferred tactic recipe for autonomous authoring.

Likely implementation boundaries are src/agent.rs instructions/task payload, src/cli.rs authoring/collection boundaries, src/plan.rs dispatch/input conventions, and src/aeneas.rs plus src/fingerprint.rs local module/input handling. Existing proof acceptance and persistent state remain reusable infrastructure. This is an authoring-controller redesign, not a total repository rewrite.

## Gates before committing to long authoring experiments

### Gate 1: measure the mandatory cost without an LLM

Replay saved, unchanged proof artifacts through the candidate's mandatory processing and final fresh acceptance. Use both existing artifact layouts, including the slow saved proof, rather than assume a new worker will generate a cheaper proof. Any format adaptation or contract metadata processing required by the candidate belongs in the measured cost; no new mathematical proof terms or weakened contracts may be supplied. This is an investigator-only replay, never prior-solution input to a fresh worker.

Measure net serial extra time H relative to the ordinary workflow, including configured actual intermediate/final status work and any necessary approval/import handling. Keep cold mandatory work and optional reusable caches distinguishable. The observed final product build of about 162 seconds is a cost of that artifact and layout, not a universal lower bound. If it replaces the ordinary 54-second build, the extra is about 108 seconds; if it is an additional final check, count its full cost. Independent post-trial audit remains separately identified and symmetric across arms.

Cheap rejection rules, using B=28.40 as the working reference:

- If unavoidable H exceeds **5.68 minutes**, +20% is impossible under the assumption of equally fast authoring. Revisit required verification/checkpoint architecture before a long benchmark.
- If H exceeds **2.84 minutes**, the same assumption cannot achieve +10%. Do not call +20% success +10% parity.
- If net H is about **2–4 minutes**, near-parity is arithmetically possible, but the authoring-workload hypothesis still has to be tested.

These are necessary-condition checks, not proof of impossibility if the candidate genuinely authors faster than ordinary Codex. The plan does not rely on that unsupported speedup to make its budget work.

### Gate 2: validate mechanisms on small independent fixtures

Verify long goal-level control, partial persistence, user interruption/resumption, accurate status scope, batched approval semantics, module dependency invalidation and resumed-counter aggregation. A changed model/import/tool must invalidate affected evidence; an unrelated local edit should not rebuild an independent local peer when incremental behavior is claimed. These checks are cheap mechanism validation, not performance evidence about real proof authoring.

### Gate 3: test whether authoring is actually near ordinary work

Use a small predeclared set of previously unused tasks with distinct proof shapes, matched mathematical contracts, original-source preservation, tools/model/settings, total budgets and status policies. Keep previous proof solutions inaccessible to fresh workers by the experiment boundary. Compare ordinary and candidate runs sequentially without competing compiler/author workloads. Freeze source/binary and acceptance rules before the trials. Inspect completion, corrected unique-thread usage, proof/compiler work, protocol cost and accepted proof extent, not just elapsed time.

The narrow return-condition comparison and the full goal-set-controller candidate answer different questions; label them separately rather than present multiple simultaneous architectural changes as a one-factor ablation. A first pair is a pilot, not a distribution or general speed claim. Repeat only when variance or incomplete results leave the decision unresolved; do not tune repeatedly on the same task.

## What would make us continue, redesign or stop

- If the fixed mandatory cost cannot fit the extra-time budget, change verification/checkpoint scheduling before spending on long model runs.
- If the goal-set candidate fits that budget and fresh matched pilots put total time near 1.1–1.2x ordinary with the same acceptance, there is evidence to continue that design. That is not a promise for every crate.
- If total time remains around 40–50 minutes relative to a comparable 28-minute ordinary run, do not call it near-parity. Attribute residual costs from observable events. Remove remaining mandatory authoring protocol if that is responsible, rather than add retry credits or Adler-specific prompts.
- If the candidate is already close to ordinary instructions/control and the remaining difference looks like proof-strategy variation, use limited repetitions before assigning causality. Do not infer architecture failure or success from a single random trajectory.
- If repeated matched results still require substantial extra exploration, reconsider or retire the authoring-controller layer while keeping the checker/ledger functionality. Do not continue indefinitely with small timing/default changes.

There is no data-backed probability that the 30–34 minute scenarios will occur. The credible reason to investigate them is architectural: the product can stop prescribing a different, more fragmented proof-authoring workflow and instead add a bounded evidence-management cost around ordinary autonomous work. Whether contract integration and status can remain within that bound is exactly what the gates are intended to establish before a larger investment.
