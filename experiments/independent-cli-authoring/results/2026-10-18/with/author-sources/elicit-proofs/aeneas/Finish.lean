import FinishLoops
open Aeneas Aeneas.Std Result ElicitGenerated AdlerMath
set_option maxHeartbeats 2000000
namespace Proofs

/-- The exact generated suffix of compute, factored only inside the Lean proof.
The compute proof checks the factorization by definitional equality. -/
def finish (a b : U32) (av bv : Vec) (remainder : Slice U8) : Result Adler32 := do
  let b_vec3 ← algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign bv 4#u32
  let i8 ← Array.index_usize av 1#usize
  let i9 ← 65521#u32 - i8
  let i10 ← Array.index_usize b_vec3 1#usize
  let i11 ← i10 + i9
  let a2 ← Array.update b_vec3 1#usize i11
  let i12 ← Array.index_usize av 2#usize
  let i13 ← 65521#u32 - i12
  let i14 ← i13 * 2#u32
  let i15 ← Array.index_usize a2 2#usize
  let i16 ← i15 + i14
  let a3 ← Array.update a2 2#usize i16
  let i17 ← Array.index_usize av 3#usize
  let i18 ← 65521#u32 - i17
  let i19 ← i18 * 3#u32
  let i20 ← Array.index_usize a3 3#usize
  let i21 ← i20 + i19
  let a4 ← Array.update a3 3#usize i21
  let s ← lift (Array.to_slice av)
  let iter1 ← core.slice.Slice.iter s
  let a5 ← algo.Adler32.compute_loop2 iter1 a
  let s1 ← lift (Array.to_slice a4)
  let iter2 ← core.slice.Slice.iter s1
  let b4 ← algo.Adler32.compute_loop3 iter2 b
  let iter3 ← core.slice.Slice.iter remainder
  let (a6, b5) ← algo.Adler32.compute_loop4 iter3 a5 b4
  let i22 ← a6 % 65521#u32
  let i23 ← lift (UScalar.cast .U16 i22)
  let i24 ← b5 % 65521#u32
  let i25 ← lift (UScalar.cast .U16 i24)
  ok { a := i23, b := i25 }

theorem vec_four (v : Vec) : v.val = [v.val[0]!,v.val[1]!,v.val[2]!,v.val[3]!] := by
  have hl : v.val.length = 4 := by simp
  generalize hx : v.val = xs at hl ⊢
  rcases xs with _ | ⟨x0, _ | ⟨x1, _ | ⟨x2, _ | ⟨x3, _ | ⟨x4, xs⟩⟩⟩⟩⟩ <;> simp_all

theorem vec_sum (v : Vec) : (v.val.map UScalar.val).sum =
    lane v 0 + lane v 1 + lane v 2 + lane v 3 := by
  conv_lhs => rw [vec_four v]
  simp [lane, Nat.add_assoc]

