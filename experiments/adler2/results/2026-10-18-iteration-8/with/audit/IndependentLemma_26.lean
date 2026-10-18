import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_26 : ∀ (s : Nat × Nat) (xs ys : List Aeneas.Std.U8), AdlerSpec.update s (xs ++ ys) = AdlerSpec.update (AdlerSpec.update s xs) ys := AdlerSpec.update_append
#check @AdlerSpec.update_append
#print IndependentLemma_26
#print axioms IndependentLemma_26
