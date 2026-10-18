import ElicitGenerated
open Aeneas Aeneas.Std Result
set_option maxHeartbeats 2000000
set_option linter.unusedSimpArgs false
namespace Verification

theorem chunks_properties {T : Type} (n : Nat) (hn : 0 < n) (xs : List T) :
    (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧
    (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧
    (List.toChunksExact n hn xs).2.length < n := by
  rw [List.toChunksExact]
  split
  next h => simp_all
  next h =>
    have ih := chunks_properties n hn (xs.drop n)
    have htake : (xs.take n).length = n := by simp; omega
    constructor
    · simp only [List.mem_cons, forall_eq_or_imp]
      exact ⟨htake, ih.1⟩
    constructor
    · simp only [List.flatten_cons, List.append_assoc]
      rw [ih.2.1]
      exact List.take_append_drop n xs
    · exact ih.2.2
termination_by xs.length
 decreasing_by simp; omega

theorem chunks_exact_correct (xs : Slice U8) (n : Usize) (hn : n.val > 0) :
    core.slice.Slice.chunks_exact xs n ⦃ it =>
      (∀ c ∈ it.chunks, c.val.length = n.val) ∧
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = xs.val ∧
      it.remainder.val.length < n.val ⦄ := by
  unfold core.slice.Slice.chunks_exact
  simp only [hn, _root_.dite_true, WP.spec_ok]
  have hh := chunks_properties n.val hn xs.val
  simp_all only [List.mem_map, List.mem_attach, true_and, Subtype.exists,
    exists_and_left, List.map_map, Function.comp_def, Slice.from_val,
    List.attach_map_val, forall_exists_index, and_imp]
  constructor
  · intro c x hx heq
    subst c
    simpa using hh.1 x hx
  · simpa only [List.attach_map_subtype_val, hh.2.1, and_true]

theorem uniform_flatten_length (cs : List (Slice U8)) (n : Nat)
    (h : ∀ c ∈ cs, c.val.length = n) :
    (cs.map Slice.val).flatten.length = cs.length * n := by
  induction cs with
  | nil => simp
  | cons c cs ih =>
    have hc := h c (by simp)
    have ht : ∀ c ∈ cs, c.val.length = n := by simp_all
    simp [hc, ih ht, Nat.add_mul, Nat.add_comm]

theorem chunks_count (xs : Slice U8) (it : core.slice.iter.ChunksExact U8) (n : Nat)
    (h : (∀ c ∈ it.chunks, c.val.length = n) ∧
      (it.chunks.map Slice.val).flatten ++ it.remainder.val = xs.val) :
    it.chunks.length * n + it.remainder.val.length = xs.val.length := by
  have hh := congrArg List.length h.2
  simpa [uniform_flatten_length it.chunks n h.1] using hh

attribute [step] chunks_exact_correct
#print axioms chunks_exact_correct
#print axioms chunks_count
