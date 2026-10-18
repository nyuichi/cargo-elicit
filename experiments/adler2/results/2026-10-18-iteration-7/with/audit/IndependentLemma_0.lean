import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_0 : ∀ (a b : Nat) (xs ys : List Aeneas.Std.U8), AdlerSpec.update a b (xs ++ ys) = AdlerSpec.update (AdlerSpec.update a b xs).1 (AdlerSpec.update a b xs).2 ys := AdlerSpec.update_append
#check @AdlerSpec.update_append
#print IndependentLemma_0
#print axioms IndependentLemma_0
