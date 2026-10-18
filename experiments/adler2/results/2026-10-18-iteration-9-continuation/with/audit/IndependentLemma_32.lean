import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_32 : ∀ (xs ys : List U8) (a b : Nat), AdlerSpec.run a b (xs ++ ys) = AdlerSpec.run (AdlerSpec.run a b xs).1 (AdlerSpec.run a b xs).2 ys := AdlerProof.run_append
#check @AdlerProof.run_append
#print IndependentLemma_32
#print axioms IndependentLemma_32
