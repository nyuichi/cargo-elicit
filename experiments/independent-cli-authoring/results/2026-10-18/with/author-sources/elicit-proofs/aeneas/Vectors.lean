import Codec
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Proofs
abbrev Vec := algo.U32X4
def lane (v : Vec) (j : Nat) : Nat := (v.val[j]!).val

theorem lane_bound (v : Vec) (j : Nat) : lane v j ≤ 4294967295 := by
  unfold lane
  scalar_tac

@[step] theorem vec_add_spec (x y : Vec)
    (h : ∀ j < 4, lane x j + lane y j ≤ 4294967295) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign x y
      ⦃ z => ∀ j < 4, lane z j = lane x j + lane y j ⦄ := by
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  unfold lane at *
  simp_lists at *
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step* by simp_all [Array.set, U32.max, U32.numBits, UScalarTy.numBits]
  intro j hj
  have : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases this with rfl | rfl | rfl | rfl <;> simp_all

@[step] theorem vec_rem_spec (x : Vec) (m : U32) (hm : m.val > 0) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign x m
      ⦃ z => ∀ j < 4, lane z j = lane x j % m.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step* by simp_all [Array.set, U32.max, U32.numBits, UScalarTy.numBits]
  intro j hj
  have : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases this with rfl | rfl | rfl | rfl <;> simp_all [lane]

@[step] theorem vec_mul_spec (x : Vec) (m : U32)
    (h : ∀ j < 4, lane x j * m.val ≤ 4294967295) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign x m
      ⦃ z => ∀ j < 4, lane z j = lane x j * m.val ⦄ := by
  have h0 := h 0 (by decide)
  have h1 := h 1 (by decide)
  have h2 := h 2 (by decide)
  have h3 := h 3 (by decide)
  unfold lane at *
  simp_lists at *
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step* by simp_all [Array.set, U32.max, U32.numBits, UScalarTy.numBits]
  intro j hj
  have : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases this with rfl | rfl | rfl | rfl <;> simp_all

@[step] theorem vec_from_spec (s : Slice U8) (h : s.val.length = 4) :
    algo.U32X4.from s ⦃ z => ∀ j < 4, lane z j = (s.val[j]!).val ⦄ := by
  unfold algo.U32X4.from
  simp only [lift, bind_ok]
  step* by simp_all [Array.set, U32.max, U32.numBits, UScalarTy.numBits]
  intro j hj
  have : j = 0 ∨ j = 1 ∨ j = 2 ∨ j = 3 := by omega
  rcases this with rfl | rfl | rfl | rfl <;> simp_all [lane, Array.make]

end Proofs
