import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_2 : ∀ {α : Type} {n : Nat} (hn : 0 < n) (xs : List α), (Aeneas.Std.List.toChunksExact n hn xs).2.length < n := AdlerProof.chunks_remainder_lt
#check @AdlerProof.chunks_remainder_lt
#print IndependentLemma_2
#print axioms IndependentLemma_2
