import Inner
open Aeneas Aeneas.Std Result
namespace AdlerProof
set_option maxHeartbeats 2000000
set_option maxRecDepth 4096

theorem chunks_properties {α : Type} (n : Nat) (hn : 0 < n) (l : List α) :
 let r := Aeneas.Std.List.toChunksExact n hn l
 (∀ c ∈ r.1, c.length = n) ∧ r.2.length < n ∧
 r.1.flatten ++ r.2 = l ∧ n*r.1.length + r.2.length = l.length := by
 rw [Aeneas.Std.List.toChunksExact]
 split
 · rename_i h
   simp_all
 · rename_i h
   have ih := chunks_properties n hn (l.drop n)
   dsimp at ih ⊢
   rcases ih with ⟨hc, hr, hf, hl⟩
   have htake : (l.take n).length = n := by simp; omega
   refine ⟨?_, hr, ?_, ?_⟩
   · intro c hm
     simp only [List.mem_cons] at hm
     rcases hm with rfl | hm
     · exact htake
     · exact hc c hm
   · simp only [List.flatten_cons, List.append_assoc]
     rw [hf]
     exact List.take_append_drop n l
   · simp only [List.length_cons, List.length_drop, Nat.mul_add, Nat.mul_one] at *
     omega
termination_by l.length
decreasing_by simp; omega

@[step] theorem chunks_exact_spec {α : Type} (s : Slice α) (n : Usize) (hn : 0 < n.val) :
 core.slice.Slice.chunks_exact s n ⦃ it =>
 (∀ c ∈ it.chunks, c.val.length = n.val) ∧ it.remainder.val.length < n.val ∧
 (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ∧
 n.val*it.chunks.length + it.remainder.val.length = s.val.length ⦄ := by
  unfold core.slice.Slice.chunks_exact
  simp only [hn, ↓reduceDIte, WP.spec_ok]
  have h := chunks_properties n.val hn s.val
  constructor
  · intro c hc
    simp only [List.mem_map] at hc
    obtain ⟨x, _, rfl⟩ := hc
    simpa using h.1 x.val x.property
  · simpa using h.2

@[step] theorem chunks_four (s : Slice U8) (hdiv : s.val.length % 4 = 0) :
 core.slice.Slice.chunks_exact s 4#usize ⦃ it =>
 (∀ c ∈ it.chunks, c.val.length = 4) ∧ it.remainder.val = [] ∧
 (it.chunks.map Slice.val).flatten = s.val ∧ 4*it.chunks.length = s.val.length ⦄ := by
 have h := chunks_exact_spec s 4#usize (by decide)
 apply WP.spec_mono h
 intro it ⟨hc,hr,hf,hl⟩
 have hl' : 4*it.chunks.length + it.remainder.val.length = s.val.length := hl
 have hr' : it.remainder.val.length < 4 := hr
 have hempty : it.remainder.val = [] := List.eq_nil_of_length_eq_zero (by omega)
 exact ⟨hc, hempty, by simpa [hempty] using hf, by simpa [hempty] using hl'⟩
