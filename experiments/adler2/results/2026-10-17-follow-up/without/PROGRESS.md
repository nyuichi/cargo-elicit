# Verification progress

Partial verification, not completion of the five-API goal or the crate.

The original `src/algo.rs`, `src/lib.rs`, and upstream `Cargo.toml` are byte-for-byte preserved. Their initial SHA-256 values are in `verification/source-sha256.txt`; the final hash check and Git diff passed. No runtime implementation was copied or rewritten. Work used one worker and direct pinned Charon, Aeneas, and Lean invocations, without cargo-elicit.

## Checked results

Lean checked successful execution and the complete requested contracts for:

- `Adler32::new`: returns `(1, 0)`.
- `Adler32::checksum`: returns `65536*b+a` for every pair of `u16` fields.
- `Adler32::from_checksum`: decodes every `u32` into its low and high 16-bit halves.

Both codec roundtrips and checksum save/restore/resume equivalence are proved against the generated functions. Resume equivalence preserves the entire result of the subsequent update; it does not prove that this update succeeds.

Supporting proofs cover the original four-byte loader (including indexing and widening), all three vector helpers under their stated arithmetic/divisor preconditions, two empty-iterator loop exits, scalar specification normalization and splitting, worst-case lane/scalar bounds, and four-lane algebra. These are partial building blocks, not a proof of the optimized compute body.

The convenience API and source split law have checked **conditional** proofs assuming the full `WriteSliceCorrect` proposition. That proposition has no proof. The split contract explicitly requires `(xs.val ++ ys.val).length <= Usize.max`; neither input's separate representability suffices.

## Unsupported obligations

`Adler32::write_slice` and `adler32_slice` remain unverified for the requested universal successful-execution and recurrence contracts. No normalized-seed assumption or artificial input-length restriction was substituted. Contracts remain in `CoreLemmas.lean`, including normalization on an empty update.

Missing work includes terminating invariants for all six extracted loops, propagation of lane arithmetic bounds between chunks, slice/chunk decomposition and helper-call preconditions, correctness of delayed scalar contributions and lane recombination in the machine model, serial remainder correctness, and final casts/normalization. Even the empty-update normalization contract is not yet proved against compute. `verification/logs/empty-attempt.log` preserves an unsuccessful attempt; the failing theorem is not included in checked proof modules. The two proved empty-iterator exits do not establish empty-update normalization.

The algebraic identity and overflow inequalities alone do not establish these missing links. Full source split-update correctness therefore also remains open. No counterexample was found; finite tests cannot establish the universal claim.

Generic BufRead I/O, Hasher laws, derived traits, no_std, and alternative targets are unverified. Running the upstream tests, including its I/O test, does not change that status.

## Reproduction and evidence

Run from this workspace:

```sh
verification/extract.sh
verification/check.sh
verification/regression.sh
```

Extraction uses the original Cargo manifest, `--preset=aeneas --sysroot=default`, and `x86_64-unknown-linux-gnu`. It excludes only generic I/O and derived Debug formatting from the model. The fresh LLBC and Lean model are in `verification/generated/`; the generated Lean file is unedited and has no project axioms. Tool versions and generated hashes are persisted.

The final Lean check passed, including an axiom audit of 27 theorem roots. Only `propext`, `Classical.choice`, and `Quot.sound` occur. There are no proof holes, project axioms, or native-decision axioms in the checked artifacts. See `verification/logs/final-check.log` and `axioms.log`.

The original Rust passed its six upstream unit tests. A harness linked to the original crate, with overflow checks enabled, passed 2,888 seeded recurrence cases, 66 convenience cases, and 14,256 split/save/resume cases. Seeds include halves 65521 through 65535; lengths exercise all short tails and multiple 22,208-byte chunk boundaries. These are tests, not proofs. See `verification/regression.rs` and the logs.

The trusted boundary includes Rust-to-LLBC extraction, Aeneas translation, standard Aeneas library models, the Lean kernel and its permitted axioms, and the pinned host/target toolchain. Translation and model compilation alone are not counted as API proofs. The precise theorem inventory and remaining obligations are in `verification-inventory.json`.
