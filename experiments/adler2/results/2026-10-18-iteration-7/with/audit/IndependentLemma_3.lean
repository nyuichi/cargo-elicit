import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_3 : ∀ (a b : ElicitGenerated.algo.U32X4), (∀ i : Fin 4, (a.val[i.val]'(by simpa using i.isLt)).val + (b.val[i.val]'(by simpa using i.isLt)).val ≤ Aeneas.Std.U32.max) → ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign a b ⦃ out => out.val.map (fun x => x.val) = List.zipWith (fun x y : Aeneas.Std.U32 => x.val + y.val) a.val b.val ⦄ := AdlerSpec.lane_add_spec
#check @AdlerSpec.lane_add_spec
#print IndependentLemma_3
#print axioms IndependentLemma_3
