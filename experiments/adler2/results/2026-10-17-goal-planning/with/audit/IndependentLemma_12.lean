import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_12 : ∀ {α : Type} {n : Nat} (hn : 0 < n) (xs : List α), ∀ c ∈ (Aeneas.Std.List.toChunksExact n hn xs).1, c.length = n := AdlerProof.exact_chunks_length
#check @AdlerProof.exact_chunks_length
#print IndependentLemma_12
#print axioms IndependentLemma_12
