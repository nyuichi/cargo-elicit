import Outer
import Next
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096
namespace Verification
open adler2

def remainingSum (s : Slice U32) (i : Nat) := ((s.val.drop i).map UScalar.val).sum

@[step] theorem sum_correct (s : Slice U32) (i : Nat) (a : U32)
    (ha : a.val + remainingSum s i ≤ U32.max) :
    algo.Adler32.compute_loop2 ⟨s,i⟩ a ⦃ r => r.val = a.val + remainingSum s i ⦄ := by
  unfold algo.Adler32.compute_loop2
  rw [loop]
  conv => lhs; arg 1; unfold algo.Adler32.compute_loop2.body
  simp only [Std.uncurry]
  rw [next_eq]
  by_cases hi : i < s.val.length
  ·
    simp only [dif_pos hi, bind_ok, Std.uncurry]
    have he := List.drop_eq_getElem_cons hi
    have hs : remainingSum s i = s.val[i].val + remainingSum s (i+1) := by
      simp only [remainingSum, he, List.map_cons, List.sum_cons]
    step as ⟨a',ha'⟩
    try simp only [bind_ok, Std.uncurry]
    have hab : a'.val + remainingSum s (i+1) ≤ U32.max := by
      rw [ha']
      simpa only [hs, Nat.add_assoc, Slice.getElem_Nat_eq] using ha
    change algo.Adler32.compute_loop2 ⟨s,i+1⟩ a' ⦃ _ ⦄
    apply WP.spec_mono (sum_correct s (i+1) a' hab)
    intro r hr
    rw [hr,ha',hs]
    try simp only [Slice.getElem_Nat_eq]
    omega
  ·
    simp only [dif_neg hi, bind_ok, Std.uncurry]
    try simp only [bind_ok, spec_ok]
    have he : s.val.drop i = [] := List.drop_eq_nil_of_le (by omega)
    simp [remainingSum,he]
termination_by s.val.length-i

 theorem sum_duplicate (it : core.slice.iter.Iter U32) (a : U32) :
    algo.Adler32.compute_loop3 it a = algo.Adler32.compute_loop2 it a := by rfl

@[step] theorem tail_correct (s : Slice U8) (i : Nat) (a b : U32)
    (ha : a.val + 255*(s.val.length-i) ≤ 1000000)
    (hb : b.val + 1000000*(s.val.length-i) ≤ 10000000) :
    algo.Adler32.compute_loop4 ⟨s,i⟩ a b ⦃ r =>
      ((r.1.val:F), (r.2.val:F)) = run ((a.val:F),(b.val:F)) (s.val.drop i) ⦄ := by
  unfold algo.Adler32.compute_loop4
  rw [loop]
  conv => lhs; arg 1; unfold algo.Adler32.compute_loop4.body
  simp only [lift, bind_ok]
  rw [next_eq]
  by_cases hi : i < s.val.length
  ·
    simp only [dif_pos hi, bind_ok, Std.uncurry, lift]
    try simp only [bind_ok]
    have hx : s.val[i].val ≤ 255 := byte_le _
    step as ⟨a',ha'⟩
    have ha'val : a'.val = a.val + s.val[i].val := by
      simpa only [core.convert.num.FromU32U8.from_val_eq, Slice.getElem_Nat_eq] using ha'
    have ha'max : a'.val ≤ 1000000 := by omega
    step as ⟨b',hb'⟩
    try simp only [bind_ok, Std.uncurry]
    change algo.Adler32.compute_loop4 ⟨s,i+1⟩ a' b' ⦃ _ ⦄
    apply WP.spec_mono (tail_correct s (i+1) a' b' (by omega) (by omega))
    intro r hr
    rw [hr, List.drop_eq_getElem_cons hi]
    simp only [run, List.foldl_cons, advance]
    rw [hb',ha'val]
    simp only [Nat.cast_add]
    congr 1
    apply Prod.ext <;> dsimp <;> ring
  ·
    simp only [dif_neg hi, bind_ok, Std.uncurry]
    simp only [bind_ok,spec_ok]
    have he : s.val.drop i = [] := List.drop_eq_nil_of_le (by omega)
    simp [run,he]
termination_by s.val.length-i
end Verification
