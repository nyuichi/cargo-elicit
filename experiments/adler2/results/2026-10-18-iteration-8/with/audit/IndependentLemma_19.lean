import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_19 : ∀ (it : Aeneas.Std.core.slice.iter.ChunksExact Aeneas.Std.U8) (xs : List Aeneas.Std.U8) (n : Nat), (it.chunks.map Aeneas.Std.Slice.val).flatten ++ it.remainder.val = xs → (∀ c ∈ it.chunks, c.val.length = n) → it.chunks.length * n + it.remainder.val.length = xs.length := AdlerSpec.chunks_count_eq
#check @AdlerSpec.chunks_count_eq
#print IndependentLemma_19
#print axioms IndependentLemma_19
