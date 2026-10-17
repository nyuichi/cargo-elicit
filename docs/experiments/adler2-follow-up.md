# Adler-32 follow-up after workflow repairs

**Status: model-dependency repair validated; fresh final authoring trial pending.** The [first pair](adler2-comparison.md) exposed unsupported-trait retry loops, coarse proof invalidation, unnecessary worker turns for checking existing proofs, and successful audits omitted from the ledger. This follow-up fixes those mechanisms without prioritizing easy APIs or importing prior proof solutions.

## Changes under test

- Unsupported Aeneas trait implementations stay visible in the denominator/status but are not automatically scheduled. Their retained contracts do not poison supported method submissions.
- An explicit worker issue blocks that target until relevant semantic context changes or an explicit retry. Diagnostics alone are not forward progress. Specify/recheck retry keys exclude unrelated proof-module edits.
- Existing proof artifacts receive a fresh checker invocation before asking a worker to repair them. All independently successful, ready audited contracts are recorded from that invocation.
- Operational report edits can retain evidence in simple crates. Custom builds/compiler hooks, active dependencies, Rust file inputs/path attributes and Lean file-reading/metaprograms keep conservative invalidation. Other proof/source changes still require a real kernel check. Shared proof-module changes may trigger a shared check; this is not arbitrary per-theorem proof reuse.
- Task events record worker/checker boundaries and input changes with file hashes. Failed checks expose a bounded phase/cache reference rather than storing arbitrary native output.

Whole-package obligations, source identity, contract meaning, theorem matching and allowed axioms are unchanged. An unsupported target prevents whole-package completion. No contracts have been weakened and no already solved Adler proof was installed in either case.

## Validation before timing

Rust regression coverage includes issue blockers, unsupported mappings, semantic retry keys, diagnostic-only changes, report-only changes and conservative build/macro/Lean-I/O fallbacks. Native Aeneas/Lean fixtures exercise methods, structs, mutation, slices and helpers, interruption/resume, cache deletion, status during verification, shared audit recording and checker-only refresh. The negative sorry/project-axiom/wrong-contract cases remain mandatory. Formatting, Clippy, Rust 1.86 and 61 regular tests pass. Both real Aeneas fixtures passed together in 801.82 seconds with model-extension retention, changed-model rejection, unsupported-declaration fallback, proof negatives and cache deletion/resume. A final real-tool documentation regression passed in 102.55 seconds: a README edit changes the conservative proof input while keeping existing specification fingerprints and readiness unchanged.

Both freshly prepared upstream cases pass six unit tests and four doctests. Their Rust sources, manifest, lockfile and common benchmark task match the first pair exactly. Tool installation and these validation runs are outside authoring time.

## Fixed measurement protocol

Run a fresh ordinary-Codex arm, then the repaired product arm, sequentially with the same 1,800-second work budget and the same Codex/model/reasoning/permissions as each other. Preserve the first pair as historical evidence rather than reusing its baseline or proof files. Use the [same common goals and acceptance rules](../../experiments/adler2/TASK.md).

The observer now preplans an actual status/text/JSON/list checkpoint after the first successful helper or API record, as well as first-selected-API/halfway-coverage, ten-minute fallback and termination. No human proof hints follow an observation. Observation time, initialization and checker-only refreshes count toward end-to-end time. Freeze terminal events and ledger before post-trial audits. No proof-authoring speedup is inferred from deterministic fixture timings.

Independently check unchanged original sources, fresh extracted model correspondence, exact theorem statements, successful execution and functional postconditions, proof rebuilds and transitive axioms limited to propext/Classical.choice/Quot.sound. Report selected APIs separately from helper results and whole-package completion. Use actual token events; cached input is a subset. A single pair remains exploratory.

## Excluded candidate run

A candidate product trial was stopped after 517.39 seconds, before scoring. Identical source files acquired different fingerprints through Cargo versus direct invocation: environment iteration order, Cargo’s Rustup selector and an overridden Lean path were being hashed as effective inputs. The additional repair canonicalizes environment order, ignores the overridden path, and uses the resolved compiler identity for simple crates while retaining selectors for complex builds. An isolated Cargo/direct/Cargo probe now produces identical fingerprints. Context events now identify hashed context components as well as changed files, without exposing environment values. This interrupted trial is retained as diagnostic cost, not silently counted as a valid paired result. Its proof/specification artifacts are not reused. The completed ordinary arm remains the control because it does not use product fingerprints.

## First complete repaired trial and additional repair

The ordinary arm completed in 860.09 seconds and independently proved 3/5 selected APIs, with 27 audited theorem roots across API, conditional and mathematical support results. The first complete repaired product trial used source `86271b9df690eb5e3be95eeb2839535aa1e0811a` and ran for 1,806.74 seconds including terminal observation. It submitted meaningful specifications and saved nine valid method proofs, including the same three selected APIs, but its timed ledger accepted none: rechecks of other specifications delayed the first official check until 27 seconds before the work deadline. Interruption was then incorrectly labeled as input change. These are measured results, not excluded observations.

A post-trial wrapper check of the untouched saved proof module accepted all nine method contracts. This does not turn the frozen timed result into 3/5. Two further repairs address the mechanism: check declared, ready proof artifacts before opening unrelated specification work, without asking an AI to solve a different API first; and classify interruption or failed input inspection separately from actual input changes. Interrupted checks accept no new proof and exit cleanly with code 130. Fresh extraction, exact propositions, kernel checking and standard-axiom restrictions remain mandatory.

## Results

The checkpoint-repaired trial (`5ea3258a684596b597c16f61f0ad7eb991e60c58`) finished in 1,807.70 seconds. It recorded nine method contracts during the timer and observed selected API coverage of 3/5 at 1,576.2 seconds. Actual status snapshots were taken after the first helper (523.1 seconds), at ten minutes, after the first selected API (1,106.0 seconds), at 3/5 coverage and at termination. Observation cost was 43.88 seconds, included in elapsed time. No unsupported trait task or arithmetic-helper reauthoring turn occurred.

The last `compute` task appended eleven independent model/lemma declarations to `Specs.lean`. Whole-file model hashing then marked every existing specification stale, leaving the terminal ledger at 0/19 current despite nine earlier successes. A fresh wrapper check accepted all nine unchanged saved method proofs against the final models. Comparing Lean's elaborated declarations found no changed prior model or contract, confirming that those rechecks were unnecessary. This is another retained measurement, not an excluded result.

The additional repair tracks elaborated contract/model dependencies and preserves type-review bindings under independent model extensions. It retains fresh proof checks and conservative fallbacks for dynamic or unsupported inputs. Its actual fresh authoring trial and final audit are pending. All earlier candidate costs/results remain part of the report; no prepared specification or proof is imported into the next case.
