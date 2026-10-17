# Adler-32 follow-up after workflow repairs

The workflow defects were repaired, but this experiment does **not** demonstrate faster proof authoring or completion of the selected API set. The optimized slice-processing loop remains unproved in both arms. All intermediate trials, including unsuccessful ones, are retained.

## Final results

| Arm | End-to-end seconds | Explicit selected API proofs | Current method contracts | Whole goal complete |
| --- | ---: | ---: | ---: | --- |
| Ordinary Codex | 860.09 | 3/5 | Not scheduled as a package inventory | No |
| Repaired cargo elicit | 1,808.22 | 2/5 | 8/19 | No |

The control proved `new`, `checksum`, and `from_checksum`. The product recorded `checksum` and `from_checksum`, four low-level helpers, `Default`, and `Hasher.finish`. Both type specifications were ready. The product opened `new` only in its final eleven seconds and did not submit its contract. Neither arm proved the optimized `write_slice` or `adler32_slice` behavior.

There is an important qualification to the explicit API count. An independent post-trial audit assigned the existing `Default` proof constant directly to the exact `new` goal, and Lean accepted it without new proof tactics or proof search. Those goals are definitionally equal in the fresh model. Thus the product's saved artifacts logically cover 3/5 selected APIs, while its timed submissions and ledger cover 2/5. We retain both observations and do not backfill the timed ledger or milestones.

This is not a same-completion latency comparison. The product schedules the whole package inventory (19 obligations and two types), including extra methods; ordinary Codex followed the common five-API task and stopped with partial results. There was one fresh ordinary control and an iterative series of product repairs, not multiple independent matched pairs. No general speedup or slowdown factor can be inferred.

## What changed

- Unsupported Aeneas trait implementations stay visible in the denominator but are blocked from automatic retry. Unsupported retained contracts do not poison supported submissions.
- Explicit worker issues block retries until relevant semantic context changes or an explicit retry is requested. Diagnostic churn is not forward progress. Retry keys exclude unrelated proof-module edits.
- Saved proof artifacts receive a checker invocation before more specification work. All independently successful ready contracts from that invocation enter the ledger.
- Model dependencies are obtained from Lean's elaborated contract propositions and their transitive definitions. Independent new definitions retain existing specification fingerprints; changed reachable definitions invalidate them. Types bind the definitions present when accepted. Rust model and trusted toolchain changes remain relevant.
- Unsupported declarations, dynamic inputs and custom build or Lean behavior conservatively fall back to broader invalidation. Metadata is not proof evidence: changed global proof inputs still require fresh kernel checks.
- Simple static projects ignore operational report edits in specification readiness. Environment fingerprints are ordered and account for effective compiler and Lean inputs rather than overridden selectors.
- Events distinguish workers, checkers and changed inputs. Interruptions are reported as interruptions, not input changes; interrupted probes cannot accept evidence.

These changes do not prioritize easy APIs, weaken contracts, reuse earlier solutions, or claim arbitrary reuse of old proof certificates. Whole Rust model dependencies and whole proof-module checks remain conservative. Scheduling extra package work, specification/model elaboration cost and completion of difficult loop proofs remain unresolved performance issues.

## Final timed workflow

