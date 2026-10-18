import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_24 : ∀ (it : Aeneas.Std.core.slice.iter.Iter Aeneas.Std.U32) (a : Aeneas.Std.U32), a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ≤ 4294967295 → ElicitGenerated.algo.Adler32.compute_loop3 it a ⦃ fun out => out.val = a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ⦄ := AdlerSpec.sum_loop3_spec
#check @AdlerSpec.sum_loop3_spec
#print IndependentLemma_24
#print axioms IndependentLemma_24
