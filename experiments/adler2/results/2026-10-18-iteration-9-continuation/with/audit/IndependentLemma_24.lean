import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_24 : ∀ (v : algo.U32X4) (n : Nat) (h : ∀ i : Nat, i < 4 → v.val[i]!.val ≤ n), (v.val.map (fun x => x.val)).sum ≤ 4 * n := AdlerProof.vector_sum_le
#check @AdlerProof.vector_sum_le
#print IndependentLemma_24
#print axioms IndependentLemma_24
