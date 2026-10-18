import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_6 : ∀ {α : Type} (s : Slice α) (n : Usize), 0 < n.val → core.slice.Slice.chunks_exact s n ⦃ fun it => (it.chunks.map Slice.val).flatten ++ it.remainder.val = s.val ∧ (∀ c ∈ it.chunks, c.val.length = n.val) ∧ it.remainder.val.length < n.val ⦄ := AdlerProof.chunks_exact_spec
#check @AdlerProof.chunks_exact_spec
#print IndependentLemma_6
#print axioms IndependentLemma_6
