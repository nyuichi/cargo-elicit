import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_5 : ∀ (s : Nat × Nat) (x : Aeneas.Std.U8), AdlerSpec.step s x = AdlerSpec.normalize (AdlerSpec.rawStep s x) := AdlerSpec.step_eq_normalize_raw
#check @AdlerSpec.step_eq_normalize_raw
#print IndependentLemma_5
#print axioms IndependentLemma_5
