import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_11 : ∀  (chunks : List (Slice U8)) (k : Nat) (a b : Fin 4 → Nat)
     (hlen : k + chunks.length ≤ 5552)
     (hc : ∀ c ∈ chunks, 4 ≤ c.length)
     (ha : ∀ i, a i ≤ 65520 + k * 255)
     (hb : ∀ i, b i ≤ 65520 + k * 65520 + 255 * (k * (k + 1) / 2)),
     AdlerSpec.safeLanes chunks a b := AdlerProof.safe_lanes_bound
#check @AdlerProof.safe_lanes_bound
#print IndependentLemma_11
#print axioms IndependentLemma_11
