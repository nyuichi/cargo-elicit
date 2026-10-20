import Specs
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Vector
abbrev V := algo.U32X4
def lane (v : V) (i : Fin 4) : Nat := (v.val[i.val]'(by have := v.property; simpa using i.isLt)).val
theorem lane_get (v : V) (i : Fin 4) : lane v i = (v.val[i.val]'(by have := v.property; simpa using i.isLt)).val := rfl
@[step] theorem add_spec (v w : V)
    (h : ∀ i, lane v i + lane w i ≤ 4294967295) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign v w
      ⦃ out => ∀ i, lane out i = lane v i + lane w i ⦄ := by
  have h0 := h 0; have h1 := h 1; have h2 := h 2; have h3 := h 3
  simp only [lane, Fin.val_zero, Fin.val_one, Fin.val_ofNat] at h0 h1 h2 h3
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step* <;> simp_all [Array.set_val_eq]
  all_goals first
    | scalar_tac
    | (intro i; fin_cases i <;> simp_all [lane, Array.set_val_eq])
@[step] theorem rem_spec (v : V) (m : U32) (hm : m.val > 0) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign v m
      ⦃ out => ∀ i, lane out i = lane v i % m.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step* <;> simp_all [Array.set_val_eq]
  intro i
  fin_cases i <;> simp_all [lane]
@[step] theorem mul_spec (v : V) (m : U32)
    (h : ∀ i, lane v i * m.val ≤ 4294967295) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign v m
      ⦃ out => ∀ i, lane out i = lane v i * m.val ⦄ := by
  have h0 := h 0; have h1 := h 1; have h2 := h 2; have h3 := h 3
  simp only [lane, Fin.val_zero, Fin.val_one] at h0 h1 h2 h3
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step* <;> simp_all [Array.set_val_eq]
  all_goals first
    | scalar_tac
    | (intro i; fin_cases i <;> simp_all [lane])
def byteLane (s : Slice U8) (i : Fin 4) : Nat := s.val[i.val]!.val
@[step] theorem from_spec (s : Slice U8) (h : 4 ≤ s.val.length) :
    algo.U32X4.from s ⦃ out => ∀ i, lane out i = byteLane s i ⦄ := by
  unfold algo.U32X4.from
  simp only [lift]
  step* <;> simp_all
  intro i
  fin_cases i <;> simp_all [lane, byteLane, core.convert.num.FromU32U8.from_val_eq]
  all_goals rw [List.getElem?_eq_getElem (by omega)]; rfl
end Vector
