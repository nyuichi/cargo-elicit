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
residues <=65520 for the next inner loop. The scheduler reports success for these two lemmas in verification-djF2SH. Full write_slice and adler32_slice remain unproved; no full
selected-API or whole-crate completion is claimed.

## Outer iteration checkpoint

The original compute_loop0.body now has a locally checked complete transition
theorem outer_step for its actual 22208-byte chunk size. It checks the actual
inner call, the usize-to-u32 cast, multiplication, scalar addition, both original
vector remainder calls, and scalar remainder. The theorem reconstructs the full
input chunk from its four-byte partition with no ignored bytes and returns exact
modular lane folds and scalar update, together with bounds needed for the next
iteration. Arbitrary original u16 state halves remain allowed (scalar seeds may
be as large as 65535). Internal vector residues are bounded by the reductions.
The exact expected plan proposition and its allowed axioms are checked locally;
this new checkpoint awaits the scheduler audit.

Remaining caller proof: compose outer_step across all full chunks, using modular
congruence and lane-fold append laws; connect the final chunk; check original
vector multiplication and final summation loops; and prove recombination and
scalar-tail agreement with AdlerSpec.run. The selected update contract is intact.

## Complete outer loop checkpoint

The scheduler reports success for outer_step in verification-XKpiWt. The new
outer_loop theorem locally checks the entire original outer loop for arbitrarily
many full chunks, using structural induction for termination. It reconstructs
all processed bytes as exact quartets, preserves the scalar modulo invariant,
and returns exact normalized lane folds and bounds. laneRun_congr proves seed
congruence preservation and laneRun_append composes consecutive partitions.

outer_iter_wp matches the actual original caller's chunk size and iterator. A
new write_slice proof attempt uses it to pass the full outer loop and stops at
the final-chunk inner call, with the processed quartet partition, exact lane
folds, original scalar seed bounds and normalized vector bounds available.
There is no overall input-length restriction. These new checkpoint roots await
the official scheduler audit. The full selected update contract remains unproved.

## Final multiplication and summation checkpoint

The scheduler reports success for the complete outer loop and its mathematical
congruence/append laws in verification-m616Eq. New locally checked original-body
proofs cover U32X4 multiplication and both complete summation loops. The latter
prove exact sums of the remaining iterator values and terminate as the slice
index advances. Their only arithmetic precondition is that the initial value
plus the mathematical remaining sum fits u32. The second generated sum loop is
definitionally equal to the first; this equality is checked. A separate vector
sum bound supplies the caller's path from four individual lane bounds to these
total-sum preconditions.

The new final_inner_iter_wp rule connects the already checked final-chunk loop
to its actual iterator argument. The original write_slice attempt now invokes
both sum-loop contracts and reaches compute_loop4, the final scalar byte loop.
Numerical obligations remain open for lane correction arithmetic and sum totals;
the scalar-tail loop and functional recombination also remain unproved. No full
update contract is claimed. These new checkpoint roots await official checking.

Scalar-tail checkpoint: the complete original compute_loop4 now has a locally
checked execution theorem with termination, indexing and every addition covered.
A derived three-byte budget and a mathematical normalization theorem connect
its unreduced fold to AdlerSpec.run. The full caller attempt in
.elicit/cache/scalar-tail/Caller.log now reaches final normalization; numeric
side conditions and lane/scalar recombination remain unresolved. New lemma
roots await the official scheduler audit. Previously saved multiplication and
summation roots have official success in verification-ia5Scu. Runtime files
and the full arbitrary-input update contract are preserved.

Quartet recombination checkpoint: local Lean checks prove the exact unreduced
four-lane/scalar identity and the complete MOD-minus-lane correction formula
used by the original implementation. An exact four-lane iterator-sum theorem
is also saved. All transitive axiom checks contain only propext, Classical.choice
and Quot.sound. The updated original caller proof in
.elicit/cache/recombine/Caller.lean discharges all numeric side conditions and
reaches a single final functional equality; it is still an unfinished scratch
proof. Connect.lean instantiates the recombination theorem for the caller's
combined quartet partition. Remaining work is to connect those partition and
congruence facts to the final scalar-tail result. New checkpoint roots await
official audit. The preceding scalar-tail roots passed in verification-pxAuYz.

Integration bridge checkpoint: run_congr, run_append and
combine_lane_residues pass local Lean checks and the exact expected plan
propositions, with only permitted axioms. These connect normalized lane seeds
through the final inner loop and compose the scalar tail with a prefix update.
The original caller attempt in .elicit/cache/integrate/Fast.lean proves byte
reconstruction, empty quartet remainder, lane congruences, scalar base count,
first-accumulator recombination, and tail composition. Its remaining reported
error is the second accumulator's modular expression rearrangement. A revised
attempt is in Next.lean. Full API success is not claimed at this checkpoint.
The preceding quartet recombination roots passed official checking in
verification-Q9ucd7. Runtime files and all arbitrary-input contracts are preserved.

Full update and root integration checkpoint: the complete Proofs.lean module
passes the local Lean kernel check against verification-euuFVk. The mapped
write_slice_correct theorem proves successful execution and the exact scalar
recurrence for every representable initial state and slice, including empty
normalization. adler32_slice_correct instantiates the original caller
composition without assumptions. Their printed transitive axioms are limited
to propext, Classical.choice and Quot.sound. Both full contracts now await
the official scheduler audit. Remaining selected API: from_checksum. Separate
codec/split/resume laws remain to be attempted. Runtime files are preserved.
