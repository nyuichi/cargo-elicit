import Outer
open Aeneas Aeneas.Std Result ElicitGenerated AdlerMath
namespace Proofs

theorem iter_next_some {T : Type} (it : core.slice.iter.Iter T)
    (hi : it.i < it.slice.val.length) :
    core.slice.iter.IteratorSliceIter.next it =
      ok (some it.slice.val[it.i], {it with i := it.i+1}) := by
  unfold core.slice.iter.IteratorSliceIter.next
  rw [dif_pos (show it.i < it.slice.len by simpa using hi)]
  rfl

theorem iter_next_none {T : Type} (it : core.slice.iter.Iter T)
    (hi : ¬it.i < it.slice.val.length) :
    core.slice.iter.IteratorSliceIter.next it = ok (none, it) := by
  unfold core.slice.iter.IteratorSliceIter.next
  rw [dif_neg (show ¬it.i < it.slice.len by simpa using hi)]


def restSum (it : core.slice.iter.Iter U32) : Nat :=
  ((it.slice.val.drop it.i).map UScalar.val).sum

@[step] theorem sum_spec (it : core.slice.iter.Iter U32) (a : U32)
    (h : a.val + restSum it ≤ 4294967295) :
    algo.Adler32.compute_loop2 it a ⦃ out => out.val = a.val + restSum it ⦄ := by
  unfold algo.Adler32.compute_loop2
  rw [loop]
  by_cases hi : it.i < it.slice.val.length
  · have hd := List.drop_eq_getElem_cons hi
    have hs : restSum it = (it.slice.val[it.i]).val +
        restSum {it with i := it.i + 1} := by
      simp only [restSum, hd, List.map_cons, List.sum_cons]
    simp only [algo.Adler32.compute_loop2.body, iter_next_some it hi, bind_ok]
    step as ⟨a', ha'⟩ by simp_all [Slice.getElem_Nat_eq]; omega
    have hr := sum_spec {it with i := it.i + 1} a' (by omega)
    change algo.Adler32.compute_loop2 {it with i := it.i + 1} a'
      ⦃ out => out.val = a.val + restSum it ⦄
    step with hr
    omega
  · simp [algo.Adler32.compute_loop2.body, iter_next_none it hi, restSum, List.drop_eq_nil_of_le (by omega : it.slice.val.length ≤ it.i)]
termination_by it.slice.val.length - it.i
decreasing_by simp_all <;> omega

@[step] theorem sum3_spec (it : core.slice.iter.Iter U32) (a : U32)
    (h : a.val + restSum it ≤ 4294967295) :
    algo.Adler32.compute_loop3 it a ⦃ out => out.val = a.val + restSum it ⦄ := by
  have he : algo.Adler32.compute_loop3 = algo.Adler32.compute_loop2 := rfl
  rw [he]
  exact sum_spec it a h

def tailPost (it : core.slice.iter.Iter U8) (a b : U32) (out : U32 × U32) : Prop :=
  let n := (it.slice.val.drop it.i).length
  out.1.val ≤ a.val + 255*n ∧
  out.2.val ≤ b.val + n*(a.val + 255*n) ∧
  ((out.1.val : F), (out.2.val : F)) =
    scalar ((a.val : F), (b.val : F)) (it.slice.val.drop it.i)

@[step] theorem tail_spec (it : core.slice.iter.Iter U8) (a b : U32)
    (ha : a.val + 255*(it.slice.val.drop it.i).length ≤ 4294967295)
    (hb : b.val + (it.slice.val.drop it.i).length *
      (a.val + 255*(it.slice.val.drop it.i).length) ≤ 4294967295) :
    algo.Adler32.compute_loop4 it a b ⦃ tailPost it a b ⦄ := by
  unfold algo.Adler32.compute_loop4
  rw [loop]
  by_cases hi : it.i < it.slice.val.length
  · have hd := List.drop_eq_getElem_cons hi
    have hn : (it.slice.val.drop it.i).length =
        (it.slice.val.drop (it.i+1)).length + 1 := by rw [hd]; rfl
    simp only [algo.Adler32.compute_loop4.body, iter_next_some it hi, bind_ok, lift]
    have hbyte : (core.convert.num.FromU32U8.from it.slice.val[it.i]).val ≤ 255 := by simp; scalar_tac
    step as ⟨a', ha'⟩ by simp_all [Slice.getElem_Nat_eq]; omega
    step as ⟨b', hb'⟩
    case hmax =>
      simp [U32.max, U32.numBits, UScalarTy.numBits]
      simp only [hn] at hb
      nlinarith
    have hrec := tail_spec {it with i := it.i+1} a' b' (by
      simp only [hn] at ha; simp only; omega) (by
      simp only [hn] at hb; simp only; nlinarith)
    unfold algo.Adler32.compute_loop4 algo.Adler32.compute_loop4.body at hrec
    simp only [lift, bind_ok] at hrec
    step with hrec as ⟨out, hout⟩
    obtain ⟨hoa, hob, hoe⟩ := hout
    refine ⟨?_, ?_, ?_⟩
    · simp only [hn]; simp only at hoa; omega
    · simp only [hn]; simp only at hoa hob; nlinarith
    · rw [hd]
      simp only [scalar, List.foldl_cons]
      rw [hoe]
      congr 1
      simp [scalarStep, ha', hb', Nat.cast_add, add_assoc]
  · simp [algo.Adler32.compute_loop4.body, iter_next_none it hi, tailPost, scalar, List.drop_eq_nil_of_le (by omega : it.slice.val.length ≤ it.i)]
termination_by it.slice.val.length - it.i
decreasing_by simp_all <;> omega

end Proofs
