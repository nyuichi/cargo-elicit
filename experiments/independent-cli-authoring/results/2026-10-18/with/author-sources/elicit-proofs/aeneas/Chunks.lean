import Inner
open Aeneas Aeneas.Std Result ElicitGenerated
namespace Proofs

theorem chunks_facts {α : Type} (n : Nat) (hn : 0 < n) (xs : List α) :
    (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧
    (List.toChunksExact n hn xs).2.length < n ∧
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs := by
  rw [List.toChunksExact]
  split
  · simp_all
  · rename_i h
    have ih := chunks_facts n hn (xs.drop n)
    simp only [List.mem_cons, List.flatten_cons, List.append_assoc]
    refine ⟨?_, ih.2.1, ?_⟩
    · intro c hc
      rcases hc with rfl | hc
      · simp only [List.length_take, Nat.min_eq_left (show n ≤ xs.length by omega)]
      · exact ih.1 c hc
    · rw [ih.2.2, List.take_append_drop]
termination_by xs.length
decreasing_by simp; omega

def chunkFacts (s : Slice U8) (n : Nat) (it : Chunks) : Prop :=
  (∀ c ∈ it.chunks, c.val.length = n) ∧
  it.remainder.val.length < n ∧ flatten it.chunks ++ it.remainder.val = s.val

@[step] theorem chunks_spec (s : Slice U8) (n : Usize) (hn : 0 < n.val) :
    core.slice.Slice.chunks_exact s n ⦃ chunkFacts s n.val ⦄ := by
  simp only [core.slice.Slice.chunks_exact, hn, ↓reduceDIte, WP.spec_ok]
  have hf := chunks_facts n.val hn s.val
  constructor
  · intro c hc
    simp only [List.mem_map] at hc
    obtain ⟨⟨cs,hcs⟩, _, rfl⟩ := hc
    simpa using hf.1 cs hcs
  · constructor
    · simpa using hf.2.1
    · simpa [flatten, List.flatMap_def, List.map_map, Function.comp_def] using hf.2.2

theorem flatten_length (cs : List (Slice U8)) (n : Nat)
    (h : ∀ c ∈ cs, c.val.length = n) :
    (flatten cs).length = cs.length * n := by
  induction cs with
  | nil => simp [flatten]
  | cons c cs ih =>
    have hc := h c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = n := by intro d hd; exact h d (by simp [hd])
    change (c.val ++ flatten cs).length = (cs.length + 1) * n
    rw [List.length_append, hc, ih ht]
    simp [Nat.add_mul, Nat.add_comm]

theorem chunkFacts_length (s : Slice U8) (n : Nat) (it : Chunks)
    (h : chunkFacts s n it) :
    it.chunks.length * n + it.remainder.val.length = s.val.length := by
  have := congrArg List.length h.2.2
  simpa [List.length_append, flatten_length it.chunks n h.1] using this

theorem chunks_divisible (s : Slice U8) (n : Nat) (it : Chunks)
    (h : chunkFacts s n it) (hm : s.val.length % n = 0) :
    it.remainder.val = [] ∧ flatten it.chunks = s.val := by
  have hl := chunkFacts_length s n it h
  have hb := h.2.1
  have hmod := congrArg (fun x => x % n) hl
  simp only [Nat.add_mod, Nat.mul_mod_left, Nat.zero_add, Nat.mod_mod,
    Nat.mod_eq_of_lt hb] at hmod
  have hr : it.remainder.val.length = 0 := by omega
  have he := List.eq_nil_of_length_eq_zero hr
  exact ⟨he, by simpa [he] using h.2.2⟩

end Proofs
