import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_11 : ∀ (s : Aeneas.Std.Slice Aeneas.Std.U8) (n : Aeneas.Std.Usize), 0 < n.val → Aeneas.Std.core.slice.Slice.chunks_exact s n ⦃ fun it => (it.chunks.map Aeneas.Std.Slice.val).flatten ++ it.remainder.val = s.val ∧ (∀ c ∈ it.chunks, c.val.length = n.val) ∧ it.remainder.val.length < n.val ⦄ := AdlerSpec.chunks_exact_spec
#check @AdlerSpec.chunks_exact_spec
#print IndependentLemma_11
#print axioms IndependentLemma_11
