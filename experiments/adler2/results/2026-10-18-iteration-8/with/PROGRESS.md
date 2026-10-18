# Verification progress

Original src/ and Cargo.toml are unchanged. This is partial selected-API work, not whole-crate completion.

The full arbitrary-input contracts for adler32_slice, Adler32::new and Adler32::write_slice have been accepted for specification processing. The slice contract has no length bound. The update contract accepts every representable seed and normalizes both halves even on an empty update. Specification acceptance is not proof evidence.

Proofs.lean contains a direct full constructor proof and its caller-facing WP rule. The ledger recorded a successful kernel check of the original new/default bodies at an earlier checkpoint. Other API contracts remain incomplete; no constructor result is used as a substitute for the update proof.

The scheduler checked the first eight support lemmas at input c0f440ddd741b448d26ee0faa05b661b69a6fb63eb87569126e5f39ff2321d70, with cache .elicit/cache/verification-oVJrvF. These establish exact chunk partition/sizes, a chunks_exact WP rule, modular deferral, raw prefix-sum bounds, lane u32 bounds and the four-byte weighted identity. The same wrapper reported that the full write_slice theorem was missing.

The new checkpoint in Lemmas.lean additionally proves properties of the original translated machine code:

- vector_from_spec: four indexed byte reads and widening conversions produce the four lane values.
- vector_add_spec: all four indexed reads, checked u32 additions, and array updates implement coordinatewise addition under explicit overflow bounds.
- inner_spec: the original compute_loop0_loop0 terminates and returns exactly the per-lane scalar fold. Induction on remaining iterator chunks checks every iteration and both vector additions. Its precondition bounds the mathematical final lane fold; monotonicity discharges each prefix overflow condition.
- inner_bounded_spec: derives that precondition from at most 5552 lane updates and reduced internal lane accumulators.
- remainder_inner_spec: the original compute_loop1 has the identical translated body and satisfies the same lane-fold rule.
- chunks_count_eq: exact chunk partition facts give the quantitative number of inner iterations.

These are internal helper obligations, not restrictions on total API length or initial API state. They are locally compiled and queued for official checkpoint checking. No trait implementation law is claimed: vector_add_spec proves the explicit original method body.

After returning to the original write_slice/compute body, the scratch attempt applies the inner-loop rules in both outer-loop branches. Outstanding obligations include deriving iteration counts from partition facts, the outer-loop invariant across reductions, scalar seed contributions, vector remainder/multiplication methods, lane recombination, the final summing/tail loops, and final normalization. The complete API proof remains open.

Persistent modules are under elicit-proofs/aeneas. The unedited diagnostic generated model is cached under .elicit/cache/verification-oVJrvF; local experiments reused the previously captured wrapper model under .elicit/cache/inspect. Failed attempts are confined to the cache. Actual local checks (PATH includes the pinned elan binaries) are:

```
LEAN_PATH=$PWD/.elicit/cache/inspect lake -d /workspace/toolchain/aeneas-src/backends/lean env lean -R elicit-proofs/aeneas -o .elicit/cache/inspect/Specs.olean elicit-proofs/aeneas/Specs.lean
LEAN_PATH=$PWD/.elicit/cache/inspect lake -d /workspace/toolchain/aeneas-src/backends/lean env lean -R elicit-proofs/aeneas -o .elicit/cache/inspect/Lemmas.olean elicit-proofs/aeneas/Lemmas.lean
LEAN_PATH=$PWD/.elicit/cache/inspect lake -d /workspace/toolchain/aeneas-src/backends/lean env lean elicit-proofs/aeneas/Proofs.lean
LEAN_PATH=$PWD/.elicit/cache/inspect lake -d /workspace/toolchain/aeneas-src/backends/lean env lean .elicit/cache/inspect/MachineAudit.lean
```

