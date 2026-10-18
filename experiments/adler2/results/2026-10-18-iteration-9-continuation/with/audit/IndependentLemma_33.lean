import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_33 : ∀ (xs ys : List (Slice U8)) (av bv av' bv' : algo.U32X4) (hseed : ∀ i : Nat, i < 4 → av.val[i]!.val = (AdlerProof.laneRun xs 0 0 i).1 % 65521 ∧ bv.val[i]!.val = (AdlerProof.laneRun xs 0 0 i).2 % 65521) (hfinal : ∀ i : Nat, i < 4 → (av'.val[i]!.val, bv'.val[i]!.val) = AdlerProof.laneRun ys av.val[i]!.val bv.val[i]!.val i), ∀ i : Nat, i < 4 → av'.val[i]!.val % 65521 = (AdlerProof.laneRun (xs ++ ys) 0 0 i).1 % 65521 ∧ bv'.val[i]!.val % 65521 = (AdlerProof.laneRun (xs ++ ys) 0 0 i).2 % 65521 := AdlerProof.combine_lane_residues
#check @AdlerProof.combine_lane_residues
#print IndependentLemma_33
#print axioms IndependentLemma_33
