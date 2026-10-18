# Final report

The five selected APIs have unconditional total-correctness proofs for their
entire translated signatures. “Unconditional” means no extra precondition beyond
representable Rust arguments, relative to the pinned extraction, translation and
standard models. This is selected-API verification, not whole-crate verification.
The fresh audit accepted all five contracts with `success` and
`full_contract` scope, with stable inputs. Its independently inspectable receipt
is `.elicit/evidence/audit-1792331422261.json`. A final audit after documentation
updates is available as the latest audit receipt under `.elicit/evidence/` and
in the saved snapshot `.elicit/snapshot.json`.

## Theorem roots

All authored Lean sources and contracts are in `elicit-proofs/aeneas/`.

| Rust API | Exact theorem | Exact generated contract |
| --- | --- | --- |
| `Adler32::new` | `Proofs.new_spec` | `ElicitContracts.c_8055bec449b87b3d` |
| `Adler32::checksum` | `Proofs.checksum_spec` | `ElicitContracts.c_0a059b8c0885cf07` |
| `Adler32::from_checksum` | `Proofs.from_checksum_spec` | `ElicitContracts.c_9a31b7b9cb8492b0` |
| `Adler32::write_slice` | `Proofs.write_slice_spec` | `ElicitContracts.c_98f3436459f22f5d` |
| `adler32_slice` | `Proofs.adler32_slice_spec` | `ElicitContracts.c_8484fdd92cfcd3e7` |

The first three roots are in `Codec.lean`; the last two are in `Api.lean`.
`Proofs.compute_spec` in `Compute.lean` establishes total correctness of the
original optimized compute body. No requested API is weakened, disproved or left
unfinished. All five are accepted by the fresh exact-type and axiom audit.

A valid dependency order is:

1. Checker-generated `ElicitGenerated`, `ElicitAdapter`.
2. `Specs`, then checker-generated `ElicitContracts`.
3. `Codec`, `Math`, `Vectors` (Vectors imports Codec).
4. `Inner`, `Chunks`, `Outer`, `FinishLoops`, `Finish`, `Compute`, `Api`.
5. `CodecLaws`, `SpecReview`, `Laws`, `Proofs`.

Generated files are freshly produced by the tool and have not been edited.
The final evidence directory preserves the generated sources and phase logs.
`Proofs.lean` imports the complete proof graph and prints the axioms of the roots.

## Specification and proof coverage

`Specs.step` is the scalar recurrence `a'=(a+byte) mod 65521`,
`b'=(b+a') mod 65521`. `Specs.update` normalizes the input pair and folds this
step. It therefore normalizes empty updates. `Proofs.update_eq_normalize_fold`
also proves that this equals folding from the original pair and normalizing the
result; `Proofs.update_cons` supplies the scalar cons equation. `Specs.encode`
is the natural number `65536*b+a`, not a bitwise specification that assumes the
conclusion. The codec contracts quantify over every u32 or every pair of u16s.

The compute proof covers the actual generated helper calls, indexing, checked
machine operations, slice splitting, both four-byte inner loops, the outer
22208-byte chunk loop, both lane-sum loops, scalar tail, remainders and casts.
`Proofs.inner_spec` uses an increasing local lane counter bounded by the actual
5552 iterations per chunk. `Proofs.outer_spec` recurses on the number of remaining
chunks, without bounding total input length. Seeds may be as large as 65535.
The modular proof uses `ZMod 65521`; normalization and bounded representatives
then give equality of natural-number states.

`Proofs.finish` in `Finish.lean` is a Lean-only factoring of the exact generated suffix. Its
machine-valued implementation is proved in `Proofs.finish_spec`, unfolded and
simplified before applying it to the original compute suffix. It is not a copied
Rust crate, scalar replacement, edited generated model, or assumed helper.

## Additional laws

All these exact roots are kernel checked and included in the final imported
proof graph:

