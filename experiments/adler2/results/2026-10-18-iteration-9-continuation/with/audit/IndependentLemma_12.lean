import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_12 : ∀ (cs : List (Slice U8)) (rem source : Slice U8) (hpartition : (cs.map Slice.val).flatten ++ rem.val = source.val) (hfull : ∀ c ∈ cs, c.val.length = 4) (hlen : source.val.length ≤ 22208), cs.length ≤ 5552 := AdlerProof.chunk_count_bound
#check @AdlerProof.chunk_count_bound
#print IndependentLemma_12
#print axioms IndependentLemma_12
