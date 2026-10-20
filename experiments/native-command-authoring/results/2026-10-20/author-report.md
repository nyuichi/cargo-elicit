# Final verification report

**All five selected APIs and all requested optional laws are proved. This is selected-API verification, not whole-crate completion.** Original `src/` and `Cargo.toml` bytes are preserved; `verification/original.sha256` records and checks their initial hashes. No counterexample or unfinished claim remains within the requested computation-core scope.

All theorem names below are in namespace `Verification`.

| API | Theorem roots | Guarantee |
|---|---|---|
| `Adler32::new` | `new_correct` | Successful return of `(1, 0)`. |
| `Adler32::checksum` | `checksum_correct` | Successful return with natural value `65536*b+a`, for every pair of `u16` halves. |
| `Adler32::from_checksum` | `from_checksum_correct` | Successful decoding of every `u32`: `a=c%65536`, `b=c/65536`. |
| `Adler32::write_slice` | `write_slice_correct`, `write_slice_returns` | Successful execution and exact scalar recurrence for every representable state and slice. |
| `adler32_slice` | `adler32_slice_correct`, `adler32_slice_returns` | Successful return encoding the scalar recurrence from `(1,0)`. |

`Algebra.scalar` first normalizes both seed halves modulo 65521, then folds `a'=(a+byte)%65521`, `b'=(b+a')%65521`. Consequently the update theorem includes normalization on empty input. There is no normalized-seed hypothesis and no artificial input-length bound. `⦃…⦄` denotes Aeneas total correctness; `Laws.spec_returns` derives explicit successful-return equations from it.

Optional laws are proved by `codec_state` / `codec_word`, their execution forms `codec_state_execution` / `codec_word_execution`, `split_update`, `save_restore_resume`, and `split_save_restore_resume`. Both split laws explicitly require `xs.length+ys.length ≤ Usize.max`; `concatSlice` constructs the representable concatenation. Seeds are unrestricted representable states.

## Correspondence and build inventory

Fresh, unedited `Generated/Adler2.lean` is extracted from the original manifest and source. Extraction starts at `adler32_slice` and `Adler32::from_checksum`; the other selected APIs and optimized helpers are reachable. `Core.compute_correct` unfolds the extracted compute body. Proofs cover all six loops, checked arithmetic, casts, array/slice indexing, chunk iterators, and helper calls. The duplicate inner and summation loops are connected by definitional-equality theorems. `Mix.mix` and `Finish.finish` factor the extracted suffix through definitional matching in `Core`; they do not replace the Rust implementation.

A valid module/build order is:

`Generated.Adler2`; `Vector`, `Algebra`, `Codec`; `Lanes`; `Bounds`; `Inner`; `Chunks`; `Outer`, `Next`; `Iterators`; `Mix`; `Finish`; `Core`; `Api`; `Laws`; `Proofs`; `Audit`.

Supporting mathematics uses `ZMod 65521` for lane recombination and scalar equivalence, plus natural-number bounds for machine safety. In particular, the lane accumulator bound at 5,552 groups is 4,294,690,200, below `u32::MAX`.

## Reproduction and actual validation

From the crate root:

```sh
source /tmp/elicit-implementation-env
bash verification/validate.sh
# The same native command through the progress utility:
cargo elicit verify --fresh
```

Final validation **passed**. The script checks original-byte hashes, freshly runs Charon with `--preset=aeneas --sysroot=default`, the original `Cargo.toml`, and target `x86_64-unknown-linux-gnu`; translates with Aeneas; compares the fresh Lean model byte-for-byte; cleans only this Lean package with `lake clean adlerVerification`; rebuilds proofs; and audits transitive axioms for 14 theorem roots. Only `propext`, `Classical.choice`, and `Quot.sound` occur. There are no authored admissions or project axioms. An early optional roundtrip attempt used native `bv_decide` axioms; it was replaced with arithmetic proofs and is absent from the final sources.

All five final cargo-elicit goals share `bash validate.sh`, directory `verification`. An earlier three-codec-goal checkpoint used `bash validate-codec.sh`. CLI success records command execution and freshness; mathematical coverage comes from the theorem statements, correspondence proof, and axiom audit above.

Installed versions: Charon `0.1.293` (`de8736f7c995ac54dc490345cef9ea5143c06c62`), Rust extraction toolchain `nightly-2026-09-17`, Aeneas `e62dd83`, Lean `4.35.0`. Lake uses the prepared installed backend and libraries through local paths.

## Boundaries and utility assessment

Trust includes the Rust/Charon/Aeneas translation chain, standard Aeneas machine/slice/iterator models, installed Lean libraries, and Lean's kernel and permitted foundational axioms. Translation/compiler correctness and physical resource exhaustion are not independently proved. Generic `BufRead` I/O, Hasher laws, derived traits, `no_std`, and alternative targets remain **unverified**.

Cargo-elicit and its skill provided useful command receipts, focused dependency views, and change detection: adding `Laws.lean` correctly made earlier receipts stale. The dependency view helped locate `compute` and the vector loader, but did not expose the complete proof structure through trait operations. External model inputs required explicit configuration, and mathematical correspondence and axiom auditing required separate native tooling. Ordinary Lean compilation alone initially accepted the disallowed native roundtrip axioms; the explicit audit found them. The utility tracked the work accurately without serving as a proof oracle or orchestrator.
