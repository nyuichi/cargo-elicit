import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_15 : ∀ (it : Aeneas.Std.core.slice.iter.ChunksExact Aeneas.Std.U8) (a b : ElicitGenerated.algo.U32X4), (∀ c ∈ it.chunks, c.val.length = 4) → (∀ i : Nat, i < 4 → (AdlerSpec.laneResult a b it.chunks i).1 < 4294967296 ∧ (AdlerSpec.laneResult a b it.chunks i).2 < 4294967296) → ElicitGenerated.algo.Adler32.compute_loop0_loop0 it a b ⦃ fun out => ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = AdlerSpec.laneResult a b it.chunks i ⦄ := AdlerSpec.inner_spec
#check @AdlerSpec.inner_spec
#print IndependentLemma_15
#print axioms IndependentLemma_15
