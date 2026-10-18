# Aeneas / Lean 4

The adapter checks the original Rust package through pinned Charon → Aeneas → Lean. Translation alone is not proof evidence. Humans and external AI tools author the proof sources; cargo-elicit never invokes an AI.

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

## Proof project

```sh
cargo elicit init --tool aeneas --api identity
cargo elicit show identity
```

Create `elicit-proofs/aeneas/contracts.toml` in the selected package:

```toml
schema_version = 1

[contracts."my-crate::identity::Function"]
requires = "fun x => True" # optional
ensures = "fun x out => out = x"
theorem = "Proofs.identity_spec"
```

The project configuration schema is 2; this backend-specific contract file independently uses schema_version 1. Use the exact target ID from the inventory. `show` reports the generated proposition name, such as `ElicitContracts.c_HASH`.

Optional `Specs.lean` defines models and may import local model modules. `Proofs.lean` is the proof entry point and may import local proof modules. For the original `pub fn identity(x: u32) -> u32 { x }`, a proof has this form:

```lean
import ElicitContracts
namespace Proofs
theorem identity_spec : ElicitContracts.c_HASH := by
  intro x _
  exact ⟨x, rfl, rfl⟩
end Proofs
```

Replace `c_HASH` with the proposition returned by `show`. Inspect generated source and phase logs in the receipt when the actual translated type is more complex. Contracts quantify the entire original translated signature, including generics/dictionaries. They require successful execution; specialized-input theorems cannot replace full API coverage.

`ElicitAdapter.spec_to_contract` and `contract_to_spec` bridge native Aeneas total-WP proofs and generated contracts. `elicit_spec rule_name from theorem_name` derives a caller-facing WP theorem registered for Aeneas `step*`. These convenience definitions are themselves checked and their transitive assumptions audited.

```sh
cargo elicit check identity
cargo elicit audit identity
cargo elicit status
```

`check` reuses matching checker-owned Lean products; `audit` performs a fresh rebuild. Only `propext`, `Classical.choice`, and `Quot.sound` are allowed axioms. `sorry`, custom axioms, `native_decide`, wrong theorem types and unrecognized audit output remain unresolved.

Declarations in `elicit.toml` may also specify a mathematical `statement` and `theorem` instead of an API `target`. Put the theorem in the imported proof graph. Its evidence covers that proposition, not an independently inferred Rust API contract.

Mapped methods, structures in function signatures, slices, mutable references and internal helper calls are supported when the fresh extraction and actual Result-valued signature are supported. Type-invariant and trait-law automation remains limited; trait implementation targets do not automatically become proved by checking an explicit method. Standard-library MIR unavailable to Charon, ambiguous mappings, external axioms and unsupported constructs remain explicit limitations. See [architecture](implementation.md) for provenance, scope and cache guarantees.
