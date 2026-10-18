import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_14 : ∀ (self : algo.U32X4), algo.U32X4.Insts.CoreOpsArithRemAssignU32.rem_assign self algo.Adler32.compute.MOD ⦃ fun out => ∀ i : Nat, i < 4 → out.val[i]!.val = self.val[i]!.val % 65521 ∧ out.val[i]!.val ≤ 65520 ⦄ := AdlerProof.rem_mod_wp
#check @AdlerProof.rem_mod_wp
#print IndependentLemma_14
#print axioms IndependentLemma_14
