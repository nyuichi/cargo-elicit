import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_17 : ∀ (xs ys : List (Slice U8)) (a b i : Nat), AdlerProof.laneRun (xs ++ ys) a b i = AdlerProof.laneRun ys (AdlerProof.laneRun xs a b i).1 (AdlerProof.laneRun xs a b i).2 i := AdlerProof.laneRun_append
#check @AdlerProof.laneRun_append
#print IndependentLemma_17
#print axioms IndependentLemma_17
