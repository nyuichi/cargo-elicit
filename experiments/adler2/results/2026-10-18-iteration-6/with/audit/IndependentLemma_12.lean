import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_12 : ∀  (chunks : List (Slice U8)) (remainder : Slice U8)
     (a b : algo.U32X4) (hn : chunks.length ≤ 5552)
     (hc : ∀ c ∈ chunks, 4 ≤ c.length)
     (ha : ∀ i, AdlerSpec.lane a i ≤ 65520)
     (hb : ∀ i, AdlerSpec.lane b i ≤ 65520),
     algo.Adler32.compute_loop0_loop0 ⟨chunks, remainder⟩ a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := AdlerProof.inner_loop_bounded
#check @AdlerProof.inner_loop_bounded
#print IndependentLemma_12
#print axioms IndependentLemma_12
