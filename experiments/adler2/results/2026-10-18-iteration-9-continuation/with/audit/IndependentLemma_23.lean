import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_23 : ∀ (it : core.slice.iter.Iter U32) (a : U32) (hmax : a.val + AdlerProof.iterSum it ≤ U32.max), algo.Adler32.compute_loop3 it a ⦃ fun out => out.val = a.val + AdlerProof.iterSum it ⦄ := AdlerProof.sum_loop3_wp
#check @AdlerProof.sum_loop3_wp
#print IndependentLemma_23
#print axioms IndependentLemma_23
