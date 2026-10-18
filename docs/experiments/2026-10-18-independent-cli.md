# Independent CLI redesign (0.2)

## Decision

The user authorized an incompatible redesign as a CLI for humans and external AI assistants. cargo-elicit no longer launches Codex, assigns tasks, schedules work, requests worker returns, or manages approval/submission/revision protocols. A short optional skill explains how an assistant can use the same commands as a human.

The purpose is to make obligations, freshness, failures and preserved evidence visible while leaving proof authoring and proof order to the author. Faster AI authoring and more reliable whole-task completion remain hypotheses requiring an end-to-end comparison.

## Changes

- Replace the old workflow with `init`, `refresh`, `status`, `list`, `show`, `check`, `audit` and `review`.
- Declare selected Rust APIs or independent exact Lean propositions in an editable schema-2 `elicit.toml`.
- Make status/list/show pure saved-snapshot reads. Explicit refresh observes Cargo scope and discovers APIs without compiling Lean proofs. Check/audit automatically refresh before verification.
- Separate machine check, independent fresh audit, semantic review and observation freshness. Unreviewed work can pass; negative review does not erase a proof. Editorial descriptions affect review, not machine proof validity.
- Introduce dependency-aware reuse of checker-produced Lean modules during check. Source, dependency provenance, context/toolchain and output hashes must match. Always rerun exact-type/axiom inspection. Audit uses an isolated fresh build and ignores incremental products.
- Preserve original-source mapping, actual translated argument types, arbitrary-input native coverage checks, transitive axiom restrictions and before/after input guards.
- Preserve per-run receipts, diagnostic sources and raw phase logs outside disposable caches. No CLI command modifies the runtime implementation.
- Remove the AI client, autonomous loop, scheduler, planner, collection protocol and associated tests. Preserve historical experiment data and mark older design documents historical.

## Review findings addressed

Astra helped design the separation and reviewed correctness. Fixes include stable before/after discovery, re-resolved Cargo scope, semantic configuration fingerprints, actual workspace-root configuration tracking, whitespace-safe Lean import parsing, rejection of unsupported import forms, stale product pruning, fresh LLBC extraction and durable diagnostic logs. Tabs in imports and virtual-workspace proposition changes have regression coverage. Pinned Lean 4.35 rejects newline module-name continuation forms; the supported import subset is documented.

## Validation interpretation

Regular tests cover snapshot purity even without tools, dependency reuse and corruption rejection, clean audit isolation, independent review, forbidden axioms, proposition goals and configuration changes during verification. Native tests separately exercise the actual proof kernel; deterministic peers alone are not evidence of kernel acceptance.

The small identity/proposition fixture measures command costs and reuse, not proof discovery. Saved-status latency and faster warm checks cannot establish that authoring a crate proof finishes faster. No new adler2 authoring trial was performed in this change. Earlier adler2 runs and their raw evidence remain unchanged.

Final results and raw validation logs are stored under `experiments/independent-cli/2026-10-18/`.

### Actual Aeneas run

Pinned Aeneas/Charon/Lean accepted both the original identity API contract and an independent `True` format-example proposition. Tests confirmed warm dependency reuse, exact-type/axiom inspection on every check, source/model invalidation, rejected custom axioms, independent proposition acceptance and fresh final audit. Original runtime source was unchanged.

| Command in this small fixture | Wall time |
| --- | ---: |
| First check (empty incremental module cache) | 22.868 s |
| Repeated check | 12.493 s |
| Check after proof-only comment edit | 14.126 s |
| Initial / final fresh audit | 22.981 / 22.164 s |
| Saved status | approximately 0.002 s |

Check still runs extraction/translation and fingerprints the verification context. It is not an instantaneous compiler frontend. Status performs neither operation. These costs are not forecasts for adler2 or other crates.

### Final checks

- 23 regular tests passed (17 unit, five workflow and one process test); three opt-in native tests are ignored by default.
- Formatting, Clippy with warnings denied and Rust 1.86 checks passed.
- Native Aeneas passed, native Creusot passed with its installed data directory selected, and native Kani passed with the actual installation bin directory on PATH.
- The initial all-native run returned failure because a custom Kani shell wrapper hid its installation from provenance tracking. Native obligations succeeded, but cargo-elicit refused acceptance without tracked installation provenance. The retry used the real installation path and passed. Both logs are preserved. No acceptance guard was relaxed.
- A targeted virtual-workspace regression also passed after adding an editorial-description change: the description preserves machine evidence, while a changed formal proposition during checking invalidates it.

Reproduction requires the pinned native tools documented in the repository. These environment-specific paths describe this validation machine, not hard-coded product configuration.