@[step] theorem finish_spec (a b : U32) (av bv : Vec) (rem : Slice U8)
    (ha : a.val ≤ 65535) (hb : b.val < 65521) (hv : reduced av bv)
    (hr : rem.val.length < 4) :
    finish a b av bv rem ⦃ out =>
      out.a.val < 65521 ∧ out.b.val < 65521 ∧
      castState (Specs.state out) = scalar (combine a.val b.val av bv) rem.val ⦄ := by
  have ha0 := (hv 0 (by decide)).1; have ha1 := (hv 1 (by decide)).1
  have ha2 := (hv 2 (by decide)).1; have ha3 := (hv 3 (by decide)).1
  have hb0 := (hv 0 (by decide)).2; have hb1 := (hv 1 (by decide)).2
  have hb2 := (hv 2 (by decide)).2; have hb3 := (hv 3 (by decide)).2
  have hm : ∀ j < 4, lane bv j * (4#u32).val ≤ 4294967295 := by
    intro j hj; have := (hv j hj).2; simp; omega
  unfold finish
  step as ⟨bm, hbm⟩
  have hbm0 := hbm 0 (by decide); have hbm1 := hbm 1 (by decide)
  have hbm2 := hbm 2 (by decide); have hbm3 := hbm 3 (by decide)
  unfold lane at ha0 ha1 ha2 ha3 hb0 hb1 hb2 hb3 hbm0 hbm1 hbm2 hbm3
  simp_lists at *
  step as ⟨v1, hv1⟩
  step as ⟨d1, hd1, hd1b⟩
  step as ⟨w1, hw1⟩
  step as ⟨z1, hz1⟩
  step as ⟨bv1, hbv1⟩
  step as ⟨v2, hv2⟩
  step as ⟨d2, hd2, hd2b⟩
  step as ⟨m2, hm2⟩
  step as ⟨w2, hw2⟩
  step as ⟨z2, hz2⟩ by simp_all [Array.set, U32.max, U32.numBits, UScalarTy.numBits]; omega
  step as ⟨bv2, hbv2⟩
  step as ⟨v3, hv3⟩
  step as ⟨d3, hd3, hd3b⟩
  step as ⟨m3, hm3⟩
  step as ⟨w3, hw3⟩
  step as ⟨z3, hz3⟩ by simp_all [Array.set, U32.max, U32.numBits, UScalarTy.numBits]; omega
  step as ⟨bv3, hbv3⟩
  simp only [lift, bind_ok, core.slice.Slice.iter]
  have hsumA : restSum ⟨Array.to_slice av, 0⟩ =
      lane av 0 + lane av 1 + lane av 2 + lane av 3 := by
    simp [restSum, vec_sum]
  have hsumB : restSum ⟨Array.to_slice bv3, 0⟩ =
      lane bv 0 * 4 + (lane bv 1 * 4 + (65521 - lane av 1)) +
      (lane bv 2 * 4 + (65521 - lane av 2)*2) +
      (lane bv 3 * 4 + (65521 - lane av 3)*3) := by
    simp only [restSum, List.drop_zero, Array.val_to_slice]
    rw [vec_sum]
    simp_all [lane]
  have hsa : a.val + restSum ⟨Array.to_slice av, 0⟩ ≤ 327615 := by
    rw [hsumA]; unfold lane; simp_lists; omega
  have hsb : b.val + restSum ⟨Array.to_slice bv3, 0⟩ ≤ 1506966 := by
    rw [hsumB]; unfold lane; simp_lists; omega
  step as ⟨asum, hasum⟩
  step as ⟨bsum, hbsum⟩
  have hna : asum.val ≤ 327615 := by omega
  have hnb : bsum.val ≤ 1506966 := by omega
  have htail := tail_spec ⟨rem, 0⟩ asum bsum (by simp; omega) (by simp; nlinarith)
  step with htail as ⟨af, bf, hout⟩
  obtain ⟨haf, hbf, hmod⟩ := hout
  step as ⟨ar, har⟩
  step as ⟨br, hbr⟩
  have harn : ar.val < 65521 := by omega
  have hbrn : br.val < 65521 := by omega
  have hca : (UScalar.cast .U16 ar).val = ar.val := by simp [UScalar.cast_val_eq]; omega
  have hcb : (UScalar.cast .U16 br).val = br.val := by simp [UScalar.cast_val_eq]; omega
  refine ⟨by simpa [hca], by simpa [hcb], ?_⟩
  simp only [Specs.state, castState, hca, hcb, har, hbr, ZMod.natCast_mod]
  rw [hmod]
  congr 1
  simp only [hasum, hbsum, hsumA, hsumB]
  apply Prod.ext
  · simp [combine, Nat.cast_add, add_assoc]
  · simp only [combine, Nat.cast_add, Nat.cast_mul, Nat.cast_ofNat, Nat.cast_sub (Nat.le_of_lt (hv 1 (by decide)).1),
      Nat.cast_sub (Nat.le_of_lt (hv 2 (by decide)).1), Nat.cast_sub (Nat.le_of_lt (hv 3 (by decide)).1)]
    have hz : (65521 : F) = 0 := by exact ZMod.natCast_self 65521
    simp only [hz]
    ring

end Proofs
