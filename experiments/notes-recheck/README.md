# Operational-note invalidation experiment

This experiment reuses the completed five-API adler2 2.0.1 proofs from the independent CLI authoring experiment. It measures checking after operational-report edits, **not** fresh proof authoring or whole-crate verification. Original authoring records are never modified.

The runner uses a fixed CLI executable and inherited pinned verifier environment, prepares two independent workspace copies without evidence/build caches, and executes the arms sequentially:

1. Root-report baseline: leave `PROGRESS.md`, `FINAL_REPORT.md` and `verification-inventory.json` at the project root.
2. Reserved-notes arm: move those three operational artifacts to `.elicit/notes/` before the first audit.

Each arm first obtains a fresh five-API audit and performs a `check` to populate its checker-owned module cache. This setup is recorded separately. The measured sequence is:

- Edit progress; refresh and read status; check all five APIs.
- Edit the final report; refresh and read status.
- In the baseline, re-audit because the report edit changed tracked inputs. In the notes arm, verify that the original fresh audit receipt remains valid and do not request another audit.

An explicit `audit` always rebuilds; the runner never changes that command's semantics. Skipping an unnecessary command in the notes workflow is an explicit protocol choice supported by refreshed input bindings. The runner asserts successful full-contract evidence for all five APIs, unchanged runtime/proof/config hashes, stable inputs in the notes arm, retained audit receipt, and at least one cache hit after the notes edit. Detailed changed-input reports and per-stage events are retained.

```sh
# Configure the pinned Rust/Charon/Aeneas/Lean toolchain first.
python experiments/notes-recheck/run.py \
  --source /path/to/completed/adler2-case \
  --binary /path/to/frozen/cargo-elicit \
  --output /path/to/new-experiment-directory
```

Do not place output inside the source case. The runner refuses existing case/result directories rather than overwriting them. The binary must remain unchanged for the whole experiment. Operational notes must never be read by build/proof code; this is a declared trust boundary, not a claim of automatic dependency detection.

A single sequential pair has order/cache/environment effects. Final refresh/status observations concern saved snapshots, not later unobserved edits. The outcome cannot establish a general authoring speedup or recovery benefit.

Measured results: [2026-10-19 report](../../docs/experiments/adler2-operational-notes.md). Public artifacts are curated metrics, input hashes, stage events and audit/acceptance observations. Complete raw workspaces and diagnostic archives remain local. Operational notes still require explicit local archival.
