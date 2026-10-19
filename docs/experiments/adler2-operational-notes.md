# adler2 operational-note invalidation experiment

Moving operational records to the existing reserved `.elicit/notes/` boundary avoided unnecessary invalidation in this completed-proof workload. After a progress edit, `check` fell from **80.711 s to 13.025 s**, reusing **18 Lean modules**. A final report edit retained the original fresh audit, avoiding another **77.991 s** audit. The recorded post-edit workflow fell from **169.662 s to 23.913 s** (145.749 s, 85.9% less).

This is a checking/workflow experiment using saved proofs, **not** a fresh proof-authoring comparison, whole-crate verification result, or prediction of a new end-to-end Codex time.

## Implementation

The fingerprint walker already excluded `.elicit/`. No cache algorithm was relaxed and no Markdown wildcard or adler2-specific filename exemption was introduced. The change establishes `.elicit/notes/` as the official place for operational progress, handoff and final reports; `init` and `cargo elicit notes` create its guide. Other project files remain conservatively tracked.

New evidence summaries save input-file hashes. Pure `status` reads identify the audit receipt and whether it is valid, failed, missing or stale at the saved snapshot. Stale reasons compare audit inputs against the saved observation, surviving repeated `refresh` calls. Older summaries explicitly lack detailed change information. The skill now saves semantic artifacts before the final audit, writes reports in the reserved notes directory, and requires new final evidence only when semantic/tracked inputs change. Explicit `audit` always rebuilds.

Operational notes must never be read by Rust, build scripts, proc macros, Lean or verifier code. This is an explicit non-semantic trust boundary, not automatic dependency discovery or sandbox enforcement. Notes must be archived separately because `.elicit` is usually Git-ignored.

## Protocol and scope

[Runner and protocol](../../experiments/notes-recheck/README.md) and [curated records](../../experiments/notes-recheck/results/2026-10-19/) are preserved. The source was the completed treatment case from the [independent 0.2 authoring experiment](adler2-independent-cli.md). Its [original crate](../../experiments/independent-cli-authoring/results/2026-10-18/original/) and [authored source](../../experiments/independent-cli-authoring/results/2026-10-18/with/author-sources/) were already published; this experiment records their semantic-file hashes rather than exporting another complete workspace.

Both arms used the **same frozen new CLI binary**, pinned Rust/Charon/Aeneas/Lean environment and unchanged original optimized Rust, contract, proof and configuration bytes. This isolates report placement and workflow; it is not a head-to-head comparison of old and new executables. Binary and production-source identities are in `protocol.json` and `production-source-identity.json`. No AI, proof generation, runtime edits, contract weakening or timed configuration changes occurred.

The arms ran sequentially in separate fresh copies without inherited evidence/build caches:

1. Root-report baseline: retain `PROGRESS.md`, `FINAL_REPORT.md` and `verification-inventory.json` at the project root.
2. Reserved-notes arm: move those three operational artifacts under `.elicit/notes/` before initial validation.

Each first obtained a fresh audit of all five APIs and warmed its own incremental cache with `check`. Preparation is recorded but excluded from the post-edit workflow total. Then each updated progress, ran `refresh`, read `status --all --json`, and ran `check`. Each updated the final report, refreshed and read status again. The root-report baseline requested another audit to restore final evidence; the notes arm observed that the original fresh receipt remained valid and did not request another audit. Nothing changed the behavior of the explicit `audit` command.

The five selected arbitrary-input API contracts were `new`, `checksum`, `from_checksum`, `write_slice`, and `adler32_slice`. Every executed check/audit succeeded for all five at full-contract scope; source/proof/config hashes stayed identical across both arms. This does not cover generic I/O, Hasher laws, derived traits, other build scopes or all crate semantics.

## Measurements

| Measurement | Root reports | Reserved notes |
| --- | ---: | ---: |
| Initial fresh audit, setup | 88.419 s | 85.359 s |
| Initial cold incremental check, setup | 79.125 s | 79.286 s |
| Progress refresh | 5.376 s | 5.440 s |
| Progress status | 0.002 s | 0.002 s |
| Check after progress edit | 80.711 s | 13.025 s |
| Cached Lean modules after progress edit | 0 | 18 |
| Native stage time within that check | 72.212 s | 4.510 s |
| Final-report refresh | 5.580 s | 5.443 s |
| Final-report status | 0.003 s | 0.002 s |
| Additional final audit | 77.991 s | Not requested; original receipt still valid |
| **Post-edit workflow total** | **169.662 s** | **23.913 s** |

The progress checks each recorded 21 native stages. In the notes arm, the 18 cached stages were Lean modules; extraction, translation and the exact-type/axiom audit still ran. Zero-time reuse does not mean verification was bypassed. The two initial audits and initial incremental checks had zero cache hits, as expected for separate fresh audit and incremental workspaces.

After the root progress edit, all five audit bindings became stale, with `PROGRESS.md` identified as modified. After the final report edit, both progress and final-report paths were listed relative to the original audit, not just the immediately preceding refresh. In the notes arm, every refreshed audit state stayed `valid_at_snapshot`, the input digest stayed identical through all edits, and each API continued to reference the initial fresh audit receipt.

## Interpretation and limits

The causal mechanism is visible in the hashes, audit bindings and module-stage events: operational-note updates no longer change tracked input context. This demonstrates a concrete reduction in unnecessary checking for this scenario without weakening the proof acceptance policy. The implementation does not fix every historical zero-cache case; genuine contract, model, proof, tool or build-context changes can still require recompilation.

The pair was not repeated or randomized. Environment warm-up and sequential execution affect timings, and post-edit totals exclude initial audit/cache preparation. The avoided audit is a workflow change, not a faster fresh audit. Do not subtract these seconds from historical authoring runs and claim a newly measured authoring time. The current result does not establish interruption/recovery benefits or a general proof-authoring speedup.

All raw workspaces, native evidence and command output remain available in the local run directory `/workspace/elicit-notes-recheck-2026-10-19`. Public records contain curated numerical commands, input identities, audit observations, native-stage events and proof-acceptance fields; complete raw workspace/diagnostic archives are not published.

Validation also passed: 28 ordinary Rust tests (including the notes/cache/invalidation regression), Clippy with warnings denied, formatting and diff checks. The adler2 experiment itself supplies the live pinned native verifier checks for this change; mock tests are not the basis for the five-API acceptance claim.
