import Algebra
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Tail
open Vector LaneAlgebra
abbrev rest {α : Type} (it : core.slice.iter.Iter α) : List α := it.slice.val.drop it.i
def total (it : core.slice.iter.Iter U32) : Nat := (rest it |>.map (fun x => x.val)).sum
 theorem next_some {α : Type} (it : core.slice.iter.Iter α) (hi : it.i < it.slice.val.length) :
    core.slice.iter.IteratorSliceIter.next it = ok (some it.slice.val[it.i], {it with i := it.i+1}) := by
  unfold core.slice.iter.IteratorSliceIter.next
  rw [dif_pos (by simpa using hi)]
  rfl
 theorem next_none {α : Type} (it : core.slice.iter.Iter α) (hi : it.slice.val.length ≤ it.i) :
    core.slice.iter.IteratorSliceIter.next it = ok (none, it) := by
  unfold core.slice.iter.IteratorSliceIter.next
  rw [dif_neg (by simpa using Nat.not_lt.mpr hi)]
@[step] theorem sum_spec (it : core.slice.iter.Iter U32) (a : U32)
    (h : a.val + total it ≤ 4294967295) :
    algo.Adler32.compute_loop2 it a ⦃ out => out.val = a.val + total it ⦄ := by
  unfold algo.Adler32.compute_loop2
  rw [loop]
  dsimp only
  rw [algo.Adler32.compute_loop2.body]
  by_cases hi : it.i < it.slice.val.length
  · rw [next_some it hi]
    have hil := hi
    have hr := List.drop_eq_getElem_cons hil
    have ht : total it = it.slice.val[it.i].val + total {it with i := it.i+1} := by
      simp only [total, rest, hr, List.map_cons, List.sum_cons]
    step as ⟨a1, ha1⟩
    have hh : a1.val + total {it with i := it.i+1} ≤ 4294967295 := by omega
    change algo.Adler32.compute_loop2 {it with i := it.i+1} a1 ⦃ _ ⦄
    apply WP.spec_mono (sum_spec _ a1 hh)
    intro out hout
    omega
  · rw [next_none it (by omega)]
    have hr : rest it = [] := List.drop_eq_nil_iff.mpr (by simpa using Nat.le_of_not_gt hi)
    simp [hr, total]
termination_by it.slice.val.length - it.i
 decreasing_by simp_wf; omega
@[step] theorem sum_b_spec (it : core.slice.iter.Iter U32) (b : U32)
    (h : b.val + total it ≤ 4294967295) :
    algo.Adler32.compute_loop3 it b ⦃ out => out.val = b.val + total it ⦄ := by
  exact sum_spec it b h
@[step] theorem serial_spec (it : core.slice.iter.Iter U8) (a b : U32)
    (h : Inner.budget (rest it).length a.val b.val) :
    algo.Adler32.compute_loop4 it a b ⦃ out =>
      ((out.1.val : Z), (out.2.val : Z)) = run ((a.val : Z), (b.val : Z)) (rest it) ⦄ := by
  unfold algo.Adler32.compute_loop4
  rw [loop]
  dsimp only
  rw [algo.Adler32.compute_loop4.body]
  by_cases hi : it.i < it.slice.val.length
  · rw [next_some it hi]
    have hil := hi
    have hr := List.drop_eq_getElem_cons hil
    have hlen : (rest it).length = (rest {it with i := it.i+1}).length + 1 := by
      simp only [rest, hr, List.length_cons]
    have hb := Inner.budget_step (rest {it with i := it.i+1}).length a.val b.val
      it.slice.val[it.i].val (by scalar_tac) (by rwa [← hlen])
    simp only [bind_tc_ok, lift]
    step as ⟨a1, ha1⟩
    step as ⟨b1, hb1⟩
    have hh : Inner.budget (rest {it with i := it.i+1}).length a1.val b1.val := by
      simpa only [ha1, hb1, core.convert.num.FromU32U8.from_val_eq, Slice.getElem_Nat_eq] using hb.2.2
    change algo.Adler32.compute_loop4 {it with i := it.i+1} a1 b1 ⦃ _ ⦄
    apply WP.spec_mono (serial_spec _ a1 b1 hh)
    intro out hout
    rw [hout]
    simp only [rest, hr, run, List.foldl_cons, step]
    congr 1 <;> simp [ha1, hb1, core.convert.num.FromU32U8.from_val_eq, Nat.cast_add, Slice.getElem_Nat_eq]
  · rw [next_none it (by omega)]
    have hr : rest it = [] := List.drop_eq_nil_iff.mpr (by simpa using Nat.le_of_not_gt hi)
    simp [hr, run]
termination_by it.slice.val.length - it.i
 decreasing_by simp_wf; omega
end Tail
