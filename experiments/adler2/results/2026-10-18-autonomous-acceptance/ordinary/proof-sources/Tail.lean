import Loops
open Aeneas Aeneas.Std Result ControlFlow
namespace Verification
open ElicitGenerated
set_option maxHeartbeats 2000000

theorem tail_budget (a b x m : Nat) (hx : x ≤ 255)
    (ha : a + 255*(m+1) ≤ U32.max)
    (hb : b + (m+1)*(a+255*(m+1)) ≤ U32.max) :
    a+x ≤ U32.max ∧ b+(a+x) ≤ U32.max ∧
    (a+x)+255*m ≤ U32.max ∧
    (b+(a+x))+m*((a+x)+255*m) ≤ U32.max := by
  have hm : 0 ≤ m := Nat.zero_le m
  constructor
  · omega
  constructor
  · nlinarith
  constructor
  · omega
  · nlinarith

theorem tail_loop (it : core.slice.iter.Iter U8) (a b : U32)
    (ha : a.val + 255*(it.slice.val.drop it.i).length ≤ U32.max)
    (hb : b.val + (it.slice.val.drop it.i).length *
      (a.val + 255*(it.slice.val.drop it.i).length) ≤ U32.max) :
    algo.Adler32.compute_loop4 it a b ⦃ a' b' =>
      (a'.val, b'.val) = raw (a.val, b.val) (it.slice.val.drop it.i) ⦄ := by
  unfold algo.Adler32.compute_loop4
  unfold loop
  unfold algo.Adler32.compute_loop4.body
  dsimp only
  by_cases hi : it.i < it.slice.val.length
  · have hn : core.slice.iter.IteratorSliceIter.next it =
        ok (some it.slice.val[it.i], { it with i := it.i + 1 }) := by
      simp [core.slice.iter.IteratorSliceIter.next, hi] <;> rfl
    have hs : it.slice.val.drop it.i =
        it.slice.val[it.i] :: it.slice.val.drop (it.i+1) := List.drop_eq_getElem_cons hi
    have hl := congrArg List.length hs
    simp only [List.length_cons] at hl
    have hx : it.slice.val[it.i].val ≤ 255 := by scalar_tac
    have budget := tail_budget a.val b.val it.slice.val[it.i].val
      (it.slice.val.drop (it.i+1)).length hx (by omega) (by simpa only [hl] using hb)
    rw [hn]
    simp only [lift]
    step as ⟨a', ha'⟩
    step as ⟨b', hb'⟩
    have hrec := tail_loop { it with i := it.i+1 } a' b'
      (by change a'.val + 255*(it.slice.val.drop (it.i+1)).length ≤ U32.max;
          simp only [core.convert.num.FromU32U8.from_val_eq] at ha'; omega)
      (by simp only [core.convert.num.FromU32U8.from_val_eq] at ha';
          simpa only [ha', hb'] using budget.2.2.2)
    have hrec' := WP.spec_mono hrec (P₁ := WP.uncurry' (fun ra rb =>
      (ra.val, rb.val) = raw (a.val, b.val) (it.slice.val.drop it.i))) (by
        intro r hr
        rcases r with ⟨ra, rb⟩
        change (ra.val, rb.val) = _ at hr
        change (ra.val, rb.val) = _
        simp only [core.convert.num.FromU32U8.from_val_eq] at ha'
        simpa only [raw, hs, List.foldl_cons, rawStep, ha', hb'] using hr)
    simpa only [algo.Adler32.compute_loop4, algo.Adler32.compute_loop4.body,
      lift, bind_tc_ok, bind_ok, WP.uncurry'] using hrec'
  · have hn : core.slice.iter.IteratorSliceIter.next it = ok (none, it) := by
      simp [core.slice.iter.IteratorSliceIter.next, hi] <;> rfl
    rw [hn]
    simp [raw, List.drop_eq_nil_of_le (by omega : it.slice.val.length ≤ it.i)]
termination_by it.slice.val.length - it.i
 decreasing_by simp_all; omega

#print axioms tail_loop
end Verification
