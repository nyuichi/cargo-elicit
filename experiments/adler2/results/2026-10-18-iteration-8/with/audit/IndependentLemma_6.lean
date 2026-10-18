import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_6 : ∀ (s : Nat × Nat) (x : Aeneas.Std.U8), AdlerSpec.normalize (AdlerSpec.rawStep (AdlerSpec.normalize s) x) = AdlerSpec.normalize (AdlerSpec.rawStep s x) := AdlerSpec.normalize_rawStep
#check @AdlerSpec.normalize_rawStep
#print IndependentLemma_6
#print axioms IndependentLemma_6
