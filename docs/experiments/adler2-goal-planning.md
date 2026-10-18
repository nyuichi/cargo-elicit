# Adler-32 comparison after goal-directed planning

The implementation works, but this fresh comparison is still a failure for cargo elicit. Ordinary Codex completed all five selected APIs in 1,704.23 seconds. Cargo elicit exhausted its budget with three APIs in its saved proof artifacts, only one current selected API in its frozen ledger, and no proof of the optimized slice-processing core. Independent fresh extraction and Lean audits confirmed these results.

This follows the [workflow repair experiment](adler2-follow-up.md). Its old ordinary control is historical evidence, not the control used here. The [evidence directory](../../experiments/adler2/results/2026-10-17-goal-planning/README.md) includes exact proof files, frozen state/events, observations, audit programs and outputs, hashes and reproduction instructions. These are solutions and must be withheld from future authoring workers.

## Results

| Arm | End-to-end seconds | Independently checked selected APIs in saved artifacts | Current selected APIs in timed terminal ledger | All five proved |
| --- | ---: | ---: | ---: | --- |
| Ordinary Codex | 1,704.23 | 5/5 | No product ledger | Yes |
| cargo elicit | 1,808.50 | 3/5 | 1/5 | No |

The control proves `new`, `checksum`, `from_checksum`, `write_slice` and `adler32_slice`. Its original six-loop optimized computation is connected to an independent scalar recurrence, for arbitrary representable states and byte slices, including empty input. The recurrence normalizes both seed halves modulo 65521 even on empty updates. Its additional split/save/restore laws are separate from the five-API count. Representability conditions on concatenated slices in those secondary laws do not restrict either primary slice API.

The product's saved modules prove `new`, `checksum`, `from_checksum` and four vector helpers. Its frozen terminal ledger has four current method contracts: `checksum`, vector addition, remainder and multiplication. `new`, the decoder and the vector conversion remain marked `specification changed`; earlier success records are not current evidence. A separate post-trial audit freshly rebuilt and accepted all seven saved method proofs without adding proof terms. This does not backfill the timed ledger. The original Rust and Cargo manifest bytes remain unchanged in both arms.

Three product checkpoints are real checked propositions: exact chunk lengths, short remainder length, and reconstruction of the input from chunks and remainder. They are useful list-decomposition facts, not loop invariants or a proof of optimized execution. The final `Proofs.lean` does not import `Lemmas.lean`, and there is no completed core caller consuming those facts. Seven generated `elicit_spec` rules compile; there is no checked optimized caller using them. The native regression fixture separately demonstrates actual caller use of a generated WP rule, not a benchmark speed benefit.

## What was implemented and validated

- Persist selected API roots independently of the Cargo build configuration; keep the full inventory visible.
- Ask workers to inspect original callers, record explicit helper/lemma/integration plans, and follow checked prerequisites instead of automatically draining the whole call graph.
- Check exact intermediate propositions in `Lemmas.lean`, including transitive axioms; retain them separately from API coverage and check saved work before asking the model again.
- Derive Aeneas total-WP rules from existential execution contracts and register them with the pinned backend's `[step]` database. The reverse conversion is also kernel checked.
- Bound planning turns independently, preserve partial files on worker deadlines, and record extraction, translation and Lean stage durations.

See [usage and boundaries](../goal-directed-aeneas.md). Plans describe intentions; neither their reasons nor their declared dependencies certify that a decomposition is necessary or sufficient. Cross-root explicit step dependencies are currently unsupported. Unsupported trait/derive analysis remains visible and can still prevent a declaration of complete verification.

