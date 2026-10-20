All five requested API contracts are proved for every translated input, including successful execution. The independent fresh audit passed **5/5, with zero unresolved obligations**. Original `src/` and `Cargo.toml` bytes are unchanged; final SHA-256 checks match `elicit-proofs/runtime.sha256`. No runtime implementation was substituted.

The final receipt is `.elicit/evidence/audit-1792456951678.json`, bound to snapshot `77d21cf036d4412c0a8d79c655b53a01ca9b239ef7cd4621392613429e51c73f`. Final `status --all --json` reports `valid_at_snapshot` for every API, no input/specification changes, and no fresh audit needed. The receipt records stable inputs and `full_contract` success. Author semantic reviews are accepted separately; they are not independent proof evidence.

| Requested API | Exact theorem root in `Proofs.lean` | Result |
| --- | --- | --- |
| `Adler32::new` | `Proofs.new_spec` | Unconditional: state `(1, 0)` |
| `Adler32::checksum` | `Proofs.checksum_spec` | Unconditional: natural value `65536*b+a` |
| `Adler32::from_checksum` | `Proofs.from_checksum_spec` | Unconditional: `a=sum%65536`, `b=sum/65536` |
| `Adler32::write_slice` | `Proofs.write_slice_spec` | Unconditional: normalized scalar recurrence |
| `adler32_slice` | `Proofs.adler32_slice_spec` | Unconditional: packed recurrence from `(1, 0)` |

“Unconditional” means no additional precondition beyond the standard translated argument types. All five contracts use `requires True`. Scope is the original adler2 2.0.1 manifest, default/std features, and `x86_64-unknown-linux-gnu`.

`AdlerSpec.run` first reduces both seed halves modulo 65521, then folds `a=(a+byte)%65521; b=(b+a)%65521`, using the new `a`. This explicitly normalizes empty updates. There is no normalized-seed assumption or artificial input-length limit. Slices retain the standard Aeneas representability bound.

The optimized body is proved by `Compute.compute_correct` in `Compute.lean`; `Compute.compute_spec` in `Proofs.lean` also checks its full generated contract. All six loops are covered by `Inner.inner_spec`, `Inner.remainder_spec`, `Outer.outer_spec`, `Tail.sum_spec`, `Tail.sum_b_spec`, and `Tail.serial_spec`. The four vector helpers are covered by `Vector.from_spec`, `Vector.add_spec`, `Vector.rem_spec`, and `Vector.mul_spec`. Checked overflow budgets, chunk partition lemmas, checked indexing/updates, the final correction arithmetic, and casts connect to the original generated computation. `LaneAlgebra.block` and `LaneAlgebra.inner_combine` establish the four-lane scalar equivalence over `ZMod 65521`; normalized bounds then recover exact natural-state equality.

The separately requested laws are also proved:

- `CodecLaws.encode_decode` and `CodecLaws.decode_encode`: successful codec roundtrips for every checksum/state.
- `Laws.split_update`: successful split-update equivalence.
- `Laws.save_restore_resume`: successful checksum save/restore/resume equivalence.
- `Laws.split_checksum_resume`: the combined split/save/restore/resume law.

Both split laws explicitly require `xs.val.length + ys.val.length ≤ Usize.max`; `Laws.concat` constructs the representable concatenation. The other laws have no extra preconditions. These are Lean theorem results, not additional declared API targets. They were independently rebuilt through `ExtraAudit.lean` in the fresh audit graph. Their transitive axiom output is preserved in `.elicit/evidence/audit-1792456951678/logs/ExtraAudit.log`.

A valid authored module order, under `elicit-proofs/aeneas/`, is:

`Specs → Codec → Vector → Chunks → Inner → Recurrence → Algebra → Outer → Tail → FinishMath → Compute → API → CodecLaws → Laws → ExtraAudit → Proofs`.

Fresh generated `ElicitGenerated` and `ElicitAdapter` precede this order; generated `ElicitContracts` imports `Specs` and must precede `Proofs`. The contracts file and `verification-inventory.json` identify every API target, generated proposition, theorem root and supporting module. Module imports alone are not claims of theorem-use dependencies.

Actual validation commands, after `source elicit-proofs/env.sh`, included:

```sh
cargo elicit check
cargo elicit audit
cargo elicit status --all --json
sha256sum -c elicit-proofs/runtime.sha256
elicit-proofs/runtime-check.sh
```

The full recorded review note is in the saved review metadata. Native iterations used `elicit-proofs/lean.sh -o MODULE.olean MODULE.lean`. `elicit-proofs/validate.sh` reproduces full checking and fresh auditing; `native-build.sh` is an authoring convenience. The frozen adapter invokes Charon with `--preset=aeneas --sysroot=default` and the original manifest/target, followed by Aeneas’s Lean backend. Models were freshly generated and unedited. Tools are pinned Aeneas `e62dd83d133d9043e7d3628096838b7430f36565`, Charon `de8736f7c995ac54dc490345cef9ea5143c06c62`, and Lean 4.35.0.

The fresh root audit and optional-law inspections list only `propext`, `Classical.choice`, and `Quot.sound`. There are no authored axioms, sorry/admit proofs, or native-decision axioms. Six upstream tests and 1,350 additional arbitrary-seed cases passed, including empty inputs, chunk boundaries, split updates and resumption. Finite tests supplement the proofs.

No requested API/law is disproved or unfinished. This is **selected-API verification, not whole-crate completion**. Generic BufRead I/O, Hasher laws, derived traits, no_std, and alternative targets remain unverified. Trusted boundaries include the Lean kernel, the permitted axioms, pinned Rust/Charon/Aeneas translation and correspondence, and standard Aeneas library models. Generated opaque I/O declarations remain outside the selected theorem dependencies. Independent semantic investigation remains external.

I was effective on the core mathematics: separate overflow budgets and modular lane invariants yielded reusable proofs for the optimized loops. Startup had avoidable overhead from Lean binder naming, simplifier behavior, and native build iteration. `init`, `plan`, and focused `show` helped establish scope and exact contract names; concise check summaries made progress clear without repeated status reads. The axiom check usefully rejected an early `bv_decide` step, which I replaced with arithmetic lemmas. A check run during source edits was correctly rejected for changed inputs; subsequent checks/audits ran with semantic inputs unchanged. The CLI served as a verification utility throughout, without orchestrating agents or supplying proofs.
