import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_9 : ∀ (s : Nat × Nat) (xs : List Aeneas.Std.U8), s.1 ≤ 65520 → s.2 ≤ 65520 → xs.length ≤ 5552 → (xs.foldl AdlerSpec.rawStep s).1 < 4294967296 ∧ (xs.foldl AdlerSpec.rawStep s).2 < 4294967296 := AdlerSpec.lane_chunk_fits_u32
#check @AdlerSpec.lane_chunk_fits_u32
#print IndependentLemma_9
#print axioms IndependentLemma_9
