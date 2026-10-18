import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_18 : ∀ (cs : List (Aeneas.Std.Slice Aeneas.Std.U8)) (n : Nat), (∀ c ∈ cs, c.val.length = n) → (cs.map Aeneas.Std.Slice.val).flatten.length = cs.length * n := AdlerSpec.slices_flatten_length
#check @AdlerSpec.slices_flatten_length
#print IndependentLemma_18
#print axioms IndependentLemma_18
