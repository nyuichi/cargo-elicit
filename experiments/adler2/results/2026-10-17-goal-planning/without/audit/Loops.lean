import Proofs
import Spec
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.Ring
set_option linter.unusedSimpArgs false
set_option linter.unnecessarySimpa false
set_option linter.unusedTactic false
set_option linter.unnecessarySeqFocus false
set_option maxHeartbeats 2000000
open Aeneas Aeneas.Std Result ControlFlow
namespace Verification
open adler2

def lane (v : algo.U32X4) (j : Fin 4) : Nat :=
  (v.val[j.val]'(by simpa using j.isLt)).val

theorem vector_add_lane (v w : algo.U32X4)
    (hb : ∀ j : Fin 4, lane v j + lane w j ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign v w
      ⦃ r => ∀ j, lane r j = lane v j + lane w j ⦄ := by
  apply WP.spec_mono (vector_add v w (by intro j hj; exact hb ⟨j, hj⟩))
  intro r hr j
  have hjv : j.val < v.val.length := by simpa using j.isLt
  have hjw : j.val < w.val.length := by simpa using j.isLt
  have hjr : j.val < r.val.length := by simpa using j.isLt
  have h := congrArg (fun l => l[j.val]!) hr
  simpa [lane, vals, List.getElem!_eq_getElem?_getD,
    List.getElem?_eq_getElem, List.getElem?_zipWith, hjv, hjw, hjr] using h

theorem vector_from_lane (bytes : Slice U8) (hb : bytes.val.length = 4) :
    algo.U32X4.from bytes
      ⦃ r => ∀ j : Fin 4, lane r j = bytes.val[j.val]!.val ⦄ := by
  apply WP.spec_mono (vector_from bytes hb)
  intro r hr j
  have hjb : j.val < bytes.val.length := by simpa [hb] using j.isLt
  have hjr : j.val < r.val.length := by simpa using j.isLt
  have h := congrArg (fun l => l[j.val]!) hr
  simpa [lane, vals, List.getElem!_eq_getElem?_getD,
    List.getElem?_eq_getElem, hjb, hjr] using h

theorem vector_rem_lane (v : algo.U32X4) (m : U32) (hm : m.val ≠ 0) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign v m
      ⦃ r => ∀ j, lane r j = lane v j % m.val ⦄ := by
  apply WP.spec_mono (vector_rem v m hm)
  intro r hr j
  have hjv : j.val < v.val.length := by simpa using j.isLt
  have hjr : j.val < r.val.length := by simpa using j.isLt
  have h := congrArg (fun l => l[j.val]!) hr
  simpa [lane, vals, List.getElem!_eq_getElem?_getD,
    List.getElem?_eq_getElem, hjv, hjr] using h

theorem vector_mul_lane (v : algo.U32X4) (m : U32)
    (hb : ∀ j, lane v j * m.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign v m
      ⦃ r => ∀ j, lane r j = lane v j * m.val ⦄ := by
  apply WP.spec_mono (vector_mul v m (by intro j hj; exact hb ⟨j, hj⟩))
  intro r hr j
  have hjv : j.val < v.val.length := by simpa using j.isLt
  have hjr : j.val < r.val.length := by simpa using j.isLt
  have h := congrArg (fun l => l[j.val]!) hr
  simpa [lane, vals, List.getElem!_eq_getElem?_getD,
    List.getElem?_eq_getElem, hjv, hjr] using h

attribute [step] vector_rem_lane vector_mul_lane

attribute [step] vector_add_lane vector_from_lane

def laneStep (j : Fin 4) (s : Nat × Nat) (xs : Slice U8) : Nat × Nat :=
  let a := s.1 + xs.val[j.val]!.val
  (a, s.2 + a)

def laneFold (j : Fin 4) (s : Nat × Nat) (chunks : List (Slice U8)) : Nat × Nat :=
  chunks.foldl (laneStep j) s

def laneBounds (k : Nat) (a b : algo.U32X4) : Prop :=
  ∀ j, lane a j ≤ 65520 + k * 255 ∧
    2 * lane b j ≤ 2*65520 + 2*k*65520 + 255*k*(k+1)

theorem lane_bound_step (k x a b : Nat) (hk : k < 5552) (hx : x ≤ 255)
    (ha : a ≤ 65520 + k * 255)
    (hb : 2*b ≤ 2*65520 + 2*k*65520 + 255*k*(k+1)) :
    a+x ≤ U32.max ∧ b+(a+x) ≤ U32.max ∧
    a+x ≤ 65520 + (k+1)*255 ∧
    2*(b+(a+x)) ≤ 2*65520 + 2*(k+1)*65520 + 255*(k+1)*(k+2) := by
  have hkk : k*k ≤ 5551*5551 := Nat.mul_self_le_mul_self (by omega)
  have ub : b+(a+x) ≤ 4294690200 := by nlinarith
  have ua : a+x ≤ 1481280 := by omega
  refine ⟨?_, ?_, ?_, ?_⟩
  · simp only [U32.max, U32.numBits, UScalarTy.numBits]; omega
  · simp only [U32.max, U32.numBits, UScalarTy.numBits]; omega
  · omega
  · nlinarith

theorem vector_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : algo.U32X4) (k : Nat)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (hk : k + cs.length ≤ 5552) (hb : laneBounds k a b) :
    algo.Adler32.compute_loop1 ⟨cs, rem⟩ a b
      ⦃ a' b' => laneBounds (k + cs.length) a' b' ∧
        ∀ j, (lane a' j, lane b' j) =
          laneFold j (lane a j, lane b j) cs ⦄ := by
  induction cs generalizing a b k with
  | nil =>
    unfold algo.Adler32.compute_loop1
    unfold loop
    simp [algo.Adler32.compute_loop1.body, core.slice.iter.IteratorChunksExact.next,
      laneFold, hb]
  | cons c cs ih =>
    have hc : c.val.length = 4 := hlen c (by simp)
    have hcs : ∀ c ∈ cs, c.val.length = 4 := by simp_all
    have hkk : k < 5552 := by simp at hk; omega
    unfold algo.Adler32.compute_loop1
    unfold loop
    simp only [algo.Adler32.compute_loop1.body,
      core.slice.iter.IteratorChunksExact.next, bind_tc_ok]
    step
    step
    · intro j
      have hh := hb j
      have hx : c.val[j.val]!.val ≤ 255 := by scalar_tac
      have := lane_bound_step k (c.val[j.val]!.val) (lane a j) (lane b j)
        hkk hx hh.1 hh.2
      simp_all
    step
    · intro j
      have hh := hb j
      have hx : c.val[j.val]!.val ≤ 255 := by scalar_tac
      have := lane_bound_step k (c.val[j.val]!.val) (lane a j) (lane b j)
        hkk hx hh.1 hh.2
      simp_all
    change algo.Adler32.compute_loop1 ⟨cs, rem⟩ _ _ ⦃ _ ⦄
    apply WP.spec_mono (ih _ _ (k+1) hcs (by simp at hk; omega) ?_)
    · intro r hr
      constructor
      · simpa [List.length_cons, Nat.add_assoc, Nat.add_comm, Nat.add_left_comm] using hr.1
      · intro j
        simpa [laneFold, laneStep, *] using hr.2 j
    · intro j
      have hh := hb j
      have hx : c.val[j.val]!.val ≤ 255 := by scalar_tac
      have := lane_bound_step k (c.val[j.val]!.val) (lane a j) (lane b j)
        hkk hx hh.1 hh.2
      simp_all [laneBounds]

theorem vector_inner_loop (cs : List (Slice U8)) (rem : Slice U8)
    (a b : algo.U32X4) (k : Nat)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (hk : k + cs.length ≤ 5552) (hb : laneBounds k a b) :
    algo.Adler32.compute_loop0_loop0 ⟨cs, rem⟩ a b
      ⦃ a' b' => laneBounds (k + cs.length) a' b' ∧
        ∀ j, (lane a' j, lane b' j) =
          laneFold j (lane a j, lane b j) cs ⦄ := by
  have eqBody : algo.Adler32.compute_loop0_loop0.body =
      algo.Adler32.compute_loop1.body := by rfl
  simpa only [algo.Adler32.compute_loop0_loop0, algo.Adler32.compute_loop1,
    eqBody] using vector_loop cs rem a b k hlen hk hb

#print axioms vector_loop
#print axioms vector_inner_loop
#print axioms lane_bound_step

def tailSum (it : core.slice.iter.Iter U32) : Nat :=
  ((it.slice.val.drop it.i).map UScalar.val).sum

theorem sum_loop (it : core.slice.iter.Iter U32) (a : U32)
    (hb : a.val + tailSum it ≤ U32.max) :
    algo.Adler32.compute_loop2 it a ⦃ r => r.val = a.val + tailSum it ⦄ := by
  unfold algo.Adler32.compute_loop2
  unfold loop
  unfold algo.Adler32.compute_loop2.body
  dsimp only
  by_cases hi : it.i < it.slice.val.length
  · have hn : core.slice.iter.IteratorSliceIter.next it =
        ok (some it.slice.val[it.i], { it with i := it.i + 1 }) := by
      simp [core.slice.iter.IteratorSliceIter.next, hi] <;> rfl
    have hs : tailSum it = it.slice.val[it.i].val +
        tailSum { it with i := it.i + 1 } := by
      unfold tailSum
      rw [List.drop_eq_getElem_cons hi]
      simp only [List.map_cons, List.sum_cons]
    rw [hn]
    step as ⟨a', ha'⟩
    change algo.Adler32.compute_loop2 { it with i := it.i + 1 } a' ⦃ _ ⦄
    apply WP.spec_mono (sum_loop _ a' (by omega))
    intro r hr
    omega
  · have hn : core.slice.iter.IteratorSliceIter.next it = ok (none, it) := by
      simp [core.slice.iter.IteratorSliceIter.next, hi] <;> rfl
    rw [hn]
    simp [bind_tc_ok, tailSum, List.drop_eq_nil_of_le (by omega : it.slice.val.length ≤ it.i)]
termination_by it.slice.val.length - it.i
 decreasing_by simp_all; omega

theorem sum_loop_b (it : core.slice.iter.Iter U32) (b : U32)
    (hb : b.val + tailSum it ≤ U32.max) :
    algo.Adler32.compute_loop3 it b ⦃ r => r.val = b.val + tailSum it ⦄ := by
  have eqBody : algo.Adler32.compute_loop3.body =
      algo.Adler32.compute_loop2.body := by rfl
  simpa only [algo.Adler32.compute_loop3, algo.Adler32.compute_loop2, eqBody]
    using sum_loop it b hb

#print axioms sum_loop
#print axioms sum_loop_b
