import Aeneas

namespace AdlerSpec

/-- Representation invariant for the natural values of Adler32.a and Adler32.b.
    All u16 pairs are valid; checksum decoding must not require normalization. -/
def Valid (a b : Nat) : Prop := a < 65536 ∧ b < 65536

/-- Stronger postcondition of a successful update, including an empty update. -/
def Normalized (a b : Nat) : Prop := a < 65521 ∧ b < 65521

/-- Mathematical encoding, with no modular reduction of either field. -/
def encode (a b : Nat) : Nat := 65536 * b + a

/-- Scalar model; normalization at entry also specifies the empty update. -/
def update (a b : Nat) (bytes : List Nat) : Nat × Nat :=
  bytes.foldl (fun s byte =>
    let a' := (s.1 + byte) % 65521
    (a', (s.2 + a') % 65521)) (a % 65521, b % 65521)

/-- Split-update laws must explicitly require representability of concatenation. -/
def ConcatRepresentable (xs ys : List Nat) : Prop :=
  xs.length + ys.length < 2 ^ 64

/-- A logical implication only: this does not prove any Rust method preserves it. -/
theorem normalized_valid {a b : Nat} (h : Normalized a b) : Valid a b := by
  constructor
  · exact Nat.lt_trans h.1 (by decide)
  · exact Nat.lt_trans h.2 (by decide)

/-- Natural lane values of the original four-element u32 array. -/
abbrev Lanes := Fin 4 → Nat

/-- Representation invariant: exactly four lanes, each representable as u32.
    Reduction modulo 65521 is a phase property, not a permanent invariant. -/
def LanesValid (v : Lanes) : Prop := ∀ i, v i < 2 ^ 32

/-- Required to prove successful original add_assign execution. -/
def AddSafe (v w : Lanes) : Prop := ∀ i, v i + w i < 2 ^ 32

/-- Required to prove successful original mul_assign execution. -/
def MulSafe (v : Lanes) (rhs : Nat) : Prop := ∀ i, v i * rhs < 2 ^ 32

/-- Required for rem_assign; callers must establish this before division. -/
def RemSafe (quotient : Nat) : Prop := 0 < quotient

/-- Logical preservation obligation for elementwise addition; not a Rust proof. -/
theorem add_lanes_valid (v w : Lanes) (h : AddSafe v w) :
    LanesValid (fun i => v i + w i) := h

/-- Logical preservation obligation for elementwise multiplication. -/
theorem mul_lanes_valid (v : Lanes) (rhs : Nat) (h : MulSafe v rhs) :
    LanesValid (fun i => v i * rhs) := h

/-- Reduction preserves representation when the divisor fits in u32. -/
theorem rem_lanes_valid (v : Lanes) (q : Nat)
    (hq : RemSafe q) (hbound : q < 2 ^ 32) :
    LanesValid (fun i => v i % q) := by
  intro i
  exact Nat.lt_trans (Nat.mod_lt (v i) hq) hbound

end AdlerSpec
