import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_26 : ∀ (bytes : List U8) (a b : Nat), ((AdlerProof.rawRun bytes a b).1 % 65521, (AdlerProof.rawRun bytes a b).2 % 65521) = AdlerSpec.run a b bytes := AdlerProof.rawRun_mod
#check @AdlerProof.rawRun_mod
#print IndependentLemma_26
#print axioms IndependentLemma_26
