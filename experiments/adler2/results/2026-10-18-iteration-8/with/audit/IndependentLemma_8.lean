import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_8 : ∀ (s : Nat × Nat) (xs : List Aeneas.Std.U8), (xs.foldl AdlerSpec.rawStep s).1 ≤ s.1 + 255 * xs.length ∧ 2 * (xs.foldl AdlerSpec.rawStep s).2 ≤ 2 * s.2 + 2 * xs.length * s.1 + 255 * xs.length * (xs.length + 1) := AdlerSpec.raw_fold_bounds
#check @AdlerSpec.raw_fold_bounds
#print IndependentLemma_8
#print axioms IndependentLemma_8
