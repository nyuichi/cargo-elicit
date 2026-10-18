import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_21 : ∀ (a : ElicitGenerated.algo.U32X4) (q : Aeneas.Std.U32), (∀ i : Nat, i < 4 → a.val[i]!.val * q.val ≤ 4294967295) → ElicitGenerated.algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign a q ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = a.val[i]!.val * q.val ⦄ := AdlerSpec.vector_mul_spec
#check @AdlerSpec.vector_mul_spec
#print IndependentLemma_21
#print axioms IndependentLemma_21
