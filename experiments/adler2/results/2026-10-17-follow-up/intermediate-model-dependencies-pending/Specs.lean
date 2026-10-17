import ElicitGenerated

namespace AdlerSpec

/-- Representation bounds; states restored from a checksum need not be reduced modulo 65521. -/
def Representable (a b : Nat) : Prop := a < 65536 ∧ b < 65536

/-- Stronger post-update condition, not a type invariant. -/
def Normalized (a b : Nat) : Prop := a < 65521 ∧ b < 65521

def Encode (a b : Nat) : Nat := 65536 * b + a

theorem normalized_representable (a b : Nat) (h : Normalized a b) :
    Representable a b := by
  unfold Normalized at h
  unfold Representable
  omega

theorem encoded_bound (a b : Nat) (h : Representable a b) :
    Encode a b < 4294967296 := by
  unfold Representable at h
  unfold Encode
  omega

/-- Invariant on the actual translated original Rust state. -/
def StateInvariant (s : ElicitGenerated.Adler32) : Prop :=
  Representable s.a.val s.b.val

theorem state_invariant (s : ElicitGenerated.Adler32) : StateInvariant s := by
  constructor
  · exact s.a.bv.isLt
  · exact s.b.bv.isLt

/-- Every possible output state preserves the representation invariant. -/
theorem representation_preserved (before after : ElicitGenerated.Adler32)
    (_ : StateInvariant before) : StateInvariant after := state_invariant after

end AdlerSpec

namespace AdlerSpec

/-- Four independent machine lanes; no common-value or modular-normalization assumption. -/
def VectorInvariant (v : ElicitGenerated.algo.U32X4) : Prop :=
  v.val.length = 4 ∧ ∀ lane ∈ v.val, lane.val < 4294967296

theorem vector_invariant (v : ElicitGenerated.algo.U32X4) : VectorInvariant v := by
  constructor
  · exact Aeneas.Std.Array.property v
  · intro lane _
    exact lane.bv.isLt

/-- Representation preservation alone does not establish arithmetic success. -/
theorem vector_representation_preserved
    (before after : ElicitGenerated.algo.U32X4) (_ : VectorInvariant before) :
    VectorInvariant after := vector_invariant after

/-- Explicit safety obligations for the checked lane arithmetic. -/
def LaneAddSafe (x y : Aeneas.Std.U32) : Prop := x.val + y.val < 4294967296
def LaneMulSafe (x y : Aeneas.Std.U32) : Prop := x.val * y.val < 4294967296
def LaneRemSafe (divisor : Aeneas.Std.U32) : Prop := divisor.val ≠ 0

end AdlerSpec

namespace AdlerSpec

/-- Normalize even an empty update from an arbitrary representable seed. -/
def Normalize (s : Nat × Nat) : Nat × Nat := (s.1 % 65521, s.2 % 65521)

/-- Mathematical scalar Adler step; this is a specification, not replacement runtime code. -/
def ScalarStep (s : Nat × Nat) (byte : Aeneas.Std.U8) : Nat × Nat :=
  let a := (s.1 + byte.val) % 65521
  (a, (s.2 + a) % 65521)

def ScalarAdler (s : Nat × Nat) (bytes : List Aeneas.Std.U8) : Nat × Nat :=
  bytes.foldl ScalarStep (Normalize s)

theorem normalize_normalized (s : Nat × Nat) :
    Normalized (Normalize s).1 (Normalize s).2 := by
  constructor <;> exact Nat.mod_lt _ (by decide)

theorem normalize_eq (s : Nat × Nat) (h : Normalized s.1 s.2) : Normalize s = s := by
  rcases s with ⟨a, b⟩
  exact Prod.ext (Nat.mod_eq_of_lt h.1) (Nat.mod_eq_of_lt h.2)

theorem scalar_step_normalized (s : Nat × Nat) (byte : Aeneas.Std.U8) :
    Normalized (ScalarStep s byte).1 (ScalarStep s byte).2 := by
  constructor <;> exact Nat.mod_lt _ (by decide)

theorem scalar_fold_normalized (bytes : List Aeneas.Std.U8) (s : Nat × Nat)
    (h : Normalized s.1 s.2) :
    Normalized (bytes.foldl ScalarStep s).1 (bytes.foldl ScalarStep s).2 := by
  induction bytes generalizing s with
  | nil => exact h
  | cons byte tail ih => exact ih _ (scalar_step_normalized s byte)

theorem scalar_adler_normalized (s : Nat × Nat) (bytes : List Aeneas.Std.U8) :
    Normalized (ScalarAdler s bytes).1 (ScalarAdler s bytes).2 :=
  scalar_fold_normalized bytes _ (normalize_normalized s)

theorem scalar_adler_empty (s : Nat × Nat) : ScalarAdler s [] = Normalize s := rfl

/-- Logical split law. The original implementation still requires its own correctness proof. -/
theorem scalar_adler_append (s : Nat × Nat) (xs ys : List Aeneas.Std.U8) :
    ScalarAdler s (xs ++ ys) = ScalarAdler (ScalarAdler s xs) ys := by
  unfold ScalarAdler
  rw [List.foldl_append]
  rw [normalize_eq _ (scalar_fold_normalized xs (Normalize s) (normalize_normalized s))]

/-- Slice-level model law explicitly requires representability of the concatenated input. -/
theorem scalar_slice_split (s : Nat × Nat) (xs ys : Aeneas.Std.Slice Aeneas.Std.U8)
    (_hrepresentable : xs.val.length + ys.val.length ≤ Aeneas.Std.Usize.max) :
    ScalarAdler s (xs.val ++ ys.val) = ScalarAdler (ScalarAdler s xs.val) ys.val :=
  scalar_adler_append s xs.val ys.val

end AdlerSpec
