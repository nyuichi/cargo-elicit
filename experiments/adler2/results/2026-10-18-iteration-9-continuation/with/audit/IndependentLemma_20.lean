import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_20 : ∀ (self : algo.U32X4) (rhs : U32) (h : ∀ i : Nat, i < 4 → self.val[i]!.val * rhs.val ≤ U32.max), algo.U32X4.Insts.CoreOpsArithMulAssignU32.mul_assign self rhs ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val * rhs.val ⦄ := AdlerProof.mul_wp
#check @AdlerProof.mul_wp
#print IndependentLemma_20
#print axioms IndependentLemma_20
