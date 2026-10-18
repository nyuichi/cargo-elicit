import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_1 : ∀ (k : Nat), k ≤ 5552 → 65520 + k * 65520 + 255 * k * (k + 1) / 2 < 4294967296 := AdlerProof.lane_budget
#check @AdlerProof.lane_budget
#print IndependentLemma_1
#print axioms IndependentLemma_1
