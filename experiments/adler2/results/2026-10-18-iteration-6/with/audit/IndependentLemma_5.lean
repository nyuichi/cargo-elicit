import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_5 : ∀ {α : Type} (it : Aeneas.Std.core.slice.iter.ChunksExact α), it.getRemainder ⦃ r => r = it.remainder ⦄ := AdlerProof.chunks_remainder_spec
#check @AdlerProof.chunks_remainder_spec
#print IndependentLemma_5
#print axioms IndependentLemma_5
