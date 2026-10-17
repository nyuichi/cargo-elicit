# Adler-32 follow-up after workflow repairs

**Status: implementation validated; fresh authoring pair not yet measured.** The [first pair](adler2-comparison.md) exposed unsupported-trait retry loops, coarse proof invalidation, unnecessary worker turns for checking existing proofs, and successful audits omitted from the ledger. This follow-up fixes those mechanisms without prioritizing easy APIs or importing prior proof solutions.

## Changes under test

- Unsupported Aeneas trait implementations stay visible in the denominator/status but are not automatically scheduled. Their retained contracts do not poison supported method submissions.
- An explicit worker issue blocks that target until relevant semantic context changes or an explicit retry. Diagnostics alone are not forward progress. Specify/recheck retry keys exclude unrelated proof-module edits.
- Existing proof artifacts receive a fresh checker invocation before asking a worker to repair them. All independently successful, ready audited contracts are recorded from that invocation.
- Operational report edits can retain evidence in simple crates. Custom builds/compiler hooks, active dependencies, Rust file inputs/path attributes and Lean file-reading/metaprograms keep conservative invalidation. Other proof/source changes still require a real kernel check. Shared proof-module changes may trigger a shared check; this is not arbitrary per-theorem proof reuse.
- Task events record worker/checker boundaries and input changes with file hashes. Failed checks expose a bounded phase/cache reference rather than storing arbitrary native output.

Whole-package obligations, source identity, contract meaning, theorem matching and allowed axioms are unchanged. An unsupported target prevents whole-package completion. No contracts have been weakened and no already solved Adler proof was installed in either case.

## Validation before timing

Rust regression coverage includes issue blockers, unsupported mappings, semantic retry keys, diagnostic-only changes, report-only changes and conservative build/macro/Lean-I/O fallbacks. Native Aeneas/Lean fixtures exercise methods, structs, mutation, slices and helpers, interruption/resume, cache deletion, status during verification, shared audit recording and checker-only refresh. The negative sorry/project-axiom/wrong-contract cases remain mandatory. Formatting, Clippy and Rust 1.86 checks pass.

Both freshly prepared upstream cases pass six unit tests and four doctests. Their Rust sources, manifest, lockfile and common benchmark task match the first pair exactly. Tool installation and these validation runs are outside authoring time.

## Fixed measurement protocol

Run a fresh ordinary-Codex arm, then the repaired product arm, sequentially with the same 1,800-second work budget and the same Codex/model/reasoning/permissions as each other. Preserve the first pair as historical evidence rather than reusing its baseline or proof files. Use the [same common goals and acceptance rules](../../experiments/adler2/TASK.md).

The observer now preplans an actual status/text/JSON/list checkpoint after the first successful helper or API record, as well as first-selected-API/halfway-coverage, ten-minute fallback and termination. No human proof hints follow an observation. Observation time, initialization and checker-only refreshes count toward end-to-end time. Freeze terminal events and ledger before post-trial audits. No proof-authoring speedup is inferred from deterministic fixture timings.

Independently check unchanged original sources, fresh extracted model correspondence, exact theorem statements, successful execution and functional postconditions, proof rebuilds and transitive axioms limited to propext/Classical.choice/Quot.sound. Report selected APIs separately from helper results and whole-package completion. Use actual token events; cached input is a subset. A single pair remains exploratory.

## Results

Pending actual authoring and independent artifact audits. Do not substitute the first pair, native fixtures or existing proof solutions for this measurement.