Formatting, Clippy, Rust 1.86 and the regular test suite passed. The final source [CI passed all 71 regular tests](https://github.com/nyuichi/cargo-elicit/actions/runs/39076077445). Real Aeneas tests passed for intermediate checkpoints/caller registration (145.22 seconds) and the richer methods/structs/slices/helper/cache-deletion/resume fixture (250.05 seconds). These checks validate mechanisms, not the quality of a model-generated benchmark plan.

## Timed product chronology

Times below are seconds from the measured start, including initialization. They describe actual tasks rather than inferring progress from a work item's name.

| Time | Observed work |
| --- | --- |
| 0–18 | Initialization (12.96 seconds) and run setup |
| 18–284 | First `adler32_slice` planning task; the first worker hit its 180-second limit, retained a contract, then resumed and installed a plan |
| 286–440 | `checksum` planning/proof work and official checkpoint; first selected API accepted at about 438.3 seconds |
| 440–533 | `new` planning/proof work and checkpoint; its success recorded at about 530.9 seconds |
| 533–729 | `write_slice` planning, scalar specification and three chunk lemmas; core and wrapper theorem names still unresolved |
| 729–980 | Decoder task, saved exact decoder proof and official checks |
| 980–1,225 | Vector conversion task and official checks |
| 1,225–1,562 | Vector addition task; the worker also completed remainder and multiplication proofs |
| 1,562–1,613 | Return to decoder: checker-only, no model called, task still unresolved |
| 1,615–1,666 | Return to vector conversion: checker-only, no model called, task still unresolved |
| 1,668–cutoff | `checksum` integration/re-submission; checksum becomes current again, overall run interrupted at the budget |

There were nine model worker calls and eleven scheduled tasks. Four roots were visited in five planning turns; the decoder's plan was supplied with the other work. Worker time inside planning tasks totaled 610.84 seconds. This includes actual contract/proof work and local commands, so it is not 610.84 seconds of prose planning alone. Planning also delayed dispatch of the explicit core work. No dedicated `compute` proof-authoring task or loop-invariant checkpoint was reached before the cutoff.

Actual status/text/JSON/list observations ran after the first verified record, first selected API, historical three-API milestone, ten minutes and termination. They cost 46.96 seconds total, including 40.17 seconds during work and 6.79 seconds after work stopped. The work period was 1,801.70 seconds, including initialization and interruption handling. Historical milestones do not mean those results remained current.

The events record 19 processing pipelines and 14 proof pipelines, including checks initiated inside workers and observations. Instrumented stages sum to 656.12 seconds: 347 stage completions, including 130 individual API/lemma audit invocations taking 243.07 seconds. Thirty-eight audit invocations failed, primarily for declared theorem names that had not yet been defined; this is not 38 failed full proof modules. All 14 recorded `Proofs.lean` builds succeeded. These accumulated stage durations overlap worker/observation time; they are not disjoint end-to-end phases. Model-input collection also invokes Lean separately and is not fully represented by these stage events.

## Why this still performed poorly

**Planning remained a barrier before the hard proof.** Selecting the five APIs removed the whole-inventory scope mismatch, but each unfinished root was still inspected before planned work was dispatched. The first caller's plan immediately listed vector helpers ahead of its core contract. The mechanism permits a caller-driven plan; it does not require the worker to exhibit an actual failed caller obligation before scheduling each prerequisite. No loop invariant, machine-arithmetic bound connected to a loop, or completed core theorem resulted in this run.

**Valid proof files and specification readiness still diverged.** The final decoder and constructor proofs compile against the exact final contracts, yet their readiness flags remain stale. Returning to the decoder and conversion consumed about 102 seconds in checker-only visits without repairing those flags. These were not repeated AI attempts to reinvent the same proofs. Inspection of the scheduler/verifier boundary shows that successful proof-module compilation does not itself reapprove a stale specification; integration can require another model turn and explicit submission. Conservative metadata fallback and later recovery can also change readiness. This run does not isolate every stale transition, so it would be premature to attribute every occurrence to a single fingerprint defect. The evidence establishes the unnecessary recovery burden and the terminal discrepancy, not a complete causal replay.

**Checking repeated too much work.** Whole modules and multiple theorem roots were rebuilt or audited as contracts accumulated, including unresolved declared roots. Storing checked lemmas helped retain facts but did not remove extraction/elaboration/audit costs or make them usable by the unfinished caller automatically. The required status observations add a measurable but much smaller cost than planning and repeated checking.

The early input events also record three context changes involving installed-tool/version fingerprints in the first 94 seconds. They are retained as observed reliability limitations, not silently excluded. The pinned source/binary and original runtime bytes were frozen; there were no later context-change events, human proof hints, or mid-trial product patches. A definitive environment/readiness diagnosis needs a separate controlled replay.

This is not explained by failing to prioritize easy APIs: even the saved-artifact count is incomplete, while the control finishes the hard original loops. A next iteration should remove mandatory planning sweeps, ensure unchanged valid specifications recover through checker-only reconciliation, avoid broad repeated audits, and measure time to actual checked loop material. Those are follow-up recommendations, not changes retroactively applied to this trial.

## Protocol, measurements and independent acceptance

Both cases were fresh from crates.io `adler2` 2.0.1, archive SHA-256 `320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa`, with the same [five-API task](../../experiments/adler2/TASK.md). All original Rust/manifest bytes and optimized loops were preserved. No earlier solutions or human proof hints were supplied. Product roots were exactly the five common APIs, not the whole package. Installation, preparation and upstream tests were outside authoring time. Authoring arms ran sequentially, ordinary first, without overlapping compiler/audit/authoring workloads. Independent audits started only after both arms finished.

Each arm had a 1,800-second end-to-end work budget. The new ordinary runner may continue completed partial turns in the same thread with a generic reminder, and stops on a five-API completion marker subject to independent audit. This control completed in one turn. Its stop policy differs from the older partial ordinary run. The product receives remaining time and plan context each turn. This was one fresh pair, not a repeated performance study or a same-completion speed ratio.

Both resolved to Codex 0.159.2, `gpt-6.1-sol`, OpenAI provider, unspecified reasoning effort/service tier, workspace-write, network-enabled configuration and on-request approvals. Observed cumulative counters were:

| Arm | Input tokens | Cached input subset | Output tokens | Started command items |
| --- | ---: | ---: | ---: | ---: |
| Ordinary | 28,327,415 | 28,067,584 | 76,504 | 204 |
| cargo elicit | 14,139,415 | 13,959,168 | 35,658 | 142 |

Cached input is a subset of input, not another additive category. Interrupted turns may omit a final update. These are observed counters, not billing estimates. Lower counters without completing the same work do not establish an efficiency advantage.

The tested product source is [`d2f8e1e7a18707d0e2f6f2e4f44706bfadf1b09b`](https://github.com/nyuichi/cargo-elicit/commit/d2f8e1e7a18707d0e2f6f2e4f44706bfadf1b09b), tree `bc6c150129b5d8ca307cb100ace8e106f93aa259`, installed binary SHA-256 `04468e9e9bf3a384d8186ffd8aca7d91e96d5ed9ecdfda56b77083e01a501a84`. Aeneas `e62dd83d133d9043e7d3628096838b7430f36565`, Charon `de8736f7c995ac54dc490345cef9ea5143c06c62` (0.1.293, nightly 2026-09-17), Lean 4.35.0, default/std, x86_64 Linux, original overflow checks.

Independent auditors freshly extracted unchanged original sources, checked transparent original API bodies and extraction options, byte-compared generated models, rebuilt proof modules, printed exact root types/definitions and audited transitive axioms. Only `propext`, `Classical.choice` and `Quot.sound` were allowed. All 52 control theorem roots passed (52 theorems are not 52 APIs); all ten control proof modules rebuilt. The product's six modules rebuilt, seven of ten declared method roots passed, and all three planned lemma propositions passed. Missing `compute`, `write_slice` and `adler32_slice` theorem names failed and remain in the evidence. The control's selected extraction omits unrelated I/O; all selected APIs and their reachable original computation remain transparent. Product extraction is broader. Neither result claims whole-crate, alternate-target or no_std verification.
