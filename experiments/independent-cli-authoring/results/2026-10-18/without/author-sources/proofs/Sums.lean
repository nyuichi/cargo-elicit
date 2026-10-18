import Outer
open Aeneas Aeneas.Std Result ControlFlow
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096

def sumValues (l : List U32) : Nat := (l.map UScalar.val).sum

theorem sum_loop (s : Slice U32) (i : Nat) (a : U32)
 (hbound : a.val + sumValues (s.val.drop i) ≤ 4294967295) :
 adler2.algo.Adler32.compute_loop2 ⟨s,i⟩ a
 ⦃ z => z.val = a.val + sumValues (s.val.drop i) ⦄ := by
 unfold adler2.algo.Adler32.compute_loop2
 rw [loop]
 simp only [adler2.algo.Adler32.compute_loop2.body,
   core.slice.iter.IteratorSliceIter.next]
 split
 · rename_i h
   have h' : i < s.val.length := h
   have hs : sumValues (s.val.drop i) = s.val[i].val + sumValues (s.val.drop (i+1)) := by
     simp only [sumValues, List.drop_eq_getElem_cons h', List.map_cons, List.sum_cons]
   step as ⟨a', ha'⟩
   change a'.val = a.val + s.val[i].val at ha'
   change adler2.algo.Adler32.compute_loop2 ⟨s,i+1⟩ a' ⦃ _ ⦄
   have ih := sum_loop s (i+1) a' (by simp_all only [Slice.getElem_Nat_eq]; omega)
   apply WP.spec_mono ih
   intro z hz
   omega
 · rename_i h
   have hs : s.val.drop i = [] := List.drop_eq_nil_of_le (by simpa using Nat.le_of_not_gt h)
   simp [hs, sumValues, WP.spec_ok]
termination_by s.val.length - i
decreasing_by simp_all [Slice.len]; omega

theorem sum_loop3 (s : Slice U32) (i : Nat) (a : U32)
 (hbound : a.val + sumValues (s.val.drop i) ≤ 4294967295) :
 adler2.algo.Adler32.compute_loop3 ⟨s,i⟩ a
 ⦃ z => z.val = a.val + sumValues (s.val.drop i) ⦄ := by
 exact sum_loop s i a hbound
