import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_25 : ∀ (it : core.slice.iter.Iter U8) (a b : U32) (ha : (AdlerProof.rawRun (it.slice.val.drop it.i) a.val b.val).1 ≤ U32.max) (hb : (AdlerProof.rawRun (it.slice.val.drop it.i) a.val b.val).2 ≤ U32.max), algo.Adler32.compute_loop4 it a b ⦃ fun out => (out.1.val, out.2.val) = AdlerProof.rawRun (it.slice.val.drop it.i) a.val b.val ⦄ := AdlerProof.tail_loop_wp
#check @AdlerProof.tail_loop_wp
#print IndependentLemma_25
#print axioms IndependentLemma_25
