import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_30 : ∀ (v : algo.U32X4), AdlerProof.iterSum { slice := Array.to_slice v, i := 0 } = v.val[0]!.val + v.val[1]!.val + v.val[2]!.val + v.val[3]!.val := AdlerProof.vector_iter_sum
#check @AdlerProof.vector_iter_sum
#print IndependentLemma_30
#print axioms IndependentLemma_30
