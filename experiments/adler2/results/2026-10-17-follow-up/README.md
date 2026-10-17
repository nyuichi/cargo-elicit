# Adler-32 workflow-repair follow-up: 2026-10-17

Read the [full report](../../../../docs/experiments/adler2-follow-up.md). These are proof solutions and must be withheld from future authoring workers.

- `measurements.json` retains the fresh ordinary control, every measured product iteration, the excluded environment candidate, reported usage/settings and independent audits.
- `without/` is the fresh ordinary arm: three selected APIs, 27 independently audited roots. Conditional/model lemmas are not optimized-loop correctness.
- `with/` is the final repaired product arm: eight explicit method contracts, including two selected APIs, plus two codec roundtrip theorems. `terminal-state.toml` and `terminal-events.jsonl` are frozen before post-trial audits.
- `intermediate-*/` retain the two valid intermediate measurements. They are not discarded because their terminal ledgers were poor.
- `excluded-environment-candidate/` retains the diagnosed early-aborted candidate and its cost, plus the Cargo/direct fingerprint reproduction.
- `post-trial-extension-replay/` is a deterministic, non-scored replay on the saved final subject: append an unused model, retain ten ready specifications, recheck eight contracts, then restore source bytes. No AI or new API proof authoring occurred.
- `source-freezes/` pins each tested source and installed binary. `sha256.json` hashes all published evidence.

The final timed ledger records 2/5 selected APIs and 8/19 package obligations, with two types specified. `new` was not submitted before the cutoff; `write_slice` and `adler32_slice` remain unproved. A separate non-scored audit observation in `with/AliasObservation.lean` assigns the already saved default-method proof directly to the exact `new` goal by kernel definitional equality. This gives an additional logical implication without a new proof tactic; it does not backfill the frozen ledger or milestones.

All original Rust source and manifest bytes are preserved. Subject: crates.io adler2 2.0.1, archive SHA-256 `320119579fcad9c21884f5c4861d16174d0e06250625266f50fe6898340abefa`, upstream license 0BSD OR MIT OR Apache-2.0. Tool pins and trusted boundaries are in the report. No arm proves the whole computation core or crate.

## Reproduce the ordinary audit

Prepare a fresh subject with `experiments/adler2/prepare.py`, configure the pinned Aeneas/Charon/Lean environment, and place the saved ordinary modules under `verification/` with `Adler2.lean` under `verification/generated/`. Run:

```sh
python3 experiments/adler2/audit-follow-up-baseline.py --root /path/to/adler2-without --out /path/to/new-audit
```

The auditor pins the manually reviewed proof bytes, fresh-extracts the original code, compares the model, checks extraction safety and transparent original API bodies, rebuilds all modules and prints exact root types/axioms.

## Reproduce the product audit

Use tested source `bd7c422e888379f4a48764ad64ed9d16d59d4cc7`. Initialize a fresh original subject with `cargo elicit init --tool aeneas`; copy `with/{Specs.lean,Proofs.lean,contracts.toml}` to `elicit-proofs/aeneas/`. Review/submit each declared method with its current `agent show` revision, then use `agent verify` for those IDs. The wrapper must freshly extract the original source and generate the contract propositions; archived generated modules are evidence for inspection, not substitutes for extraction.

For a separate kernel rebuild of that fresh wrapper cache:

```sh
python3 experiments/adler2/audit-follow-up-product.py --root /path/to/adler2-with --cache /path/to/fresh/.elicit/cache/verification-XXXXXX --out /path/to/new-audit
```

This prints actual contract definitions and checks exact theorem assignments and allowed transitive axioms. API acceptance additionally requires reading the printed contracts; matching names alone is insufficient. The saved support and alias audit modules can be checked in that output directory with the same prepared backend on `LEAN_PATH`.
