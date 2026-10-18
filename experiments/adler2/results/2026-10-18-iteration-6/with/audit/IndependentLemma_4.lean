import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_4 : ∀ {α : Type} (s : Aeneas.Std.Slice α) (n : Aeneas.Std.Usize) (hn : 0 < n.val), Aeneas.Std.core.slice.Slice.chunks_exact s n ⦃ it => it.chunks.map Aeneas.Std.Slice.val = (Aeneas.Std.List.toChunksExact n.val hn s.val).1 ∧ it.remainder.val = (Aeneas.Std.List.toChunksExact n.val hn s.val).2 ∧ (∀ c ∈ it.chunks, c.length = n.val) ∧ it.remainder.length < n.val ∧ (it.chunks.map Aeneas.Std.Slice.val).flatten ++ it.remainder.val = s.val ⦄ := AdlerProof.chunks_exact_spec
#check @AdlerProof.chunks_exact_spec
#print IndependentLemma_4
#print axioms IndependentLemma_4
