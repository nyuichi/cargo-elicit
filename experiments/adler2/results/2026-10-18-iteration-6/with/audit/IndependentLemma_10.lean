import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_10 : ∀  (k : Nat),
     (k + 1) * (k + 1 + 1) / 2 = k * (k + 1) / 2 + (k + 1) := AdlerProof.triangular_succ
#check @AdlerProof.triangular_succ
#print IndependentLemma_10
#print axioms IndependentLemma_10