- `Proofs.decode_encode`: checksum then restore returns every representable state.
- `Proofs.encode_decode`: restore then checksum returns every u32.
- `Proofs.scalar_update_append`: the mathematical update composition law.
- `Proofs.split_update`: two original API updates equal one combined update.
- `Proofs.save_restore_resume`: saving/restoring any state before updating preserves execution.
- `Proofs.split_save_restore_resume`: saving between pieces equals one combined update.

Codec and single-resume laws have no extra preconditions. Both split API laws
explicitly require `(xs.val ++ ys.val).length ≤ Usize.max`, so their combined
slice exists. They impose no normalized-state assumption. There are no known
counterexamples and no disproof claim.

## Actual validation and reproduction

From the repository root, run `bash scripts/validate.sh`. It sets paths to the
installed tools, checks original hashes, runs the upstream tests, runs
`cargo elicit audit`, reads status, and checks hashes again.

Commands used during authoring included:

```sh
cargo elicit init --tool aeneas --target x86_64-unknown-linux-gnu \
  --api Adler32::new --api Adler32::checksum --api Adler32::from_checksum \
  --api Adler32::write_slice --api adler32_slice
cargo elicit list --inventory
cargo elicit check
cargo elicit status
cargo test --locked --target x86_64-unknown-linux-gnu
sha256sum -c elicit-proofs/runtime.sha256
```

Native iterations used `lake env lean --root=<proof-directory> -o <temporary-object>
<module.lean>` from the installed Aeneas Lean backend, with LEAN_PATH containing
temporary authored products, the checker-generated workspace, and authored
sources. Author objects were never supplied to cargo-elicit as evidence.
The accepted full check receipt is `.elicit/evidence/check-1792331223299.json`.
Its five results have `success` and `full_contract` scope. Final auditing uses a
fresh independent workspace and recompiles the graph, including the optional
laws. Six upstream unit tests and four doctests passed; those tests are not
substitutes for the universal proofs.

Extraction uses the original manifest, default/std features and requested
x86_64-unknown-linux-gnu target. The preserved LLBC options record preset Aeneas
and sysroot `default`. Versions reported by the checker are Charon 0.1.293
(de8736f7c995ac54dc490345cef9ea5143c06c62), Aeneas e62dd83, and Lean 4.35.0.

## Trust, exclusions and assessment

The trust boundary comprises Rust compilation/extraction by pinned Charon,
Aeneas translation, its standard Rust models, the Lean kernel, and the
cargo-elicit correspondence and exact-type audit. Transitive axiom inspection
allows only `propext`, `Classical.choice` and `Quot.sound`; no project axioms,
proof holes or native-decision axioms are used. The generated generic I/O error
type has an external axiom, but it is absent from the selected roots' transitive
axiom sets. Mathematical specifications still require independent semantic review.
No semantic-review receipt is claimed.

Generic BufRead I/O, Hasher laws, derived traits, no_std, and alternative targets
remain unverified. Passing upstream I/O tests does not change that status.
Discovery reports derive-attribute inventory limitations; these do not establish
or conceal proofs for excluded APIs. Original runtime bytes are unchanged;
`elicit-proofs/runtime.sha256` contains the initial hashes.

The workflow was productive: inventory IDs and generated contracts made the
five mappings concrete; `check` accepted the three codec roots while recurrence
work remained open, then accepted all five. Generated diagnostic sources and phase logs exposed exact
signatures and axiom sets. The skill allowed free native authoring and imposed no
per-lemma protocol. Overhead included initially configuring PATH for Cargo and
Charon, one early check invalidated by a concurrent progress-file edit, and
waiting for full-graph checks when only local Lean feedback was needed. Native
friction included dependent iterator simplification, tuple binder naming and an
expensive failed attempt to match a long suffix automatically; applying WP
monotonicity to its explicitly checked suffix theorem resolved that problem.
No wall-time or token-count estimate is claimed.
