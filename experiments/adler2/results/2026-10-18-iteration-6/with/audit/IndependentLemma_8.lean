import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_8 : ∀  (chunk remainder : Slice U8) (chunks : List (Slice U8))
     (a b : algo.U32X4) (hc : 4 ≤ chunk.length)
     (ha : ∀ i : Fin 4, AdlerSpec.lane a i + chunk.val[i.val]!.val ≤ U32.max)
     (hb : ∀ i : Fin 4, AdlerSpec.lane b i + (AdlerSpec.lane a i + chunk.val[i.val]!.val) ≤ U32.max),
     algo.Adler32.compute_loop0_loop0.body ⟨chunk :: chunks, remainder⟩ a b ⦃ r =>
       ∃ av bv, r = ControlFlow.cont (⟨chunks, remainder⟩, av, bv) ∧
       ∀ i : Fin 4,
         AdlerSpec.lane av i = AdlerSpec.lane a i + chunk.val[i.val]!.val ∧
         AdlerSpec.lane bv i = AdlerSpec.lane b i + (AdlerSpec.lane a i + chunk.val[i.val]!.val) ⦄ := AdlerProof.inner_step_spec
#check @AdlerProof.inner_step_spec
#print IndependentLemma_8
#print axioms IndependentLemma_8
