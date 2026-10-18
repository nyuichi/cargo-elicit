import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_27 : ∀ (it : core.slice.iter.Iter U8) (a b : U32) (hn : (it.slice.val.drop it.i).length ≤ 3) (ha : a.val ≤ 400000) (hb : b.val ≤ 2000000), algo.Adler32.compute_loop4 it a b ⦃ fun out => (out.1.val, out.2.val) = AdlerProof.rawRun (it.slice.val.drop it.i) a.val b.val ⦄ := AdlerProof.tail_bounded_wp
#check @AdlerProof.tail_bounded_wp
#print IndependentLemma_27
#print axioms IndependentLemma_27
