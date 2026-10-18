import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_4 : ∀ (cs : List (Aeneas.Std.Slice Aeneas.Std.U8)) (rem : Aeneas.Std.Slice Aeneas.Std.U8)
    (a b : ElicitGenerated.algo.U32X4) (t : Nat)
    (hlen : ∀ c ∈ cs, c.val.length = 4)
    (ha : ∀ i, AdlerSpec.laneVal a i ≤ AdlerSpec.aBound t)
    (hb : ∀ i, AdlerSpec.laneVal b i ≤ AdlerSpec.bBound t)
    (ht : t + cs.length ≤ 5552),
    ElicitGenerated.algo.Adler32.compute_loop0_loop0 { chunks := cs, remainder := rem } a b ⦃ out =>
      (AdlerSpec.vals out.1, AdlerSpec.vals out.2) = AdlerSpec.laneModel (AdlerSpec.vals a, AdlerSpec.vals b) cs ∧
      (∀ i, AdlerSpec.laneVal out.1 i ≤ AdlerSpec.aBound (t + cs.length)) ∧
      (∀ i, AdlerSpec.laneVal out.2 i ≤ AdlerSpec.bBound (t + cs.length)) ⦄ := AdlerSpec.inner_loop
#check @AdlerSpec.inner_loop
#print IndependentLemma_4
#print axioms IndependentLemma_4
