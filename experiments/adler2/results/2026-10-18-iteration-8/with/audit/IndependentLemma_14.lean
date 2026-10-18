import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_14 : ∀ (s : Nat × Nat) (xs : List Aeneas.Std.U8), s.1 ≤ (xs.foldl AdlerSpec.rawStep s).1 ∧ s.2 ≤ (xs.foldl AdlerSpec.rawStep s).2 := AdlerSpec.raw_fold_mono
#check @AdlerSpec.raw_fold_mono
#print IndependentLemma_14
#print axioms IndependentLemma_14
