import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_25 : ∀ (s : Nat × Nat) (xs : List Aeneas.Std.U8), AdlerSpec.normalize (xs.foldl AdlerSpec.rawStep (AdlerSpec.normalize s)) = AdlerSpec.normalize (xs.foldl AdlerSpec.rawStep s) := AdlerSpec.normalize_raw_fold
#check @AdlerSpec.normalize_raw_fold
#print IndependentLemma_25
#print axioms IndependentLemma_25
