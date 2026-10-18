import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_1 : ∀ {α : Type} {n : Nat} (hn : 0 < n) (xs : List α), ∀ c ∈ (Aeneas.Std.List.toChunksExact n hn xs).1, c.length = n := AdlerProof.chunks_exact_lengths
#check @AdlerProof.chunks_exact_lengths
#print IndependentLemma_1
#print axioms IndependentLemma_1
