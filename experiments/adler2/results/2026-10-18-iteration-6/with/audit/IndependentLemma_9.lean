import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_9 : ∀  (chunks : List (Slice U8)) (remainder : Slice U8)
     (a b : algo.U32X4) (hs : AdlerSpec.safeLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b)),
     algo.Adler32.compute_loop0_loop0 ⟨chunks, remainder⟩ a b ⦃ av bv =>
       (AdlerSpec.lane av, AdlerSpec.lane bv) =
         AdlerSpec.rawLanes chunks (AdlerSpec.lane a) (AdlerSpec.lane b) ⦄ := AdlerProof.inner_loop_spec
#check @AdlerProof.inner_loop_spec
#print IndependentLemma_9
#print axioms IndependentLemma_9
