import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_8 : ∀ (s : Aeneas.Std.Slice Aeneas.Std.U32) (i : Nat) (a : Aeneas.Std.U32)
    (h : a.val + ((s.val.drop i).map (fun x => x.val)).sum ≤ Aeneas.Std.U32.max),
    ElicitGenerated.algo.Adler32.compute_loop2 {slice := s, i := i} a ⦃ out =>
      out.val = a.val + ((s.val.drop i).map (fun x => x.val)).sum ⦄ := AdlerSpec.sum_loop
#check @AdlerSpec.sum_loop
#print IndependentLemma_8
#print axioms IndependentLemma_8
