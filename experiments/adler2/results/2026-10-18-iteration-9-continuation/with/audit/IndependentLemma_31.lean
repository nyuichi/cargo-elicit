import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_31 : ∀ (bytes : List U8) (a b c d : Nat) (ha : a % 65521 = c % 65521) (hb : b % 65521 = d % 65521), AdlerSpec.run a b bytes = AdlerSpec.run c d bytes := AdlerProof.run_congr
#check @AdlerProof.run_congr
#print IndependentLemma_31
#print axioms IndependentLemma_31
