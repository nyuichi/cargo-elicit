import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_28 : ∀ (cs : List (Slice U8)) (hfull : ∀ c ∈ cs, c.val.length = 4) (a b : Nat), let s := AdlerProof.rawRun (cs.map Slice.val).flatten a b; let l := fun i => AdlerProof.laneRun cs 0 0 i; s.1 = a + (l 0).1 + (l 1).1 + (l 2).1 + (l 3).1 ∧ s.2 + (l 1).1 + 2 * (l 2).1 + 3 * (l 3).1 = b + 4 * cs.length * a + 4 * ((l 0).2 + (l 1).2 + (l 2).2 + (l 3).2) := AdlerProof.quartet_recombine
#check @AdlerProof.quartet_recombine
#print IndependentLemma_28
#print axioms IndependentLemma_28
