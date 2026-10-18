# Aeneas with Lean 4

Aeneas is an optional third verifier. The adapter extracts the selected **original Rust package** with Charon, translates it with Aeneas, and asks Lean to check a theorem against a generated contract. Translation alone never establishes `verified` status.

## Toolchain

Install and build the following pinned sources using their upstream instructions:

- [Aeneas](https://github.com/AeneasVerif/aeneas/tree/e62dd83d133d9043e7d3628096838b7430f36565): `e62dd83d133d9043e7d3628096838b7430f36565`.
- Charon: `de8736f7c995ac54dc490345cef9ea5143c06c62` (0.1.293), including its pinned Rust nightly and submodules.
- Lean: `leanprover/lean4:v4.35.0`, as selected by the Aeneas backend's `lean-toolchain`.

Put `aeneas`, `charon`, `charon-driver`, and `lake` on PATH. Set `AENEAS_HOME` to the Aeneas checkout containing `backends/lean`. In that backend directory, fetch its pinned dependencies and build the Lean library before running cargo-elicit:

```sh
cd "$AENEAS_HOME/backends/lean"
lake exe cache get
lake build
```

cargo-elicit checks the tool versions and fingerprints installed backend sources, models, dependencies, compiled libraries, the Charon Rust toolchain, and Lean installation. On Unix, an acceleration cache stores installed-file digests guarded by inode, size, modification time, and change time; project sources are always read directly. A cold scan can be expensive. It does not install tools or download/build the Aeneas library during verification. Rust extraction uses Charon's distributed sysroot mode (`--sysroot=default`); functions needing unavailable standard-library MIR remain unsupported. Other Aeneas proof backends are not supported yet.

## Project files

Aeneas now defaults to [autonomous goal-set authoring and staged artifact collection](autonomous-aeneas.md). Use `run --workflow planned` to select the previous step dispatcher.


```sh
cargo elicit init --tool aeneas
cargo elicit agent show 'my-crate::identity::Function'
cargo elicit run
```

Keep proof inputs under `elicit-proofs/aeneas/` in the **selected package**, including when using a workspace. `contracts.toml` is separate from the three settings in `elicit.toml`:

```toml
schema_version = 1

[contracts."my-crate::identity::Function"]
# Arguments have their original order; the final ensures argument is the result.
requires = "fun x => True" # optional, defaults to True
ensures = "fun x result => result = x"
theorem = "Proofs.identity_spec" # needed for proof completion
```

For `pub fn identity(x: u32) -> u32 { x }`, the generated proposition universally quantifies `x : U32`. The proposition is constructed inside Lean from the actual translated function type, preserving every argument and any generic parameters or dictionaries. It requires the generated original function to return `.ok result` and the postcondition to hold whenever the precondition holds. Thus the proof also establishes successful termination in Aeneas's result semantics. The adapter constructs this proposition; a theorem about one chosen input or a different function cannot substitute for it.

`agent show ID` reports `aeneas_contract`, the stable generated proposition name. Optional `Specs.lean` imports `ElicitGenerated` and defines logical models. `Proofs.lean` imports `ElicitContracts` and proves the generated proposition:

```lean
import ElicitContracts

namespace Proofs

-- Replace c_HASH with the name returned by agent show.
theorem identity_spec : ElicitContracts.c_HASH := by
  intro x _
  exact ⟨x, rfl, rfl⟩

end Proofs
```

A complete example is in [fixtures/aeneas](../fixtures/aeneas). The wrapper retains fresh generated `ElicitGenerated.lean`, `ElicitContracts.lean`, and audit files under `.elicit/cache/`. Inspect these when developing proofs, but never edit them as persistent proof inputs. Each invocation generates and compiles fresh modules; deleting this cache preserves the ledger and project proof sources.

With `verification.required = false`, the wrapper type-checks translation, models, and contracts; `Proofs.lean` is not required or compiled. With `true`, it also compiles proofs, checks each theorem against the full generated proposition, and inspects its transitive axioms. Only Lean's standard `propext`, `Classical.choice`, and `Quot.sound` are allowed. `sorry`, project axioms, compiler-trusting decision procedures, missing theorems, type mismatches, or unrecognized audit output leave verification unresolved.

## Initial coverage and changes

Full-contract completion supports mapped free functions and methods whose Aeneas translation produces a `Result`-valued function, including structures, slices, mutable references, machine-size integers, generic parameters, and internal helper calls. The fresh Charon body must be local and transparent, with the exact original source contents and definition span; the generated Lean definition must map unambiguously to that same span. Contracts quantify the entire translated function signature, rather than a selected Rust primitive signature or one generic instantiation. Charon monomorphization, skipped borrow checking, and disabled type checking are rejected. The Lean proof and its transitive axiom audit cover helper dependencies; opaque/project-axiomatized helpers cannot establish an unconditional full contract. A translated output may contain updated state or borrow continuations rather than the Rust return value alone: inspect `ElicitGenerated.lean` and state the postcondition on that actual output. Rust runtime bodies are not copied into a separate proof crate. Unsupported Aeneas constructs, ambiguous source mappings, non-`Result` return shapes, and missing proofs remain unresolved. General type invariants and trait-law coverage retain the limitations described in [implementation.md](implementation.md).

Changing a contract's `requires`/`ensures` or `Specs.lean` returns affected specifications and their dependents to review. Changing a theorem name or `Proofs.lean` invalidates proof freshness without changing the specification hash. All project proof files participate in the broad proof-input fingerprint. The trusted base includes Charon, the Aeneas translation and Rust models, and the Lean kernel and standard axioms; it does not imply verified translation correctness or specification adequacy.

The native `aeneas_methods_structs_slices_and_helpers` test uses [fixtures/aeneas-rich](../fixtures/aeneas-rich) to check a constructor, a struct field reader, a mutable state update, arbitrary slice length, and a free function calling a local method. Whole-crate completion still requires all ledger targets and complete semantic analysis. Proving selected methods does not independently establish trait laws or type invariant preservation, and built-in derives or unprocessed APIs may continue to appear as analysis limitations.

## Blocked work and proof refresh

Trait implementation targets remain in the ledger but are explicitly unsupported by the current adapter. The scheduler does not repeatedly attempt their submission; use explicit method obligations. An archived or retained unsupported trait contract does not prevent supported method processing. Type invariants can be written directly as Lean predicates/theorems; structs have no generated function-contract constant.

`agent issue` records a blocked target, not a request to immediately revisit it. Relevant source/toolchain, target-contract or declared-dependency changes can unblock it; `run --retry` explicitly clears worker blockers and attempt limits. Built-in unsupported mappings remain blocked until adapter support changes. Blocked obligations prevent whole-package completion and appear in status/list output.

Existing proof artifacts are checked before asking an AI worker to repair them. A successful check records all independently audited, ready contracts from that invocation, including contracts outside the scheduler's requested target. Proof edits still invalidate evidence until a fresh kernel check passes. Verification failure records expose a bounded phase and fresh-cache location, also supplied in the next worker task; raw compiler/model output is not persisted.

For simple crates, the operational files `PROGRESS.md` and `verification-inventory.json` do not affect proof freshness. This optimization falls back to hashing those files when custom build/compiler hooks, active dependencies, Rust file-input macros/path attributes, or Lean file-reading/metaprogramming could make them semantic inputs. Other files remain conservatively tracked. Adding another proof can require a fresh shared-module check, but that refresh does not itself require another AI turn.

`.elicit/events.jsonl` records task/worker/checker boundaries, proof results and input transitions with changed-file hashes in the ledger. It excludes conversation text and raw tool output. It allows benchmark timelines to distinguish proof work, blocked work and checker-only refreshes without reading model session databases.

Prepared, declared proof artifacts are checked before unrelated specification tasks. A checker checkpoint records all independently successful ready roots; it does not start an AI proof-authoring turn. Interrupted checks retain an interrupted record and accept no new proof, even when final input inspection cannot run.

For static Rust crates and static Lean model definitions, specification dependencies are collected from Lean's elaborated contract propositions and the transitively referenced `Specs` constants. Bound-variable names and source metadata are removed before hashing, while types, values, universe parameters, declaration kinds and the freshly translated Rust model and imported toolchain remain inputs. Adding an independent model or lemma keeps existing specification reviews ready. Changing a referenced definition or contract reopens the affected specifications. Type reviews bind the model definitions present when they were accepted; extending that set does not rewrite prior definitions, while changing or removing a bound definition requires review.

This metadata never establishes a proof. Every changed proof/model input still requires fresh kernel checking before current verification is recorded. Model I/O, metaprograms, custom imports/build hooks and unsupported declaration kinds retain conservative whole-model invalidation. Missing, malformed or interrupted dependency collection also falls back conservatively. Inductive/constructor/recursor declarations are not approximated using only their visible type.
