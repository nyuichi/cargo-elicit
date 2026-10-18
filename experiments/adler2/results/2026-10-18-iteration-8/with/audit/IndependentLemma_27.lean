import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_27 : ∀ (it : Aeneas.Std.core.slice.iter.Iter Aeneas.Std.U8) (a b : Aeneas.Std.U32), (let s := (it.slice.val.drop it.i).foldl AdlerSpec.rawStep (a.val, b.val); s.1 < 4294967296 ∧ s.2 < 4294967296) → ElicitGenerated.algo.Adler32.compute_loop4 it a b ⦃ fun out => (out.1.val, out.2.val) = (it.slice.val.drop it.i).foldl AdlerSpec.rawStep (a.val, b.val) ⦄ := AdlerSpec.tail_loop_spec
#check @AdlerSpec.tail_loop_spec
#print IndependentLemma_27
#print axioms IndependentLemma_27
