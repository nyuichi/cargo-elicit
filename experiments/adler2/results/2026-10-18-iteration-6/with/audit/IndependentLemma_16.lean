import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_16 : ∀  (iter : core.slice.iter.ChunksExact U8)
     (a b : algo.U32X4) (hn : iter.chunks.length ≤ 5552)
     (hc : ∀ c ∈ iter.chunks, 4 ≤ c.length)
     (ha : ∀ i, AdlerSpec.lane a i ≤ 65520)
     (hb : ∀ i, AdlerSpec.lane b i ≤ 65520),
     algo.Adler32.compute_loop1 iter a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes iter.chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := AdlerProof.remainder_iter_rule
#check @AdlerProof.remainder_iter_rule
#print IndependentLemma_16
#print axioms IndependentLemma_16
