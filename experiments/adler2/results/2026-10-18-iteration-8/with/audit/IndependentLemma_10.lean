import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_10 : ∀ (s : Nat × Nat) (x0 x1 x2 x3 : Aeneas.Std.U8), [x0,x1,x2,x3].foldl AdlerSpec.rawStep s = (s.1 + x0.val + x1.val + x2.val + x3.val, s.2 + 4 * s.1 + 4 * x0.val + 3 * x1.val + 2 * x2.val + x3.val) := AdlerSpec.raw_four
#check @AdlerSpec.raw_four
#print IndependentLemma_10
#print axioms IndependentLemma_10
