import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_14 : ∀ {α : Type} {n : Nat} (hn : 0 < n) (xs : List α), (Aeneas.Std.List.toChunksExact n hn xs).1.flatten ++ (Aeneas.Std.List.toChunksExact n hn xs).2 = xs := AdlerProof.exact_chunks_reconstruct
#check @AdlerProof.exact_chunks_reconstruct
#print IndependentLemma_14
#print axioms IndependentLemma_14
