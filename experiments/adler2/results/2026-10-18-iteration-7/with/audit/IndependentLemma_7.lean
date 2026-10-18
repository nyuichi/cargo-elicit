import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_7 : ∀ (a : ElicitGenerated.algo.U32X4) (q : Aeneas.Std.U32)
    (h : ∀ i, AdlerSpec.laneVal a i * q.val ≤ Aeneas.Std.U32.max),
    ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign a q ⦃ out =>
      AdlerSpec.vals out = (AdlerSpec.vals a).map (fun x => x*q.val) ⦄ := AdlerSpec.lane_mul_spec
#check @AdlerSpec.lane_mul_spec
#print IndependentLemma_7
#print axioms IndependentLemma_7
