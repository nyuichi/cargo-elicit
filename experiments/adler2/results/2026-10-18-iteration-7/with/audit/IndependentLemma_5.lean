import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_5 : ∀ (cs : List (Aeneas.Std.Slice Aeneas.Std.U8)) (rem : Aeneas.Std.Slice Aeneas.Std.U8)
    (a b : Aeneas.Std.U32) (av bv : ElicitGenerated.algo.U32X4)
    (hlen : ∀ c ∈ cs, c.val.length = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535)
    (hav : ∀ i, AdlerSpec.laneVal av i ≤ 65520) (hbv : ∀ i, AdlerSpec.laneVal bv i ≤ 65520),
    ElicitGenerated.algo.Adler32.compute_loop0 22208#usize {chunks := cs, remainder := rem} a b av bv ⦃ out =>
      ∃ groups : List (List (Aeneas.Std.Slice Aeneas.Std.U8)),
        List.Forall₂ AdlerSpec.blockRep groups cs ∧
        (out.1.val, AdlerSpec.vals out.2.1, AdlerSpec.vals out.2.2) = groups.foldl (AdlerSpec.outerStep a.val) (b.val, AdlerSpec.vals av, AdlerSpec.vals bv) ∧
        out.1.val ≤ 65535 ∧
        (∀ i, AdlerSpec.laneVal out.2.1 i ≤ 65520) ∧ (∀ i, AdlerSpec.laneVal out.2.2 i ≤ 65520) ⦄ := AdlerSpec.outer_loop
#check @AdlerSpec.outer_loop
#print IndependentLemma_5
#print axioms IndependentLemma_5
