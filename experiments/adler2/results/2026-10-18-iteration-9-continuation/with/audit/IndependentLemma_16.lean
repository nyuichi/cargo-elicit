import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_16 : ∀ (cs : List (Slice U8)) (a b c d i : Nat) (ha : a % 65521 = c % 65521) (hb : b % 65521 = d % 65521), (AdlerProof.laneRun cs a b i).1 % 65521 = (AdlerProof.laneRun cs c d i).1 % 65521 ∧ (AdlerProof.laneRun cs a b i).2 % 65521 = (AdlerProof.laneRun cs c d i).2 % 65521 := AdlerProof.laneRun_congr
#check @AdlerProof.laneRun_congr
#print IndependentLemma_16
#print axioms IndependentLemma_16
