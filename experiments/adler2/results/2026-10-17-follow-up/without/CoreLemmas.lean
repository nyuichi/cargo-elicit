import Proofs
import Mathlib.Tactic.IntervalCases
open Aeneas Aeneas.Std Result
set_option maxRecDepth 2048
set_option maxHeartbeats 2000000
namespace Verification

/-- Mathematical specification: normalization is observable even on empty input. -/
def normalize (s : Nat × Nat) : Nat × Nat := (s.1 % 65521, s.2 % 65521)
def scalarStep (s : Nat × Nat) (x : Nat) : Nat × Nat :=
  let a := (s.1 + x) % 65521
  (a, (s.2 + a) % 65521)
def scalarAdler (s : Nat × Nat) (xs : List Nat) : Nat × Nat :=
  xs.foldl scalarStep (normalize s)

theorem normalize_idem (s : Nat × Nat) : normalize (normalize s) = normalize s := by
  simp only [normalize, Nat.mod_mod]

theorem scalarStep_normalized (s : Nat × Nat) (x : Nat) :
    normalize (scalarStep s x) = scalarStep s x := by
  simp only [normalize, scalarStep, Nat.mod_mod]

theorem scalarAdler_normalized (s : Nat × Nat) (xs : List Nat) :
    normalize (scalarAdler s xs) = scalarAdler s xs := by
  have h (ys : List Nat) (t : Nat × Nat) (ht : normalize t = t) :
      normalize (ys.foldl scalarStep t) = ys.foldl scalarStep t := by
    induction ys generalizing t with
    | nil => exact ht
    | cons x ys ih => exact ih _ (scalarStep_normalized t x)
  exact h xs (normalize s) (normalize_idem s)

/-- Pure specification law. Connecting this to optimized compute is a separate obligation. -/
theorem scalarAdler_split (s : Nat × Nat) (xs ys : List Nat) :
    scalarAdler s (xs ++ ys) = scalarAdler (scalarAdler s xs) ys := by
  simp only [scalarAdler, List.foldl_append]
  rw [show normalize (xs.foldl scalarStep (normalize s)) =
    xs.foldl scalarStep (normalize s) from scalarAdler_normalized s xs]

/-- Success and byte widening for the actual four-byte loader. -/
theorem loader_correct (xs : Slice U8) (h : xs.val.length ≥ 4) :
    adler2.algo.U32X4.from xs ⦃ r =>
      ∀ i : Nat, (hi : i < 4) → (r.val[i]'(by scalar_tac)).val = (xs.val[i]'(by omega)).val ⦄ := by
  unfold adler2.algo.U32X4.from
  simp only [core.convert.num.FromU32U8.from, lift]
  step*
  intro j hj
  subst i; subst i2; subst i4; subst i6
  interval_cases j <;> simp only [Array.make_val, List.getElem_cons_zero, List.getElem_cons_succ]
  all_goals
    change (BitVec.setWidth 32 _).toNat = _
    rw [BitVec.toNat_setWidth_of_le (by decide : 8 ≤ 32)]
    rfl

/-- The original lane-wise reduction cannot fail when the divisor is nonzero. -/
theorem vector_rem_correct (v : adler2.algo.U32X4) (q : U32) (hq : q.val > 0) :
    adler2.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign v q ⦃ r => ∀ j : Nat, (hj : j < 4) →
      (r.val[j]'(by scalar_tac)).val = (v.val[j]'(by scalar_tac)).val % q.val ⦄ := by
  unfold adler2.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step*
  intro j hj
  interval_cases j <;> simp_all only [Array.set_val_eq, UScalar.ofNatCore_val_eq, List.getElem_set, Nat.reduceEqDiff, ite_false, ite_true]

