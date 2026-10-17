# Recorded exploratory pair: 2026-10-17

Read the [analysis and limitations](../../../../docs/experiments/adler2-comparison.md). This directory contains proof solutions: withhold it from workers in future authoring trials.

- `measurements.json`: actual elapsed times, cumulative token aggregates, matching resolved settings, source hashes, observations and separate independent audit summaries.
- `without/`: the ordinary worker's proof module, original generated model and frozen self-reported inventory.
- `with/`: the product worker's frozen proof module, contracts and unsupported laws, plus the freshly generated model/contract modules used in post-trial checking. The worker inventory is a checkpoint, not authoritative proof acceptance.
- `audit/`: exact theorem types and transitive axiom output from independent Lean checks.
- `status/`: actual product text status at ten minutes, an additional helper-progress checkpoint and termination. JSON observations were also captured; the measurements preserve their commands and durations.
- `sha256.json`: hashes of the saved evidence files.

The accepted common result is 3/5 APIs without cargo-elicit and 0/5 with it. The saved product artifacts additionally prove three arithmetic helper methods under explicit nonzero/overflow preconditions. Fresh post-trial auditing does not replace the frozen timed ledger: it showed 1/19 current verified targets at termination.

The original subject is adler2 2.0.1 from crates.io, licensed upstream under 0BSD OR MIT OR Apache-2.0. The archive hash, source hashes and tool pins are in the report and measurements. Generated Lean reflects the original source; no Rust implementation rewrite was used.

## Recheck the ordinary arm

Use the pinned Aeneas/Charon/Lean installation, with its prepared backend on disk and `AENEAS_HOME`, `ELAN_HOME`, `CARGO_HOME`, `RUSTUP_HOME` and tool PATH configured as in the experiment. Prepare a fresh upstream case with `../../prepare.py`. Copy `without/Proofs.lean` into its `verification/` directory and `without/Adler2.lean` into `verification/selected/`. Then run:

```sh
python3 experiments/adler2/audit-baseline.py \
  --root /path/to/adler2-without --out /path/to/new-audit-directory
```

This auditor is specific to the recorded, manually reviewed proof artifact and original source hashes. It rejects changed proof bytes rather than assuming a matching theorem name means the same contract. It performs fresh extraction, compares generated models, rebuilds both modules, checks exact root types/axioms, and checks original source contents, transparent API bodies and extraction safety settings. Contract interpretation still requires reading the emitted theorem types.

## Recheck product helpers

Initialize a fresh case with the tested cargo-elicit version and Aeneas backend. Copy the saved `with/Proofs.lean` and `with/contracts.toml` to `elicit-proofs/aeneas/`. Use `agent show` and submit the three explicit helper method IDs in the contracts file with their current revisions. Then run `agent verify` for those IDs with the current revision. Generated models/contracts must be freshly produced by the wrapper, not edited or accepted solely because copies are included here. Do not submit the archived unsupported trait laws as successfully mapped targets.

The recorded post-trial check also copied the freshly generated modules into a separate audit directory, rebuilt `ElicitGenerated`, `ElicitAdapter`, `ElicitContracts` and `Proofs` directly with Lean and the standard prepared backend, and checked these assignments:

```lean
import Proofs
theorem Audit_remainder : ElicitContracts.c_f321f3834d6c15c0 :=
  U32X4Proofs.remainder_method
theorem Audit_multiply : ElicitContracts.c_6c7496278e307fe7 :=
  U32X4Proofs.multiply_method
theorem Audit_add : ElicitContracts.c_20b5198a3591154d :=
  U32X4Proofs.add_method
#print axioms Audit_remainder
#print axioms Audit_multiply
#print axioms Audit_add
```

Only `propext`, `Classical.choice` and `Quot.sound` occurred. These are helper proofs, not proofs of the optimized loops, their caller-side bounds or the five requested APIs.
