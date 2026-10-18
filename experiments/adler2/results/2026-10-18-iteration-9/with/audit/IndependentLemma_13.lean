import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_13 : ∀ (self : algo.U32X4) (q : U32), 0 < q.val → algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign self q ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val % q.val ⦄ := AdlerProof.rem_wp
#check @AdlerProof.rem_wp
#print IndependentLemma_13
#print axioms IndependentLemma_13
