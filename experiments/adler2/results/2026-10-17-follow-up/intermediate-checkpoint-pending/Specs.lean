import Aeneas

namespace AdlerSpec

/-- The public codec admits every pair of unsigned 16-bit halves. -/
def Representable (a b : Nat) : Prop := a < 65536 ∧ b < 65536

/-- This is an update postcondition, not a type invariant. -/
def Normalized (a b : Nat) : Prop := a < 65521 ∧ b < 65521

/-- Mathematical interpretation of the stored halves. -/
def Checksum (a b : Nat) : Nat := 65536 * b + a

/-- Scalar specification, including normalization for an empty update. -/
def Update (a b : Nat) (bytes : List Nat) : Nat × Nat :=
  bytes.foldl (fun s byte =>
    let a' := (s.1 + byte) % 65521
    (a', (s.2 + a') % 65521)) (a % 65521, b % 65521)

/-- Obligations to instantiate with each original constructor or mutator.
    This definition alone does not establish preservation by any Rust API. -/
def PreservesRepresentable (f : Nat → Nat → Nat × Nat) : Prop :=
  ∀ a b, Representable a b → Representable (f a b).1 (f a b).2

end AdlerSpec

namespace AdlerSpec

/-- Logical view of exactly four lanes in the original fixed-size array. -/
abbrev Lanes := Fin 4 → Nat

/-- No byte or modulus bound is a permanent invariant of U32X4. -/
def LanesRepresentable (v : Lanes) : Prop := ∀ i, v i < 4294967296

/-- Successful checked addition requires every lane sum to fit u32. -/
def AddSafe (v w : Lanes) : Prop := ∀ i, v i + w i < 4294967296

/-- Successful checked multiplication requires every lane product to fit u32. -/
def MulSafe (v : Lanes) (k : Nat) : Prop := ∀ i, v i * k < 4294967296

/-- Preservation obligation for the original methods, still to be instantiated. -/
def PreservesLanes (f : Lanes → Lanes) : Prop :=
  ∀ v, LanesRepresentable v → LanesRepresentable (f v)

end AdlerSpec
