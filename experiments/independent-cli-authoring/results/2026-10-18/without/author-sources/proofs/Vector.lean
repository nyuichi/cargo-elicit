import Codec
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096

def vec (a b c d : U32) : adler2.algo.U32X4 := Array.make 4#usize [a,b,c,d]

def view (v : adler2.algo.U32X4) : Fin 4 → Nat := fun i => v.val[i.val].val

theorem vec_cases (v : adler2.algo.U32X4) : ∃ a b c d, v = vec a b c d := by
  obtain ⟨a,b,c,d,h⟩ := List.length_eq_four.mp v.property
  refine ⟨a,b,c,d,?_⟩
  apply Std.Array.ext
  simpa [vec] using h

@[simp] theorem view_vec (a b c d : U32) : view (vec a b c d) = ![a.val,b.val,c.val,d.val] := by
  funext i
  fin_cases i <;> simp [view, vec]

@[step] theorem vector_add (x y : adler2.algo.U32X4)
 (h : ∀ i, view x i + view y i ≤ 4294967295) :
 adler2.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign x y
 ⦃ v => view v = fun i => view x i + view y i ⦄ := by
  obtain ⟨a,b,c,d,rfl⟩ := vec_cases x
  obtain ⟨e,f,g,h',rfl⟩ := vec_cases y
  have h0 := h 0; have h1 := h 1; have h2 := h 2; have h3 := h 3
  simp only [view_vec, Matrix.cons_val_zero, Matrix.cons_val_one, Matrix.cons_val_two,
    Matrix.cons_val_three] at h0 h1 h2 h3
  unfold adler2.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign vec
  step*
  all_goals simp_all [Array.set, U32.max, U32.numBits, view, Array.make]
  all_goals funext i; fin_cases i <;> simp_all [view, Array.set]

@[step] theorem vector_rem (x : adler2.algo.U32X4) (q : U32) (hq : q.val > 0) :
 adler2.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign x q
 ⦃ v => view v = fun i => view x i % q.val ⦄ := by
  obtain ⟨a,b,c,d,rfl⟩ := vec_cases x
  unfold adler2.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign vec
  step*
  all_goals simp_all [Array.set, U32.max, U32.numBits, view, Array.make]
  all_goals funext i; fin_cases i <;> simp_all [view, Array.set]

@[step] theorem vector_mul (x : adler2.algo.U32X4) (q : U32)
 (h : ∀ i, view x i * q.val ≤ 4294967295) :
 adler2.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign x q
 ⦃ v => view v = fun i => view x i * q.val ⦄ := by
  obtain ⟨a,b,c,d,rfl⟩ := vec_cases x
  have h0 := h 0; have h1 := h 1; have h2 := h 2; have h3 := h 3
  simp only [view_vec, Matrix.cons_val_zero, Matrix.cons_val_one, Matrix.cons_val_two,
    Matrix.cons_val_three] at h0 h1 h2 h3
  unfold adler2.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign vec
  step*
  all_goals simp_all [Array.set, U32.max, U32.numBits, view, Array.make]
  all_goals funext i; fin_cases i <;> simp_all [view, Array.set]

@[step] theorem from_u8 (x : U8) : lift (core.convert.num.FromU32U8.from x)
 ⦃ y => y.val = x.val ⦄ := by
  simp [lift, core.convert.num.FromU32U8.from_val_eq]

@[step] theorem from_u16 (x : U16) : lift (core.convert.num.FromU32U16.from x)
 ⦃ y => y.val = x.val ⦄ := by
  simp [lift, core.convert.num.FromU32U16.from_val_eq]

@[step] theorem vector_from (s : Slice U8) (h : s.val.length = 4) :
 adler2.algo.U32X4.from s
 ⦃ v => ∀ i : Fin 4, view v i = s.val[i.val].val ⦄ := by
  unfold adler2.algo.U32X4.from
  step*
  all_goals simp_all [view]
  all_goals intro i; fin_cases i <;> simp_all [view]
