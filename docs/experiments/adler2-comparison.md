# cargo-elicit versus ordinary Codex: Adler-32 experiment

**Status: prepared, not measured.** A live Codex authentication smoke test returned HTTP 401 and an expired-session / sign-in-again error on 2026-10-17. No successful AI authoring trial has run. Wall-time, token and coverage comparisons remain unmeasured. This environment failure must not be scored as either arm's performance.

## Reference and subject selection

[proofs.rs report 7](https://proofs.rs/report/7) covers the five computation-core APIs of adler2 2.0.1, plus supplementary codec, split and resumption laws. Its evidence lists 46 audited theorem roots. The recorded **680.2 seconds** are the time to build and audit existing proofs, including dependency preparation; they are not the time spent authoring them. The authoring evidence describes a multi-model workflow, whereas this experiment uses one Codex worker in both arms. We cannot infer an authoring speedup from that recorder time.

An initial candidate, adler 1.0.2, has 433 Rust lines across two files and very similar APIs and optimized loops. Its six unit tests and four doctests passed, and Charon extraction succeeded. However, the current pinned Aeneas could not translate its old `zip`-based lane helper, so it is unsuitable for a comparison with this shared toolchain. gcd 2.3.0 was also inspected; its macro-generated APIs and different algorithm make it a less comparable workload.

The final intended subject is therefore **adler2 2.0.1 itself**, starting from the unmodified crates.io release with no existing proof files. It has 442 Rust lines across `src/lib.rs` and `src/algo.rs`, including comments/tests. Its six unit tests and four doctests passed, and the shared pinned Charon/Aeneas installation successfully translated it. The downloaded archive's SHA-256 matches the official registry index:

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
| Model and reasoning | Inherited settings, record resolved values | Same inherited settings, check resolved values match |
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

## Current results

| Metric | With cargo-elicit | Without cargo-elicit |
|---|---|---|
| Successful live authoring trial | Not started | Not started |
| Authoring elapsed time | Unmeasured | Unmeasured |
| Tokens | Unmeasured | Unmeasured |
| Independently accepted Adler-32 APIs | Unmeasured | Unmeasured |
| Mid-experiment status observations | Pending | Pending |

The isolated case directories, common prompt, source manifests, telemetry forwarding and timed observer scripts have been prepared in the execution workspace. Resume only after a live Codex request succeeds and the expanded adapter's validation is complete. Populate measured results from actual runs; do not substitute native fixture timings or the reference report's recorder duration.

The mid-verification native observation test exposed a state-lock contention failure: `status` and verifier/scheduler state writes previously failed immediately when another process held the state lock. The extension adds bounded, interruptible waiting for state transactions/reconciliation while retaining fail-fast run/worker/verification locks. This preparation fix is recorded separately from timed authoring results.

Reproduction helpers and the common prompt are checked in under [experiments/adler2](../../experiments/adler2). Preparation was exercised against the registry archive and verified to produce identical original Rust sources, manifests and lockfiles in fresh case directories. The live runner remains gated on a successful Codex request.
