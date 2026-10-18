import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_17 : ∀ (it : Aeneas.Std.core.slice.iter.ChunksExact Aeneas.Std.U8) (a b : ElicitGenerated.algo.U32X4), (∀ c ∈ it.chunks, c.val.length = 4) → it.chunks.length ≤ 5552 → (∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520) → (∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520) → ElicitGenerated.algo.Adler32.compute_loop1 it a b ⦃ fun out => ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = AdlerSpec.laneResult a b it.chunks i ⦄ := AdlerSpec.remainder_inner_spec
#check @AdlerSpec.remainder_inner_spec
#print IndependentLemma_17
#print axioms IndependentLemma_17
