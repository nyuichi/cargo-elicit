import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_5 : ∀ {α : Type} (n : Nat) (hn : 0 < n) (xs : List α), (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧ (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧ (List.toChunksExact n hn xs).2.length < n := AdlerProof.chunks_partition
#check @AdlerProof.chunks_partition
#print IndependentLemma_5
#print axioms IndependentLemma_5
