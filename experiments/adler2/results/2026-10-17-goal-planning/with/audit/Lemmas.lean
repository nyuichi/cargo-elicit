import ElicitContracts
open Aeneas Aeneas.Std
namespace AdlerProof

theorem exact_chunks_length {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
    ∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n := by
  unfold List.toChunksExact
  split
  · simp
  · simp only [List.mem_cons]
    intro c hc
    rcases hc with rfl | hc
    · simp only [List.length_take]
      omega
    · exact exact_chunks_length hn _ c hc
termination_by xs.length
decreasing_by simp [List.length_drop]; omega

theorem exact_chunks_remainder_lt {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
    (List.toChunksExact n hn xs).2.length < n := by
  unfold List.toChunksExact
  split
  · simpa
  · exact exact_chunks_remainder_lt hn _
termination_by xs.length
decreasing_by simp [List.length_drop]; omega

theorem exact_chunks_reconstruct {α : Type} {n : Nat} (hn : 0 < n) (xs : List α) :
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs := by
  unfold List.toChunksExact
  split
  · simp
  · simp only [List.flatten_cons, List.append_assoc]
    rw [exact_chunks_reconstruct hn (xs.drop n)]
    exact List.take_append_drop n xs
termination_by xs.length
decreasing_by simp [List.length_drop]; omega

end AdlerProof