The local axiom audit of the new machine and loop lemmas reports only propext, Classical.choice and Quot.sound. No sorry, admit, project axiom, native_decide or trusted helper specification is used. Trust includes pinned Charon/Aeneas extraction and standard models and the Lean kernel. Official cargo-elicit checking remains the source of ledger proof evidence.

Remaining benchmark work includes full checksum encoding, from_checksum decoding, optimized arbitrary-state update, slice composition, codec roundtrips, split-update equivalence with concatenated-slice representability, and save/restore/resume equivalence. The additional API laws have not yet been attempted. Generic BufRead I/O, Hasher laws, derived traits, no_std and alternative targets are unverified and outside scope.

## Outer execution checkpoint

The scheduler checked the original loader/addition and inner-loop support lemmas at input 169dab419a44cc6c8fe45731a901c1cfab181a9108eb9b9c4a89f93be4d7cd59 (cache .elicit/cache/verification-EG7RDB). The complete write_slice theorem still has no mapped proof.

New locally checked results cover the original vector remainder and multiplication methods, and outer_execution_spec covers every full-chunk outer-loop iteration and its nested inner loop. This establishes termination, successful checked arithmetic and indexing, reduced lane bounds, and the exact scalar seed contribution to b. It intentionally does not claim the functional lane invariant needed for the API recurrence. The API contract is unchanged and still requires the complete scalar result.

The new rules were applied again at the original compute body. The remaining continuation begins with slice iteration and compute_loop2/compute_loop3, followed by compute_loop4 and normalization. Arithmetic/count side conditions remain in the scratch attempt. The crucial functional gap is preservation of per-lane mathematical meaning across outer modular reductions, followed by a theorem connecting lane recombination to the scalar recurrence. The three new theorems compile in Lemmas.lean; .elicit/cache/inspect/OuterAudit.lean reports only the three permitted logical axioms. The command is the same lake env lean invocation as MachineAudit.lean above, with OuterAudit.lean as the input.

## Plan repair and final sums

At input 8bda774d38bcdb52b8cf7e6d755786cf08cb8c5b87a21235c09b97c804ae95db, the scheduler checked outer_execution_spec, vector_rem_spec and vector_mul_spec (cache .elicit/cache/verification-oj8MbD); the full update theorem remained unresolved.

New locally checked lemmas sum_loop_spec and sum_loop3_spec prove exact sums, successful additions and termination for both original final summing loops under explicit mathematical sum bounds. normalize_raw_fold proves modular normalization commutes with an entire raw fold. update_append proves composition for the pure scalar list model; this is only an attempt at the benchmark split law, not an original Rust API law. A future API split statement must explicitly require concatenated slice representability.

The new sum rules were reconnected to the original body in scratch SumReconnect.lean. The remaining continuation starts at compute_loop4, with the exact sums in context, followed by normalization. Arithmetic side conditions and the cross-chunk functional lane invariant are still open. The repaired plan names the exact next tail-loop proposition and theorem; it is not yet proved. No incomplete theorem is stored in persistent modules.

## Checksum API checkpoint

The full checksum contract now has a local checked theorem adler32_checksum_correct in Proofs.lean and a caller-facing WP rule. It covers every representable Adler32 state and proves successful execution and output 65536*b+a using checked shifting, widening conversions and the disjoint-bit OR identity. It has no normalized-state precondition. Official checking is pending. The existing local Proofs.lean command passed after rebuilding scratch ElicitContracts with the new mapping.

The scheduler also checked both summing-loop lemmas and scalar model composition at input 94cbb8c5d9a81fb30a51a83600f9fee8057ee7057a2136cc1c2f7e2846f57a24 (cache .elicit/cache/verification-zabi3T). Tail-loop proof remains unimplemented. from_checksum, codec roundtrips and save/restore/resume have no proof; the split-update attempt proves only the scalar list-model law. Full write_slice and adler32_slice functional correctness remain unresolved. The full contracts were not weakened. No counterexample was established.
