import Inner
open Aeneas Aeneas.Std Result WP
set_option maxHeartbeats 1500000
namespace Verification

lemma chunks_nat (n : Nat) (hn : 0 < n) (l : List U8) :
    (∀ c ∈ (List.toChunksExact n hn l).1, c.length = n) ∧
    (List.toChunksExact n hn l).1.flatten ++ (List.toChunksExact n hn l).2 = l ∧
    (List.toChunksExact n hn l).2.length < n := by
  rw [List.toChunksExact]
  split
  · rename_i h
    exact ⟨by simp, by simp, h⟩
  · rename_i h
    have ih := chunks_nat n hn (l.drop n)
    dsimp
    refine ⟨?_, ?_, ih.2.2⟩
    · intro c hc
      rcases List.mem_cons.mp hc with rfl | hc
      · simp only [List.length_take]; omega
      · exact ih.1 c hc
    · simp only [List.flatten_cons, List.append_assoc]
      rw [ih.2.1, List.take_append_drop]
termination_by l.length
decreasing_by simp only [List.length_drop]; omega

@[step] theorem chunks_spec (s : Slice U8) (n : Usize) (hn : n.val > 0) :
    core.slice.Slice.chunks_exact s n ⦃ r =>
      (∀ c ∈ r.chunks, c.val.length = n.val) ∧
      flat r.chunks ++ r.remainder.val = s.val ∧ r.remainder.val.length < n.val ⦄ := by
  rw [core.slice.Slice.chunks_exact, dif_pos hn]
  simp only [spec_ok]
  have h := chunks_nat n.val hn s.val
  constructor
  · intro c hc
    simp only [List.mem_map, List.mem_attach, true_and] at hc
    rcases hc with ⟨c',rfl⟩
    simpa only [Slice.from_val] using h.1 _ c'.property
  · constructor
    · simp only [flat, List.flatMap_map, Function.comp_def, Slice.from_val]
      rw [List.flatMap_subtype (g := id) (by intros; rfl), List.unattach_attach]
      simpa only [List.flatten_eq_flatMap] using h.2.1
    · simpa only [Slice.from_val] using h.2.2

lemma flat_length (cs : List (Slice U8)) (n : Nat) (hc : ∀ c ∈ cs, c.val.length = n) :
    (flat cs).length = cs.length * n := by
  induction cs with
  | nil => simp [flat]
  | cons c cs ih =>
    simp only [flat, List.flatMap_cons, List.length_append, List.length_cons]
    have h0 := hc c (by simp)
    have h1 : ∀ c ∈ cs, c.val.length = n := by intro c h; exact hc c (by simp [h])
    rw [h0, ih h1]
    ring

lemma four_complete (s : Slice U8) (it : core.slice.iter.ChunksExact U8)
    (hcs : ∀ c ∈ it.chunks, c.val.length = 4)
    (hf : flat it.chunks ++ it.remainder.val = s.val)
    (hr : it.remainder.val.length < 4) (hs : s.val.length % 4 = 0) :
    flat it.chunks = s.val ∧ it.chunks.length * 4 = s.val.length := by
  have hl := congrArg List.length hf
  rw [List.length_append, flat_length _ _ hcs] at hl
  have hz : it.remainder.val.length = 0 := by omega
  have he : it.remainder.val = [] := List.length_eq_zero_iff.mp hz
  constructor
  · simpa only [he, List.append_nil] using hf
  · omega

@[step] theorem inner_aligned (s : Slice U8) (it : core.slice.iter.ChunksExact U8) (x y : Q)
    (hcs : ∀ c ∈ it.chunks, c.val.length = 4) (hf : flat it.chunks = s.val)
    (hs : s.val.length ≤ 22208) (hx : x.bound 65520) (hy : y.bound 65520) :
    adler2.algo.Adler32.compute_loop0_loop0 it x.arr y.arr ⦃ r =>
      ∃ u v : Q, r = (u.arr,v.arr) ∧
      ∀ a b : F, combineQ a (b + (s.val.length : Nat)*a) u v =
        run (combineQ a b x y) s.val ⦄ := by
  have hl : it.chunks.length * 4 = s.val.length := by
    rw [← hf, flat_length _ _ hcs]
  apply WP.spec_mono (inner_correct it.chunks it.remainder 0 x y hcs (by omega)
    (Q.initial_ab x hx) (Q.initial_bb y hy))
  intro r hr
  rcases hr with ⟨u,v,hr,_,_,hm⟩
  refine ⟨u,v,hr,?_⟩
  intro a b
  simpa only [hf, Nat.mul_comm 4, hl] using hm a b
end Verification
