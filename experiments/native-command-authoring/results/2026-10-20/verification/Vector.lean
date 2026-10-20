import Generated.Adler2
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 1000000
namespace Verification
open adler2
abbrev vec (x0 x1 x2 x3 : U32) : algo.U32X4 := Array.make 4#usize [x0,x1,x2,x3]

@[step] theorem vec_add (a0 a1 a2 a3 b0 b1 b2 b3 : U32)
    (h0 : a0.val + b0.val ≤ U32.max) (h1 : a1.val + b1.val ≤ U32.max)
    (h2 : a2.val + b2.val ≤ U32.max) (h3 : a3.val + b3.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
      (vec a0 a1 a2 a3) (vec b0 b1 b2 b3) ⦃ r =>
      ∃ c0 c1 c2 c3, r = vec c0 c1 c2 c3 ∧
      c0.val = a0.val + b0.val ∧ c1.val = a1.val + b1.val ∧
      c2.val = a2.val + b2.val ∧ c3.val = a3.val + b3.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign
  step* <;> simp_all [vec, Array.make, Std.Array.set]
  refine ⟨i2, i5, i8, i11, ?_, i2_post, i5_post, i8_post, i11_post⟩
  subst_vars
  apply Std.Array.ext
  simp [vec, Array.make, Std.Array.set]

@[step] theorem vec_rem (a0 a1 a2 a3 : U32) (q : U32) (hq : q.val > 0) :
    algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign (vec a0 a1 a2 a3) q ⦃ r =>
      ∃ c0 c1 c2 c3, r = vec c0 c1 c2 c3 ∧
      c0.val = a0.val % q.val ∧ c1.val = a1.val % q.val ∧
      c2.val = a2.val % q.val ∧ c3.val = a3.val % q.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign
  step* <;> simp_all [vec, Array.make, Std.Array.set]
  refine ⟨i1, i3, i5, i7, ?_, i1_post, i3_post, i5_post, i7_post⟩
  subst_vars
  apply Std.Array.ext
  simp

@[step] theorem vec_mul (a0 a1 a2 a3 q : U32)
    (h0 : a0.val * q.val ≤ U32.max) (h1 : a1.val * q.val ≤ U32.max)
    (h2 : a2.val * q.val ≤ U32.max) (h3 : a3.val * q.val ≤ U32.max) :
    algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign (vec a0 a1 a2 a3) q ⦃ r =>
      ∃ c0 c1 c2 c3, r = vec c0 c1 c2 c3 ∧
      c0.val = a0.val * q.val ∧ c1.val = a1.val * q.val ∧
      c2.val = a2.val * q.val ∧ c3.val = a3.val * q.val ⦄ := by
  unfold algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign
  step* <;> simp_all [vec, Array.make, Std.Array.set]
  refine ⟨i1, i3, i5, i7, ?_, i1_post, i3_post, i5_post, i7_post⟩
  subst_vars
  apply Std.Array.ext
  simp

@[step] theorem vec_from (s : Slice U8) (hs : s.val.length = 4) :
    algo.U32X4.from s ⦃ r => ∃ c0 c1 c2 c3, r = vec c0 c1 c2 c3 ∧
      c0.val = s.val[0].val ∧ c1.val = s.val[1].val ∧
      c2.val = s.val[2].val ∧ c3.val = s.val[3].val ⦄ := by
  unfold algo.U32X4.from
  simp only [lift, bind_ok]
  step* <;> simp_all [Slice.length]
end Verification
