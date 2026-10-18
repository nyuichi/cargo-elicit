import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_4 : ∀ {α : Type} (n : Nat) (hn : 0 < n) (xs : List α), (List.toChunksExact n hn xs).1.flatten ++ (List.toChunksExact n hn xs).2 = xs ∧ (∀ c ∈ (List.toChunksExact n hn xs).1, c.length = n) ∧ (List.toChunksExact n hn xs).2.length < n := AdlerSpec.chunks_partition
#check @AdlerSpec.chunks_partition
#print IndependentLemma_4
#print axioms IndependentLemma_4
