import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_13 : ∀ {α : Type} {n : Nat} (hn : 0 < n) (xs : List α), (Aeneas.Std.List.toChunksExact n hn xs).2.length < n := AdlerProof.exact_chunks_remainder_lt
#check @AdlerProof.exact_chunks_remainder_lt
#print IndependentLemma_13
#print axioms IndependentLemma_13
