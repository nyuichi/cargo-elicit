# cargo-elicit versus ordinary Codex: Adler-32 experiment

**Measured on 2026-10-17: this pair did not show a proof-authoring benefit from cargo-elicit.** Ordinary Codex produced independently accepted contracts for **3/5 selected APIs in 594.06 seconds**. cargo-elicit reached its 30-minute work limit, with **0/5 selected APIs** and **1/19 currently verified ledger targets** at termination. Its saved work did contain three valid arithmetic-helper proofs, confirmed after the timer stopped. Neither arm proved the optimized update loops or the complete computation core. This is one exploratory pair, with different completed work; it is not a general speedup estimate.

## Reference and subject selection

[proofs.rs report 7](https://proofs.rs/report/7) covers the five computation-core APIs of adler2 2.0.1, plus supplementary codec, split and resumption laws. Its evidence lists 46 audited theorem roots. The recorded **680.2 seconds** are the time to build and audit existing proofs, including dependency preparation; they are not the time spent authoring them. The authoring evidence describes a multi-model workflow, whereas this experiment uses one Codex worker in both arms. We cannot infer an authoring speedup from that recorder time.

An initial candidate, adler 1.0.2, has 433 Rust lines across two files and very similar APIs and optimized loops. Its six unit tests and four doctests passed, and Charon extraction succeeded. However, the current pinned Aeneas could not translate its old `zip`-based lane helper, so it is unsuitable for a comparison with this shared toolchain. gcd 2.3.0 was also inspected; its macro-generated APIs and different algorithm make it a less comparable workload.

The subject is **adler2 2.0.1 itself**, starting from the unmodified crates.io release with no existing proof files. It has 442 Rust lines across `src/lib.rs` and `src/algo.rs`, including comments/tests. Its six unit tests and four doctests passed, and the shared pinned Charon/Aeneas installation successfully translated it. The downloaded archive's SHA-256 matches the official registry index:

```text
320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa
```

Both workers are forbidden to consult published adler/adler2 proofs, the report's proof repository, sibling experiment workspaces, or cargo-elicit fixture proofs. Shared tool installation and standard model documentation are allowed.

## Capability checkpoint before timing

At cargo-elicit commit `8c529199871ef31c5d40dfc8aae29412f81a3b58`, the Aeneas adapter accepts full completion only for independent primitive free functions. All five computation-core APIs fall outside that scope: four are methods and the free function takes a slice and calls helpers. This is a source/adaptor capability finding, **not a measured 0/5 authoring result**. The adler 1.0.2 initialization also reported 21 definitions and incomplete analysis for derives.

The user chose to extend support before running the timed comparison. The extension generates contracts inside Lean from the entire actual translated function telescope. It maps transparent Charon bodies and generated Lean definitions to exact original source spans, then checks the resulting theorem and its transitive axioms. Rich native fixtures exercise methods, structures, mutable state, slices, machine-size integers and local helper calls. These fixture checks are preparation evidence, not an Adler-32 proof or a performance benchmark.

Whole-package completion and the five selected computation-core contracts are separate outcomes. Derived traits, generic I/O and other unprocessed targets may remain unresolved in the cargo-elicit ledger even if selected core contracts succeed. They must remain visible in status output.

## Fixed common goals

Prove successful execution in the extracted model and functional correctness for:

1. `Adler32::new`: state `(a,b) = (1,0)`.
2. `Adler32::checksum`: encoding `65536*b+a` for every state.
3. `Adler32::from_checksum`: low/high 16-bit decoding for every `u32`.
4. `Adler32::write_slice`: the scalar Adler recurrence modulo 65521 for every representable state and input slice, including normalization on empty input.
5. `adler32_slice`: that recurrence from `(1,0)`.

The original optimized body, loops, machine arithmetic, indexing and helpers must be covered. No artificial length bound or normalized-seed precondition. Counterexamples are valid findings and must not be hidden by silently weakening a contract. Supplementary codec roundtrips, split updates and save/restore/resume laws are scored separately, with concatenated-slice representability explicit. Generic `BufRead` I/O, `Hasher` laws, derived traits, `no_std` and other targets are outside the common selected-API goal.

## Arms and measurements

| Control | With cargo-elicit | Without cargo-elicit |
|---|---|---|
| Worker | One Codex app-server worker | One ordinary Codex app-server worker |
| Model and reasoning | Same resolved model/settings, recorded below | Same resolved model/settings, recorded below |
| Sources and tools | Identical original release, lockfile and pinned tools | Identical original release, lockfile and pinned tools |
| Authoring | Product scheduler, ledger and official wrapper | Direct Charon/Aeneas/Lean commands |
| End-to-end budget | 1,800 seconds, including initialization and progress checks | 1,800 seconds, including startup and analogous observations |
| Human proof hints after start | None | None |
| Independent acceptance | Fresh source extraction, proof rebuild, API/contract mapping and transitive axiom audit | The same acceptance criteria |

Prepare dependencies and shared libraries outside the authoring timer; report setup separately. Use fresh independent sessions and run sequentially, ordinary Codex first, to avoid CPU/memory contention. Record the order. A single pair is exploratory and does not establish a statistically reliable speedup.

Record end-to-end time, agent time when available, input/cached-input/output tokens, verifier invocations, retries, approval or authentication interruptions, human interventions, and independently accepted API coverage. Missing telemetry is unavailable, never zero. A successful turn or process exit is not a proof. Preserve original source hashes and inspect the actual theorem statements, generated model correspondence and axioms; only `propext`, `Classical.choice` and `Quot.sound` are permitted.

## Progress checks requested by the user

Run actual `cargo elicit status` and `cargo elicit status --json` after the first successful API proof and around halfway through coverage. If no proof succeeds, capture a ten-minute checkpoint and the terminal state instead. Preserve stdout, exit status, duration, counts, current limitations and any reconciliation/revision effects. Use exactly the same environment and Cargo invocation style as the running arm so the observation does not accidentally select a different compiler configuration.

Include observation time in the end-to-end measurement, and also report it separately. No proof hints are injected after viewing status. For ordinary Codex, capture available progress/artifacts at analogous checkpoints without introducing cargo-elicit; theorem declarations alone do not count as successful coverage. Compare how clearly each workflow exposes verified, stale, unfinished and unsupported work.

## Measured results

The tested product was commit `e4def34d4b1879835b226273b369539860693adf`, including the earlier Aeneas extension and state-lock fix. Both arms used Codex CLI 0.159.2, `gpt-6.1-sol`, default reasoning (`reasoningEffort: null`), one thread/worker, the same tools and original-source hashes, and no human proof hints. Ordinary Codex ran first. The baseline stopped voluntarily with a partial-result report; the product was interrupted at the work limit. Do not interpret their time ratio as a speedup for equal completed work.

| Metric | With cargo-elicit | Without cargo-elicit |
|---|---:|---:|
| End-to-end elapsed, including terminal observation | 1,808.40 s | 594.06 s |
| Work/startup through process stop | 1,801.76 s | 594.06 s |
| Initialization within timer | 12.71 s | Not applicable |
| Observation commands/scans, including extra checkpoint | 22.25 s | 0.00080 s |
| Input tokens, including cached input | 23,160,269 | 4,212,259 |
| Cached input tokens (subset of input) | 22,806,016 | 4,109,440 |
| Output tokens | 47,589 | 20,583 |
| Independently accepted selected APIs | **0/5** | **3/5** |
| Original Rust/manifest unchanged | Yes | Yes |
| Human proof hints | 0 | 0 |
| Process exit | 130, budget interruption | 0, partial result |

The observer's time during authoring overlaps the work timer and must not be added again. The terminal check added 6.64 seconds after process stop in the product arm. The underlying 1,800-second work budget also incurred 1.76 seconds of polling/interruption overhead. Token totals are actual per-thread cumulative usage, grouped across resumed peers. Cached input is not an additional token category. Agent-only time, total verifier invocations including local experiments, per-API retries, and monetary cost are unavailable. Command-item starts were 150 with the product and 74 without; these are not counts of verifier invocations.

### Which claims survived independent review

| Selected API | With cargo-elicit | Without cargo-elicit |
|---|---|---|
| `Adler32::new` | No submitted API proof | Successful state `(1,0)` |
| `Adler32::checksum` | No submitted API proof | Successful encoding for every representable state |
| `Adler32::from_checksum` | No submitted API proof | Successful decoding for every `u32` |
| `Adler32::write_slice` | Unproved | Unproved |
| `adler32_slice` | Unproved | Conditional on the unproved update contract; not accepted |

Ordinary Codex also proved both codec roundtrips and exact save/restore equality of subsequent computations. That equality does not prove that `write_slice` succeeds or implements the recurrence. Its split theorem concerns the mathematical scalar specification, not the original optimized implementation. A four-byte loader theorem and arithmetic bounds are useful support, not full loop verification. Its finite runtime tests are not universal proofs.

The product worked first on structure invariants, arithmetic trait laws and their explicit helper methods. The scheduler repeatedly returned to unsupported trait targets and helpers before reaching the requested core APIs. The saved proof module proves the original remainder, multiplication and addition helper methods. Remainder requires a nonzero divisor; multiplication and addition require their natural intermediate results to fit `u32`. Their compute callers' safety conditions and the loops remain unproved. These helper contracts must not be scored as the five selected APIs.

The timed ledger contains five official proof-result records: one unresolved record and four successful records across two distinct helper targets. Repeated success for the same helper is not additional coverage. At the frozen terminal input, only multiplication retained a current success record. Post-trial verification of all three saved helper contracts succeeded in 22.78 seconds, and a separate rebuild/audit of the fresh generated model, contract generator, contracts and unchanged proof module took 7.83 seconds. These checks confirmed useful partial artifacts; they do not change timed coverage or the frozen terminal ledger.

For ordinary Codex, the final reproducibility audit took 7.25 seconds. It re-extracted the original Rust, obtained a byte-identical Lean model, checked the source contents and transparent API bodies, confirmed overflow checks without monomorphization or skipped type/borrow checks, rebuilt the model/proofs, and printed the exact root types and transitive axioms. The product helper audit checked the same source and extraction properties and independently rebuilt every Lean module. Every accepted root uses only `propext`, `Classical.choice` and `Quot.sound`. The audit times are outside authoring; earlier review/debugging time is not included in those final checker durations.

### What the requested status checks showed

At 601.46 seconds, actual text status, JSON status and target listing all exited zero in 8.47 seconds. They showed types specified 2/2, functions/methods/impls specified 1/19, fully verified 0/19, and unresolved derive analysis.

An **additional, non-preplanned** checkpoint after historical helper success ran the same three commands in 7.14 seconds. It showed specifications 2/19 but current verification 0/19. The target JSON marked earlier successful helper records stale after subsequent input edits, including report/proof edits. This is a concrete benefit of checking current status rather than counting successful log entries. The checkpoint supplied no proof hints and its time was already inside the ongoing timer.

At termination, the three commands again exited zero in 6.64 seconds: types specified 2/2, other specifications 3/19, fully verified 1/19, unresolved, with an interruption checkpoint retained. Neither the five selected APIs nor the whole package were complete. No first-selected-API or halfway-coverage trigger fired. The ordinary arm finished before its ten-minute observation fallback, so only its terminal artifact scan ran; filenames were not scored as proof success.

### Setup failures and execution controls

Earlier HTTP 401 requests never became authoring trials. Device authorization repaired authentication and real requests then succeeded. Two initial ordinary-arm starts each stopped after about eight seconds, before proof authoring, because commands required approval: first inherited read-only mode, then a protected outer sandbox mount appeared as a mode-000 app-server socket directory. Their original sources were unchanged and they produced no proof artifacts. They are disclosed setup failures and excluded from the accepted pair, not scored as poor proof performance.

Both accepted arms used the same real-Codex wrapper with these explicit configuration overrides:

```text
-c sandbox_mode="workspace-write"
-c sandbox_workspace_write.network_access=true
-c sandbox_workspace_write.writable_roots=["/workspace/toolchain"]
```

The inner Codex sandbox remained enabled and approvals remained `on-request`; unattended clients declined requests. The supported outer elevated execution path allowed that inner sandbox to initialize without the outer protected socket mount. No protected directory was chmodded and no inner approval was auto-granted. Authentication and command-execution smoke checks were outside authoring. Shared installation, earlier feature development and native fixture checks were preparation, whose total elapsed time was not measured. No published proof solution, sibling-arm proof or fixture proof was supplied to either worker.

Pinned shared versions: Aeneas `e62dd83d133d9043e7d3628096838b7430f36565`, Charon `de8736f7c995ac54dc490345cef9ea5143c06c62` (0.1.293), Lean 4.35.0, and extraction target `x86_64-unknown-linux-gnu`. The Aeneas checkout had no tracked changes after the runs. The upstream test suite passed before authoring: six unit tests and four doctests. Installed-tool build artifacts and dependencies were prepared outside authoring.

### Interpretation and evidence

This experiment exposes a task-prioritization problem for this workload: the current product spends its budget on auxiliary structure/trait obligations and repeat checking before the simple requested APIs. Unsupported trait mapping and conservative invalidation after input edits add work. The ordinary worker instead completed the three simpler API proofs and accurately reported the hard loop contracts as unfinished. The two workflows produced different useful support lemmas, so even the partial work is not interchangeable.

A follow-up should make requested API goals affect scheduling while retaining visible whole-package obligations, avoid repeatedly spending turns on known unsupported mappings, and review which report-only inputs should invalidate proof records. Those are proposed changes, not changes made during this pair. Test any such changes in fresh cases; do not reuse these proof solutions as a new authoring trial.

The mid-verification native preparation test had previously exposed state-lock contention: `status` and verifier/scheduler writes failed immediately when another process held the lock. The published extension added bounded, interruptible waiting for state reconciliation while retaining fail-fast run/worker/verification locks. The real observations above passed with that fix; its development time is not part of either authoring arm.

[Measured artifacts](../../experiments/adler2/results/2026-10-17/) contain aggregate telemetry/settings/timings, source hashes, frozen worker inventories, the actual proof/model sources, exact audited root statements, status transcripts and a hash manifest. Raw model conversations, session databases and authentication diagnostics are not published. [Reproduction helpers](../../experiments/adler2/) include the common task, fresh-case preparation, authoring runners and the recorded baseline's fresh-extraction audit. The prepared libraries, generated Rust models and Lean kernel remain trusted translation infrastructure; this is not a proof of the compiler or translators.

See the [workflow repair follow-up and fresh measurements](adler2-follow-up.md).
