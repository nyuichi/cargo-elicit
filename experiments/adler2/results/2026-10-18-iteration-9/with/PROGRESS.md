# Verification progress

Partial selected-API result. This is not whole-crate verification. The original
src/ files and Cargo.toml remain byte-for-byte unchanged.

The official scheduler ledger in verification-EFALCe reports successful original
function-contract checks for Adler32::new, Adler32::checksum, U32X4::from and
U32X4::add_assign. It also reports success for the lane overflow budget and the
chunk partition / chunks_exact WP lemmas. Only the first two are selected APIs.

The complete arbitrary-input contracts for write_slice and adler32_slice are
submitted but unproved. write_slice has no normalized-seed or artificial length
precondition; its model normalizes both halves even on empty input. The checked
slice_of_update implication composes the original new, update and checksum bodies
but explicitly assumes the entire write_slice contract. It is not root evidence.

Lemmas.lean proves the exact transition of the original generated
compute_loop0_loop0.body for a nonempty iterator: consume one chunk, widen its
four bytes, return a'=a+byte and b'=b+a' in each lane. Its stated preconditions
require four readable bytes and both sums to fit u32. Those conditions must still
be derived from an invariant in the full update proof. The lemma uses the proved
original vector-method bodies, not trusted helper assumptions.

The inner_step plan declaration was accidentally truncated by a text extraction
script. It is now repaired as a complete single-line Lean proposition. That exact
expected proposition checks against the saved theorem in the recorded official
cache. Its transitive axioms are only propext, Classical.choice and Quot.sound.
The scheduler now reports success for the repaired inner_step entry in verification-OcgYK3.

A new locally checked checkpoint proves both complete original inner loops
(compute_loop0_loop0 and compute_loop1), including termination and every indexing
and arithmetic operation, for at most 5552 four-byte chunks and lane accumulator
residues at most 65520. Prefix overflow safety follows from the mathematical lane
fold bounds and the checked lane_budget lemma. It is no longer an assumption of
the bounded loop theorems. These are bounds on internal lane accumulators, not
restrictions on the arbitrary user-provided Adler32 state. The two generated
inner-loop bodies are definitionally equal; that equality is kernel checked.

A further lemma derives the 5552-iteration bound from the actual chunks_exact
partition, exact four-byte chunk lengths, and a source chunk of at most 22208
bytes. It imposes no bound on the full input slice. Exact expected propositions
for all three new checkpoint roots are checked locally against their constants.
The scheduler now reports success for both bounded inner loops and the chunk-count bound in verification-kxmyom.

Remaining update obligations: connect the lane folds to the outer modular
invariant; prove the original vector remainder/multiplication calls; prove the
final summation loops and scalar tail; and establish final lane recombination
against the scalar Adler recurrence.
Adler32::from_checksum, codec roundtrips, split-update equivalence (with explicit
concatenated-slice representability), and save/restore/resume equivalence have not
yet been proved. No counterexample is claimed.

Generic BufRead I/O, Hasher laws, derived traits, no_std and alternative targets
are unverified and outside the common goal. Trait-implementation laws are not
claimed; vector helpers are explicit original-method obligations.

Persistent sources are under elicit-proofs/aeneas. Scratch attempts and compiler
objects are under .elicit/cache. Extraction used the original manifest, Charon
--preset=aeneas --sysroot=default and x86_64-unknown-linux-gnu; generated models are
unedited. Local checks use the installed backend's lake env lean with --root and
LEAN_PATH directed to this workspace's scratch and recorded verification cache.
The official scheduler wrapper performs fresh extraction, kernel checking and
transitive axiom auditing. No project axioms, sorry or native_decide were added.

The original U32X4 remainder method now has a locally checked total WP contract
for every nonzero divisor, checking all four original indices and updates. Its
compute.MOD specialization proves exact modulo-65521 results and derives lane
residues <=65520 for the next inner loop. These final two lemmas await the official
scheduler audit. Full write_slice and adler32_slice remain unproved; no full
selected-API or whole-crate completion is claimed.