The final product source was [`bd7c422e888379f4a48764ad64ed9d16d59d4cc7`](https://github.com/nyuichi/cargo-elicit/commit/bd7c422e888379f4a48764ad64ed9d16d59d4cc7), tree `405dbaacafff7df37f9858b2baec49228d3dc649`. Its binary SHA-256 was `cd2e65233b610d2b415c8af56b1f593777eb17dc0d5fb3fbbcca5295132ed2d6`.

The work period was 1,801.25 seconds; terminal observation added 6.97 seconds. Initialization took 12.05 seconds within the measured run. There were eleven worker turns, no repeated target, no unsupported-trait worker turn, and no repeated arithmetic-helper authoring. Eight formal checkpoint invocations succeeded, checking 36 roots cumulatively as the saved proof set grew. Their event-bracketed wall time was 192.10 seconds; this is not the total verification overhead. Worker wall time also includes local commands and model/specification elaboration, whose cost was not separately isolated.

The first helper was recorded at 675.45 seconds and the first selected API at approximately 1,337.4 seconds. Actual `cargo elicit status`/JSON/list observations ran during progress, at the ten-minute fallback, after the first verified record, after the first API proof, and at termination. Their total cost was 32.76 seconds, including 25.79 seconds during work. Observations supplied no proof hints.

Reported cumulative token counters were 21,175,482 input tokens (20,991,232 cached) and 38,114 output tokens for the final product run, versus 7,045,645 input (6,911,488 cached) and 30,075 output for ordinary Codex. Cached input is a subset of input. Interrupted turns may omit their final update; these are observed counters, not billing estimates.

## Intermediate trials and costs

| Product candidate | Seconds | Outcome |
| --- | ---: | --- |
| Environment fingerprint candidate | 517.39 | Excluded before scoring: Cargo/direct environment identity mismatch reproduced and repaired; artifacts not reused |
| `86271b9` | 1,806.74 | Nine saved valid methods, including three selected APIs, but zero timed ledger acceptances; first official check started only 27 seconds before interruption |
| `5ea3258` | 1,807.70 | Nine successful checker invocations; three selected APIs and nine methods previously recorded, then all made stale by independent model additions near the deadline |
| `bd7c422` final | 1,808.22 | Eight current methods, two explicitly mapped selected APIs; no repeated worker target |

The first complete repaired trial exposed delayed checking and incorrectly classified interruption. Early artifact checkpoints fixed that problem. The next trial exposed whole-model hashing: eleven independent definitions appended near minute 29 invalidated existing specifications and caused another type-specification visit. Elaborated dependency tracking repaired that mechanism. Fresh post-trial checks confirmed all nine saved method proofs in both intermediate runs. These post-trial findings are not timed ledger results.

The original [first comparison](adler2-comparison.md) remains historical evidence. Its product run spent about 400 seconds on six unsupported-trait rechecks and finished with zero selected API results. The final run removes that recurrence, but still does not complete the requested proof work. Failed development runs are additional cost, not discarded successful measurements. Only the final trial ran without competing authoring/compiler/audit workloads; an ordinary-arm audit overlapped roughly the first minute of the first complete intermediate product trial.

## Subject, acceptance and independent audits

The subject is unmodified crates.io `adler2` 2.0.1, archive SHA-256 `320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa`, also used by proofs.rs/report/7. That report's 680.2 seconds describes proof rebuilding/auditing, not comparable end-to-end authoring time. The [common task](../../experiments/adler2/TASK.md) covers arbitrary representable seeds and byte slices, including empty slices, with the original optimized Rust loops. There is no normalized-seed assumption, artificial slice-length bound, or replacement Rust implementation.

Both arms used the same task, source, lockfile, 1,800-second work budget, Codex 0.159.2, resolved `gpt-6.1-sol`, provider OpenAI, unspecified reasoning/service tier, workspace-write sandbox, network-enabled configuration and on-request approvals. Neither received human proof hints or imported proof solutions. Prepared tool installation and validation were outside authoring time. Raw model messages, authentication material and session databases are not published.

Independent audits checked unchanged original Rust and manifest bytes, fresh Charon extraction and generated-model correspondence, transparent API bodies, exact contracts, rebuilt proofs and transitive axioms. Only `propext`, `Classical.choice` and `Quot.sound` were accepted. The control has 27 audited theorem roots, including conditional/math support; that is not 27 Rust APIs. The final product has eight exact audited method-contract assignments and two codec roundtrips. The latter are not slice-loop proofs. Original helper preconditions remain explicit.

Tool pins: Aeneas `e62dd83d133d9043e7d3628096838b7430f36565`, Charon `de8736f7c995ac54dc490345cef9ea5143c06c62` (0.1.293, nightly 2026-09-17), Lean 4.35.0, x86_64 Linux, default/std extraction with the original overflow checks. Exact source freezes, settings, timed ledger/events, proof files, audit outputs and token counters are in the [evidence directory](../../experiments/adler2/results/2026-10-17-follow-up/README.md).

## Post-trial invalidation replay and implementation validation

After freezing the final ledger and independently auditing the original artifacts, a separate replay appended an unused pure Lean definition to the actual subject. All ten ready specifications retained their fingerprints, the changed global proof input caused all eight method contracts to receive a fresh successful native check, and no worker started. The original specification file was restored byte-for-byte. This took 78.56 seconds including restoration/status and is explicitly outside the authoring benchmark. It demonstrates repair of the observed invalidation mechanism, not improved authoring speed.

Formatting, Clippy, Rust 1.86 and 61 regular tests passed. Both real Aeneas fixtures passed together in 801.82 seconds on the core dependency-tracking implementation, covering independent extensions, reachable-model rejection, unsupported-declaration fallback, negative proofs and resume/cache deletion. Final semantic-stamp and fallback refinements passed the 61 tests and a dedicated 102.55-second native documentation regression. The tested source's [CI passed](https://github.com/nyuichi/cargo-elicit/actions/runs/39055140337).

The final evidence supports a narrower conclusion than a performance win: repeated scheduling and avoidable loss of recorded progress were repaired. It does not show that cargo elicit makes full-crate or selected-API verification easier to finish. Achieving the optimized loop proof and reducing specification/scheduling overhead need further work and another controlled experiment.
