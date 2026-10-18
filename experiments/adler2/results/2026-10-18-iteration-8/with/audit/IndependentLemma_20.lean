import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_20 : ∀ (a : ElicitGenerated.algo.U32X4) (q : Aeneas.Std.U32), 0 < q.val → ElicitGenerated.algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign a q ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = a.val[i]!.val % q.val ⦄ := AdlerSpec.vector_rem_spec
#check @AdlerSpec.vector_rem_spec
#print IndependentLemma_20
#print axioms IndependentLemma_20
