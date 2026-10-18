import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_12 : ∀ (a b : ElicitGenerated.algo.U32X4), (∀ i : Nat, i < 4 → a.val[i]!.val + b.val[i]!.val ≤ 4294967295) → ElicitGenerated.algo.U32X4.Insts.CoreOpsArithAddAssignU32X4.add_assign a b ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = a.val[i]!.val + b.val[i]!.val ⦄ := AdlerSpec.vector_add_spec
#check @AdlerSpec.vector_add_spec
#print IndependentLemma_12
#print axioms IndependentLemma_12
