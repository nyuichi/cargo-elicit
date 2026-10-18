# Autonomous authoring implementation and fixed-artifact validation

This implements the controller candidate in the [near-parity plan](adler2-parity-plan.md). It changes the Aeneas default; it is not a new matched authoring benchmark and does not alter previous trial records. The full interface and limitations are in [autonomous Aeneas authoring](../autonomous-aeneas.md).

## Changes

| Area | Result |
| --- | --- |
| Authoring control | One author receives the whole user-selected goal set and remaining overall authoring budget. Returning after every lemma, installing a plan, and individually submitting each API are no longer mandatory. Generated contract names and current review/proof status are supplied with the goals. |
| Semantic review | `agent stage` saves one explicit review bound to contract/model identities and original runtime/tool/configuration context. Proof-term edits may continue without restaging; semantic changes require new intent. |
| Acceptance | `collect` checks stable files without an AI worker. A single fresh pipeline records individually mapped/elaborated contracts, exact kernel decisions and transitive axiom checks. It accepts only the processed reviewed subset and retains any unprocessed staged reviews. |
| Proof organization | Tracked local `.lean` modules compile in import order. Local source objects cannot shadow generated/toolchain modules; missing sources, cycles and symlinks fail closed. Imported models invalidate semantic review; proof changes invalidate evidence. |
| Recovery | Interruptions retain active work and can resume without `--retry` solely due to interruption. Authoring deadlines preserve saved work without consuming a completed failed attempt. Unsuccessful checks can be retried on unchanged files. |
| Status scope | Saved sources and pending/stale reviews are distinct from current proofs. Selected Aeneas APIs can complete despite unrelated inventory discovery limitations, while whole-package inventory remains incomplete. Scope/configuration/unknown analysis issues still block completion; other backends and specification-only mode retain their previous analysis gate. |
| Counters | Unique-thread cumulative counters are merged by maxima, preserving old-ledger compatibility and avoiding resumed-thread double addition. |
| Compatibility | `--workflow planned` preserves the earlier dispatcher. Kani and Creusot retain that default. Official cross-invocation compiled-object reuse is not introduced. |

Production changes are generic workflow/evidence mechanisms. No Adler API names, modulus, proof tactics, benchmark filenames or task-specific timing defaults were added to production code. The saved-artifact replay script is explicitly an Adler experiment, outside the product implementation.

## Astra review

Astra reviewed the implementation twice, read-only. The review identified and the implementation fixed:

- unsuccessful current-input results suppressing all later collection attempts;
- stale existing dependency hashes surviving joint specification review;
- interrupted and budget-exhausted authoring being suppressed as unchanged-input failures;
- a concurrently staged review being replaced or archived by an older collector;
- approving staged contracts that proof mode had skipped without mapping/elaboration;
- an analysis-gate relaxation unintentionally applying to other backends and specification-only mode.

The checks still cannot decide whether a human-requested mathematical property was adequately specified. The explicit semantic note records that judgment; kernel acceptance does not replace it.

## Validation and cost replay

Validation logs, frozen binary/source hashes, command timings, saved-artifact adaptation and final ledger/status records are linked from the accompanying evidence directory. The replay includes preparation, initialization, semantic staging, three real `status --json` calls and final fresh collection. It does not reuse project `.olean` files or run an LLM. Toolchain libraries and their digest acceleration cache are installed/warm, as in the prior experiments.

The product layout preserves its saved proof terms. The ordinary layout mechanically renames imports/namespaces and the old `Proofs` support module, then adds direct wrappers exporting the already proved API theorems to the generated contract interface. Those wrappers introduce no new mathematical lemma or weakened input domain. Their compilation/acceptance belongs in the measured replay cost. The original optimized Rust and manifest hashes must remain unchanged.

A replay measures evidence-management cost around saved work, not the distribution of fresh authoring time. Even a passing cost gate cannot establish that autonomous authoring will reproduce the ordinary 28.40-minute trial. The next authoring experiment must use fresh tasks/threads and keep these prior solutions inaccessible. Do not turn repeated Adler trials into production tuning.

See [all validation records and measurements](../../experiments/adler2/results/2026-10-18-autonomous-acceptance/README.md).

## Measured cost

| Saved artifacts | Total processing | `collect` command | Selected API completion | Conditional 28.40 min + gross cost |
| --- | --- | --- | --- | --- |
| ordinary saved layout | 99.747 s (1.66 min) | 77.546 s | 5/5 | 30.07 min (+5.85%) |
| product saved layout | 216.298 s (3.60 min) | 181.434 s | 5/5 | 32.01 min (+12.69%) |

Both layouts pass the +20% gross-cost screen under equal-authoring arithmetic. The ordinary layout also fits +10%; the slow product layout does not. These are necessary-condition screens, not observed end-to-end ratios or forecasts. No authoring improvement has been measured here. Avoid subtracting the ordinary final compile unless a future workflow actually replaces it; the table conservatively appends the entire replay cost.

## Astra interpretation and stopping point

Astra reviewed both measured costs and agreed that mandatory processing no longer rules out +20% under these warm-toolchain conditions. Appending all measured work to the ordinary reference leaves about 241.10 seconds of the +20% budget with the ordinary saved layout, and only 124.55 seconds with the slow layout. The ordinary layout fits the +10% cost allowance; the slow layout exceeds it under the same conservative arithmetic.

This remaining allowance is for any additional authoring overhead, not free time already demonstrated. Net cost may be lower only if actual replacement of baseline work is established. No source tuning is justified by these acceptance results alone. Freeze this candidate and evaluate fresh matched authoring separately rather than continue adapting the product to Adler.
