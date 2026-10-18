import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_7 : ∀ (s : Nat × Nat) (xs : List Aeneas.Std.U8), AdlerSpec.update s xs = AdlerSpec.normalize (xs.foldl AdlerSpec.rawStep s) := AdlerSpec.update_eq_raw
#check @AdlerSpec.update_eq_raw
#print IndependentLemma_7
#print axioms IndependentLemma_7
