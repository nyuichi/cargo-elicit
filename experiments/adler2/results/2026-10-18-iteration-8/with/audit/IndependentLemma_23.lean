import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_23 : ∀ (it : Aeneas.Std.core.slice.iter.Iter Aeneas.Std.U32) (a : Aeneas.Std.U32), a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ≤ 4294967295 → ElicitGenerated.algo.Adler32.compute_loop2 it a ⦃ fun out => out.val = a.val + ((it.slice.val.drop it.i).map (fun x => x.val)).sum ⦄ := AdlerSpec.sum_loop_spec
#check @AdlerSpec.sum_loop_spec
#print IndependentLemma_23
#print axioms IndependentLemma_23
