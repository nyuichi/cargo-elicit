# Verification progress

All five selected original APIs have full arbitrary-input contracts and local
Lean kernel-checked proofs in elicit-proofs/aeneas/Proofs.lean. The earlier
official audit reported success for new, checksum, write_slice and adler32_slice;
from_checksum is being submitted for the next official audit. This is selected
API coverage, not whole-crate completion.

write_slice covers the original optimized loops, indexing, machine arithmetic,
chunk decomposition, lane reductions/recombination, scalar tail and empty-input
normalization. It imposes no normalized-state or artificial full-slice bound.
from_checksum decodes every u32 into its low and high 16-bit halves.

Both codec roundtrips, split-update equivalence, and checksum save/restore/resume
pass local Lean kernel checks. They call the original methods through their
complete contracts. Split and resume laws explicitly require concatenated slice
representability. These laws are included in Proofs.lean for module checking;
they do not have separate API-ledger coverage units.

Printed transitive axioms contain only propext, Classical.choice and Quot.sound.
The trusted boundary is the pinned Charon/Aeneas extraction and standard Rust
models plus Lean's kernel. No project axioms, sorry or native_decide are used.

src/ and Cargo.toml are unchanged. Generic BufRead I/O, Hasher laws, derived
traits, no_std and alternative targets remain unverified. Exact theorem names,
checker commands and status are in verification-inventory.json. Prior checkpoint
history is preserved in elicit-proofs/aeneas/CHECKPOINTS.md.
