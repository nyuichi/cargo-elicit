import Specs
open Aeneas Aeneas.Std Result
namespace Chunks
 theorem raw_partition {α : Type} (n : Nat) (hn : 0 < n) (xs : List α) :
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧
    (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧
    (List.toChunksExact n hn xs).2.length < n := by
  rw [List.toChunksExact]
  split
  · simp_all
  · rename_i h
    have ih := raw_partition n hn (xs.drop n)
    simp only [List.flatten_cons, List.append_assoc]
    refine ⟨?_, ?_, ih.2.2⟩
    · rw [ih.1, List.take_append_drop]
    · intro c hc
      simp only [List.mem_cons] at hc
      rcases hc with rfl | hc
      · simp [List.length_take, Nat.min_eq_left (by omega : n ≤ xs.length)]
      · exact ih.2.1 c hc
termination_by xs.length
 decreasing_by simp [List.length_drop]; omega

def values {α : Type} (cs : List (Slice α)) : List α := (cs.map Slice.val).flatten
@[simp] theorem values_nil {α : Type} : values ([] : List (Slice α)) = [] := rfl
@[simp] theorem values_cons {α : Type} (c : Slice α) (cs : List (Slice α)) :
    values (c :: cs) = c.val ++ values cs := rfl
@[step] theorem exact_spec {α : Type} (s : Slice α) (n : Usize) (hn : 0 < n.val) :
    core.slice.Slice.chunks_exact s n ⦃ it =>
      values it.chunks ++ it.remainder.val = s.val ∧
      (∀ c ∈ it.chunks, c.val.length = n.val) ∧
      it.remainder.val.length < n.val ⦄ := by
  unfold core.slice.Slice.chunks_exact
  simp only [hn, _root_.dite_true, WP.spec_ok, values, List.map_map]
  have h := raw_partition n.val hn s.val
  constructor
  · simpa using h.1
  constructor
  · intro c hc
    simp only [List.mem_map, List.mem_attach, true_and, Subtype.exists] at hc
    obtain ⟨c', hc', heq⟩ := hc
    subst c
    simpa using h.2.1 c' hc'
  · simpa using h.2.2
 theorem values_length {α : Type} (cs : List (Slice α)) (n : Nat)
    (h : ∀ c ∈ cs, c.val.length = n) : (values cs).length = cs.length * n := by
  induction cs with
  | nil => simp
  | cons c cs ih =>
    have hc := h c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = n := by
      intro d hd
      exact h d (by simp [hd])
    simp [hc, ih ht, Nat.add_mul, Nat.add_comm]
@[step] theorem four_spec (s : Slice U8) (hm : s.val.length % 4 = 0) :
    core.slice.Slice.chunks_exact s 4#usize ⦃ it =>
      values it.chunks = s.val ∧
      (∀ c ∈ it.chunks, c.val.length = 4) ∧
      it.chunks.length = s.val.length / 4 ⦄ := by
  apply WP.spec_mono (exact_spec s 4#usize (by decide))
  intro it hi
  have hl := values_length it.chunks 4 hi.2.1
  have hp := congrArg List.length hi.1
  simp only [List.length_append] at hp
  have hlt : it.remainder.val.length < 4 := hi.2.2
  have hr : it.remainder.val.length = 0 := by omega
  have he : it.remainder.val = [] := List.length_eq_zero_iff.mp hr
  refine ⟨?_, hi.2.1, ?_⟩
  · simpa [he] using hi.1
  · omega
end Chunks
