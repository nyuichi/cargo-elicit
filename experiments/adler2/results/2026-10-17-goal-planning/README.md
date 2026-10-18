# Goal-directed planning comparison: 2026-10-17

Read the [full report](../../../../docs/experiments/adler2-goal-planning.md). These are proof solutions: withhold them from future authoring workers.

- `without/`: fresh ordinary control, 1,704.23 seconds, five independently accepted selected APIs, including original optimized loops. `BENCHMARK_COMPLETE.json` is a self-report; acceptance comes from the separate audit.
- `with/`: fresh selected-root cargo elicit run, 1,808.50 seconds including terminal observation. Saved proofs cover three selected APIs and four helpers; the frozen terminal ledger covers one selected API and four methods overall. Three chunk-list checkpoints are intermediate facts, not API completion or checked loop invariants.
- `with/terminal-state.toml` and `terminal-events.jsonl` were frozen before independent audits. `workflow-analysis.json` and actual status observations explain scheduling, stale results and repeated checks. Post-trial audits do not backfill this ledger.
- Each arm's `audit/` contains fresh model copies, independent exact-root audit modules, compiler logs, command timings and `summary.json`. Extraction intermediates and compiled binaries are omitted; the auditors regenerate them. Missing product roots are retained as failures.
- `telemetry-summary.json` contains whitelisted actual model/settings/counters; raw model messages, command telemetry, auth material and session databases are not published.
- `source-freezes/` pins tested source/tree/binary. `validation/` records the real native regressions. `measurements.json` records acceptance separately from model claims. `sha256.json` hashes every other evidence file.

Subject: unchanged crates.io adler2 2.0.1, archive SHA-256 `320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa`, upstream license 0BSD OR MIT OR Apache-2.0. Upstream licenses are in `subject-licenses/`. Tool pins, scope and trusted boundaries are in the report. Whole-crate verification is not claimed.

## Reproduce independent audits

Prepare original cases with `experiments/adler2/prepare.py`. Install the report's pinned Aeneas/Charon/Lean environment, including its Lean backend dependencies, and put `charon`, `aeneas`, `lake` and `lean` on PATH. Set `AENEAS_HOME` to the pinned Aeneas checkout. Do not run authoring workers with these saved solutions visible.

Copy `without/verification/` into the fresh ordinary case, then run from the cargo-elicit repository:

```sh
python3 experiments/adler2/audit-goal-planning-baseline.py \
  --root /path/to/adler2-without \
  --review experiments/adler2/results/2026-10-17-goal-planning/without/review.json \
  --out /path/to/new-ordinary-audit
```

The review file pins exact proof/model bytes and module order. The auditor performs selected-root extraction from the original source, checks every original selected body and overflow checks, byte-compares the generated model, rebuilds all modules and checks 52 theorem roots/axioms. Read the actual API types and scalar definitions; theorem counts alone are not coverage.

Copy `with/elicit-proofs/aeneas/` into the fresh product case, then run:

```sh
python3 experiments/adler2/audit-goal-planning-product.py \
  --root /path/to/adler2-with \
  --cache experiments/adler2/results/2026-10-17-goal-planning/with/cache \
  --plan experiments/adler2/results/2026-10-17-goal-planning/with/terminal-state.toml \
  --out /path/to/new-product-audit
```

This independently regenerates the original Rust model and compares it with the frozen wrapper model, rebuilds saved modules and checks every declared method theorem and planned lemma against the exact proposition with standard axioms only. Missing product theorems are expected failures. Manually review emitted contracts and reference definitions before assigning API coverage. No proof-authoring or new tactics are performed by these audits. Audit paths/logs preserve the actual execution environment; adapt PATH rather than treating machine paths as installation requirements.
