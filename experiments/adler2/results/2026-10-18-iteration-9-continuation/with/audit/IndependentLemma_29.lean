import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_29 : ∀ (cs : List (Slice U8)) (hfull : ∀ c ∈ cs, c.val.length = 4) (a b : Nat), let l := fun i => AdlerProof.laneRun cs 0 0 i; let av := fun i => (l i).1 % 65521; let bv := fun i => (l i).2 % 65521; ((a + av 0 + av 1 + av 2 + av 3) % 65521, ((b + 4 * cs.length * a) % 65521 + 4 * (bv 0 + bv 1 + bv 2 + bv 3) + (65521 - av 1) + 2 * (65521 - av 2) + 3 * (65521 - av 3)) % 65521) = AdlerSpec.run a b (cs.map Slice.val).flatten := AdlerProof.quartet_mod_recombine
#check @AdlerProof.quartet_mod_recombine
#print IndependentLemma_29
#print axioms IndependentLemma_29
