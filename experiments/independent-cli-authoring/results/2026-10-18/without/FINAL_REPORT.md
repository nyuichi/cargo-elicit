The selected computation core is proved in Lean 4. All five requested APIs have unconditional successful-execution and functional-correctness results for every input representable in the standard Aeneas model. This is a selected-API result, not whole-crate verification. No requested claim was disproved or left unfinished.

| Rust API | Independently checkable theorem | Result |
| --- | --- | --- |
| `Adler32::new` | `AdlerProof.new_correct` (`Codec.lean`); `AdlerProof.new_spec` (`API.lean`) | Returns `(1,0)`. |
| `Adler32::checksum` | `AdlerProof.checksum_correct`; `AdlerProof.checksum_spec` | Succeeds and returns `65536*b+a` for every pair of `u16` halves. |
| `Adler32::from_checksum` | `AdlerProof.from_checksum_correct`; `AdlerProof.from_checksum_spec` | Succeeds for every `u32`; halves are `c % 65536` and `c / 65536`. |
| `Adler32::write_slice` | `AdlerProof.write_slice_correct` (`API.lean`) | Succeeds and returns exactly `AdlerProof.scalar` from arbitrary representable seed halves and byte slice. |
| `adler32_slice` | `AdlerProof.adler32_slice_correct` (`API.lean`) | Succeeds and encodes that recurrence from `(1,0)`. |

`AdlerProof.scalar` in `proofs/Specification.lean` first normalizes `(a,b)` to `(a % 65521,b % 65521)`, then folds the scalar step `a'=(a+byte)%65521`, `b'=(b+a')%65521`. Consequently the empty-update claim includes normalization; it is also stated separately as `AdlerProof.write_slice_empty`. There is no normalized-seed precondition and no imposed whole-input length bound.

All paths through the original optimized `compute` model are covered by `AdlerProof.compute_correct_ring` in `proofs/Compute.lean`. Supporting roots include `vector_from`, `vector_add`, `vector_rem`, `vector_mul`, `inner_loop`, `final_inner_loop`, `outer_loop`, `sum_loop`, `sum_loop3`, and `tail_loop`, all in namespace `AdlerProof`. These establish bounds checks, arithmetic safety, helper behavior, and termination. The lane invariant bounds `a` by `65520+255*k` and twice `b` by `131040+131040*k+255*k*(k+1)` within each block. The limit `k <= 5552` is derived from the implementation's internal chunking, not assumed of the caller's input. The outer loop handles arbitrarily many representable chunks. `ZMod 65521` is used only for the mathematical invariant; the final theorem converts it back to the explicit natural-number recurrence.

The optional results in `proofs/Laws.lean` are:

- `AdlerProof.codec_state_roundtrip`: encode/decode preserves every representable state, unconditionally.
- `AdlerProof.codec_checksum_roundtrip`: decode/encode preserves every `u32`, unconditionally.
- `AdlerProof.split_update`: sequential updates and one concatenated update both succeed and return equal states.
- `AdlerProof.save_restore_resume`: updating, saving the checksum, restoring, and resuming returns the same state as the concatenated update.

Both split laws explicitly require `left.val.length + right.val.length <= Usize.max`, the concatenated-slice representability condition in the standard model. `concatSlice` constructs the concatenation using exactly that proof. The laws quantify over arbitrary initial halves and include empty pieces. Equality is of the full returned states, not merely their checksums.

The checked module order is:

1. Fresh generated `proofs/generated/Adler2.lean`.
2. `Codec`, `Vector`, `Inner`, `Chunks`, `Mathematics`, `Outer`, `Sums`, `Tail`, `Combine`, `Specification`, `Compute`, `API`, `Laws` (each under `proofs/`).
3. `proofs/Audit.lean` imports the completed development, checks the exported API/law signatures, and prints axiom dependencies for 21 API, law, core, helper, and loop roots.

The reproducible final command is `./proofs/validate.sh` from this crate directory. It regenerates the model, compiles every module in that order, runs the axiom/source audit, runs `cargo test --locked --target x86_64-unknown-linux-gnu`, and verifies runtime hashes. Final validation passed, including 6 upstream unit tests and 4 doctests. Logs are in `proofs/logs/`, including `validation.log`, `axioms.log`, and `cargo-test.log`. The exact extraction command is persisted in `proofs/extract.sh`: it invokes the installed Charon with `cargo --preset=aeneas --sysroot=default --error-on-warnings`, five `--start-from` paths (`adler2::Adler32::new`, `checksum`, `from_checksum`, `write_slice`, and `adler2::adler32_slice`), the original `Cargo.toml`, and `--target x86_64-unknown-linux-gnu`. Aeneas then runs with `-backend lean -no-progress-bar -dest proofs/generated` on that fresh LLBC. The shell script contains the complete absolute command arguments.

Individual modules can be independently checked with `proofs/lean.sh --root=proofs -o proofs/NAME.olean proofs/NAME.lean` after checking their imports. The generated model uses `--root=proofs/generated`. `proofs/lean.sh` obtains the dependency search path from the installed Aeneas Lean backend's `lake env`, and uses its pinned Lean toolchain.

Installed tools used: Charon `0.1.293 (de8736f7c995ac54dc490345cef9ea5143c06c62)`, Aeneas `e62dd83`, Charon Rust toolchain `nightly-2026-09-17`, and Lean `4.35.0` (`eb6cafb7290e8bd5f2937522d8d804f58f02bafc`). No cargo-elicit tooling, contracts, ledger, or prior experiment material was used. No subagents were used.

The trusted boundaries are Lean's kernel and the three permitted logical axioms (`propext`, `Classical.choice`, `Quot.sound`), the installed standard Aeneas models, and the Rust/Charon/Aeneas translation pipeline. The translation pipeline itself is not verified here. The generated selected-API model has no project axioms or unknown external declarations. All 21 audited roots depend only on the permitted axioms. There are no proof holes, project axioms, native-decision axioms, or edited generated models. Successful execution is expressed using Aeneas total `WP.spec`, rather than partial correctness or mere translation success.

`Slice` and scalar types supply their standard representability bounds; physical allocation/layout and the correctness of the standard library models remain trusted. Results concern the extracted x86_64 computation, not other configurations. Generic `BufRead` I/O, Hasher laws, derived traits, `no_std`, and alternative targets remain unverified. Passing upstream tests for any of these is not a formal proof of them.

Every original byte of `src/`, `Cargo.toml`, and `Cargo.toml.orig` is preserved. The initial hashes are in `proofs/runtime.sha256`; validation checks them before extraction and after testing. No runtime function was copied into another Rust crate, and no scalar runtime replacement was introduced.

Native-tool workflow friction came from executable/search-path setup, Charon's accepted inherent-method root syntax, explicit lifting of pure conversions, tuple-output naming in `step`, and normalizing bit-vector/coercion expressions before arithmetic tactics. Failed root-pattern attempts were corrected; final extraction starts afresh so it cannot reuse their artifacts. Small supporting lemmas for casts, array views, chunk decomposition, and modular interpretation made the remaining proofs tractable. My self-assessment is that building those reusable lemmas was productive: it enabled complete coverage of the optimized loops and the optional laws, though interactive elaboration and coercion debugging required several iterations. I do not report an invented duration or token count.
