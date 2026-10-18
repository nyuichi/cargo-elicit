import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_0 : ∀ n a b : Nat, n ≤ 5552 → a ≤ 65520 → b ≤ 65520 → b + n * a + 255 * (n * (n + 1) / 2) < 4294967296 := AdlerProof.lane_bound
#check @AdlerProof.lane_bound
#print IndependentLemma_0
#print axioms IndependentLemma_0
