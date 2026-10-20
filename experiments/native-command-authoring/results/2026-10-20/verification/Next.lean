import Chunks
open Aeneas Aeneas.Std Result WP
namespace Verification
lemma next_eq {T : Type} (s : Slice T) (i : Nat) :
    core.slice.iter.IteratorSliceIter.next ⟨s,i⟩ =
    if h : i < s.val.length then ok (some s.val[i], ⟨s,i+1⟩) else ok (none,⟨s,i⟩) := by
  unfold core.slice.iter.IteratorSliceIter.next
  split
  · rename_i h
    have hi : i < s.val.length := by simpa only [Slice.len_val] using h
    rw [dif_pos hi]
    congr 2
  · rename_i h
    have hi : ¬ i < s.val.length := by simpa only [Slice.len_val] using h
    rw [dif_neg hi]
end Verification
