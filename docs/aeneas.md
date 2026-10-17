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

For `pub fn identity(x: u32) -> u32 { x }`, the generated proposition universally quantifies `x : U32`. It requires the generated original function to return `.ok result` and the postcondition to hold whenever the precondition holds. Thus the proof also establishes successful termination in Aeneas's result semantics. The adapter constructs this proposition; a theorem about one chosen input or a different function cannot substitute for it.

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

Full-contract completion currently covers independent, non-generic free functions with bool, fixed-width signed/unsigned integers, and unit. The fresh translation must map back to the original Rust name and source location. Methods, generics, references, `char`, `usize`/`isize`, helper calls, external dependencies, custom Charon metadata, and other unconfirmed mappings remain unresolved. Rust runtime bodies are not copied into a separate proof crate. General type invariants and trait-law coverage retain the limitations described in [implementation.md](implementation.md).

Changing a contract's `requires`/`ensures` or `Specs.lean` returns affected specifications and their dependents to review. Changing a theorem name or `Proofs.lean` invalidates proof freshness without changing the specification hash. All project proof files participate in the broad proof-input fingerprint. The trusted base includes Charon, the Aeneas translation and Rust models, and the Lean kernel and standard axioms; it does not imply verified translation correctness or specification adequacy.
