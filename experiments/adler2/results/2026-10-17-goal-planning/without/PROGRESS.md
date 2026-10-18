# Verification result

The five requested API contracts are proved against a fresh, unedited Charon/Aeneas translation of the original adler2 2.0.1 sources. `src/` and Cargo.toml remain byte-for-byte unchanged (`git diff --exit-code -- src Cargo.toml` succeeds).

The theorem roots are `Verification.new_correct`, `checksum_correct`, and `from_checksum_correct` in `verification/Proofs.lean`, and `Verification.write_slice_correct` and `adler32_slice_correct` in `verification/APIs.lean`. The computation root is `Verification.compute_correct` in `verification/Core.lean`. It proves successful execution and exact equality with the scalar Adler recurrence for every representable seed and byte slice. Both halves are normalized even on an empty update. There is no normalized-seed assumption or artificial bound on total slice length.

The proof covers the original optimized body, all six loop functions, four-lane helpers, machine arithmetic, indexing, casts, and standard slice/iterator calls. The 5,552-group and 22,208-byte bounds arise only from the implementation's internal chunks. `Spec.lean` defines the scalar recurrence; `Chunks`, `Loops`, `Tail`, `Algebra`, `Outer`, and `Micro` supply structural, termination, arithmetic and modular correctness lemmas.

Both codec roundtrips are proved. `split_equivalence` compares successful one-shot and split executions and explicitly assumes representable concatenation. `resume_equivalence` compares direct continuation with checksum save/restore/continuation; `save_restore_resume_correct` also establishes the two-part scalar law with explicit concatenation representability.

Run `./verification/check.sh` to rebuild the generated model and every proof module against the installed prepared Aeneas Lean backend. Exact extraction commands, tool versions, hashes, contracts, theorem mapping, checker commands and logs are in `verification-inventory.json`. Final sequential rebuild logs are `verification/logs/check-*.log`. The printed selected-root axiom sets contain only `propext`, `Classical.choice`, and `Quot.sound`. No sorry, admit, project axiom, native decision axiom, edited generated model, or replacement Rust implementation is used by these roots.

Trusted boundaries are the pinned Rust/Charon extraction, Aeneas translation and standard models, and Lean's kernel and checked libraries. The active selected-API model contains no project axioms. An earlier full-crate extraction, including an opaque I/O error type, is archived as text and is not imported. Generic BufRead I/O, Hasher laws, derived traits, no_std and alternative targets remain unverified. This is selected-API verification, not whole-crate completion.

An earlier failed conditional resume lemma is preserved as text under `verification/attempts`; it is not imported or claimed. The stronger unconditional resume equivalence is checked in `APIs.lean`.
