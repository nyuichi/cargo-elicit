import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_14 : ∀  {α : Type} {n : Nat} (hn : 0 < n) (xs : List α),
     (List.toChunksExact n hn xs).1.length * n ≤ xs.length := AdlerProof.chunks_count_bound
#check @AdlerProof.chunks_count_bound
#print IndependentLemma_14
#print axioms IndependentLemma_14