theorem vector_add_correct (v w : adler2.algo.U32X4)
    (h : ∀ i : Nat, (hi : i < 4) →
      (v.val[i]'(by scalar_tac)).val + (w.val[i]'(by scalar_tac)).val ≤ U32.max) :
    adler2.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign v w ⦃ r => ∀ j : Nat, (hj : j < 4) →
      (r.val[j]'(by scalar_tac)).val = (v.val[j]'(by scalar_tac)).val + (w.val[j]'(by scalar_tac)).val ⦄ := by
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  unfold adler2.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step*
  all_goals first | (intro j hj; interval_cases j) | skip
  all_goals simp_all only [Array.set_val_eq, UScalar.ofNatCore_val_eq, List.getElem_set, Nat.reduceEqDiff, ite_false, ite_true]

theorem vector_mul_correct (v : adler2.algo.U32X4) (q : U32)
    (h : ∀ i : Nat, (hi : i < 4) →
      (v.val[i]'(by scalar_tac)).val * q.val ≤ U32.max) :
    adler2.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign v q ⦃ r => ∀ j : Nat, (hj : j < 4) →
      (r.val[j]'(by scalar_tac)).val = (v.val[j]'(by scalar_tac)).val * q.val ⦄ := by
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  unfold adler2.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step*
  all_goals first | (intro j hj; interval_cases j) | skip
  all_goals simp_all only [Array.set_val_eq, UScalar.ofNatCore_val_eq, List.getElem_set, Nat.reduceEqDiff, ite_false, ite_true]

/-- Worst case for a lane at a reduction boundary, with up to 5552 new bytes. -/
theorem lane_accumulator_bound (n : Nat) (hn : n ≤ 5552) :
    65520 + n * 65520 + 255 * n * (n+1) / 2 ≤ 4294967295 := by
  have hsq : n*n ≤ 5552*5552 := Nat.mul_self_le_mul_self hn
  have hpoly : 2*(65520+n*65520) + 255*n*(n+1) ≤ 2*4294967295 := by
    nlinarith
  omega

/-- The initial scalar seed is an arbitrary u16, rather than a normalized residue. -/
theorem scalar_chunk_seed_bound (a b n : Nat)
    (ha : a ≤ 65535) (hb : b ≤ 65535) (hn : n ≤ 22208) :
    b + n*a ≤ 4294967295 := by
  have := Nat.mul_le_mul hn ha
  omega

/-- Contract still to be discharged against every loop in the generated model. -/
def WriteSliceCorrect : Prop :=
  ∀ (s : adler2.Adler32) (xs : Slice U8),
    ∃ t, adler2.Adler32.write_slice s xs = ok t ∧
      (t.a.val, t.b.val) = scalarAdler (s.a.val, s.b.val) (xs.val.map UScalar.val)

def ConvenienceCorrect : Prop :=
  ∀ xs : Slice U8, ∃ r, adler2.adler32_slice xs = ok r ∧
    r.val = let t := scalarAdler (1,0) (xs.val.map UScalar.val); 65536*t.2+t.1

/-- The concatenation must fit in one Rust slice in the backend's standard model. -/
def SplitUpdateCorrect : Prop :=
  ∀ (s : adler2.Adler32) (xs ys : Slice U8)
    (hcat : (xs.val ++ ys.val).length ≤ Usize.max),
    (do let t ← adler2.Adler32.write_slice s xs
        adler2.Adler32.write_slice t ys) =
      adler2.Adler32.write_slice s (.from (xs.val ++ ys.val) hcat)

/-- This composition result is conditional on the full, currently open compute contract. -/
theorem convenience_of_write (hw : WriteSliceCorrect) : ConvenienceCorrect := by
  intro xs
  rcases hw ⟨1#u16, 0#u16⟩ xs with ⟨t, ht, hv⟩
  refine ⟨encode t, ?_, ?_⟩
  · unfold adler2.adler32_slice
    rw [new_correct]
    simp only [bind_ok]
    rw [ht]
    simp only [bind_ok, checksum_eq]
  · rw [encode_val]
    have ha := congrArg Prod.fst hv
    have hb := congrArg Prod.snd hv
    change t.a.val = (scalarAdler (1,0) (xs.val.map UScalar.val)).1 at ha
    change t.b.val = (scalarAdler (1,0) (xs.val.map UScalar.val)).2 at hb
    simp only [ha, hb]

/-- Conditional source split law with concatenated representability explicit. -/
theorem split_of_write (hw : WriteSliceCorrect) : SplitUpdateCorrect := by
  intro s xs ys hcat
  rcases hw s xs with ⟨t, ht, htv⟩
  rcases hw t ys with ⟨u, hu, huv⟩
  rcases hw s (.from (xs.val ++ ys.val) hcat) with ⟨v, hv, hvv⟩
  rw [ht]
  simp only [bind_ok]
  rw [hu, hv]
  apply congrArg Result.ok
  have hs : (u.a.val, u.b.val) = (v.a.val, v.b.val) := by
    rw [huv, hvv, htv]
    simp only [Slice.from_val, List.map_append, scalarAdler_split]
  have ha : u.a = v.a := UScalar.eq_of_val_eq (congrArg Prod.fst hs)
  have hb : u.b = v.b := UScalar.eq_of_val_eq (congrArg Prod.snd hs)
  cases u
  cases v
  simp only at ha hb
  cases ha
  cases hb
  rfl

#print axioms scalarAdler_split
#print axioms loader_correct
#print axioms vector_rem_correct
#print axioms vector_add_correct
#print axioms vector_mul_correct
#print axioms lane_accumulator_bound
#print axioms scalar_chunk_seed_bound
#print axioms convenience_of_write
#print axioms split_of_write
end Verification
