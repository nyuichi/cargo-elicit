import Iterators
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096
namespace Verification
open adler2

/-- A continuation presentation of the recombination sequence in the generated body.
The API proof unfolds this definition against that body. -/
def mix {T : Type} (a b : U32) (x y : Q) (k : U32 → U32 → Result T) : Result T := do
  let b_vec3 ← algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign y.arr 4#u32
  let i8 ← Std.Array.index_usize x.arr 1#usize
  let i9 ← 65521#u32 - i8
  let i10 ← Std.Array.index_usize b_vec3 1#usize
  let i11 ← i10 + i9
  let a2 ← Std.Array.update b_vec3 1#usize i11
  let i12 ← Std.Array.index_usize x.arr 2#usize
  let i13 ← 65521#u32 - i12
  let i14 ← i13 * 2#u32
  let i15 ← Std.Array.index_usize a2 2#usize
  let i16 ← i15 + i14
  let a3 ← Std.Array.update a2 2#usize i16
  let i17 ← Std.Array.index_usize x.arr 3#usize
  let i18 ← 65521#u32 - i17
  let i19 ← i18 * 3#u32
  let i20 ← Std.Array.index_usize a3 3#usize
  let i21 ← i20 + i19
  let a4 ← Std.Array.update a3 3#usize i21
  let s := Std.Array.to_slice x.arr
  let iter1 ← core.slice.Slice.iter s
  let a5 ← algo.Adler32.compute_loop2 iter1 a
  let s1 := Std.Array.to_slice a4
  let iter2 ← core.slice.Slice.iter s1
  let b4 ← algo.Adler32.compute_loop3 iter2 b
  k a5 b4

theorem mix_correct {T : Type} (a b : U32) (x y : Q) (k : U32 → U32 → Result T)
    (P : T → Prop) (ha : a.val ≤ 65535) (hb : b.val ≤ 65520)
    (hx : x.bound 65520) (hy : y.bound 65520)
    (hk : ∀ a' b' : U32, a'.val ≤ 327615 → b'.val ≤ 1506966 →
      ((a'.val:F),(b'.val:F)) = combineQ a.val b.val x y → k a' b' ⦃ P ⦄) :
    mix a b x y k ⦃ P ⦄ := by
  rcases hx with ⟨hx0,hx1,hx2,hx3⟩
  rcases hy with ⟨hy0,hy1,hy2,hy3⟩
  unfold mix Q.arr
  step with vec_mul y.x0 y.x1 y.x2 y.x3 4#u32 as
    ⟨r,c0,c1,c2,c3,hr,h0,h1,h2,h3⟩
  all_goals try scalar_tac
  subst r
  simp only [vec, Array.make, lift, bind_ok, core.slice.Slice.iter]
  simp only [sum_duplicate]
  step* <;> simp_all [remainingSum, Std.Array.to_slice, Std.Array.set, Array.make, Q.arr, vec]
  all_goals try scalar_tac
  all_goals try omega
  all_goals (apply Prod.ext <;> dsimp [combineQ,combine] <;> ring)
end Verification
