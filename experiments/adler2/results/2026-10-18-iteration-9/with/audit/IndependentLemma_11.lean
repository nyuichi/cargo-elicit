import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_11 : ∀ (cs : List (Slice U8)) (rem : Slice U8) (a b : algo.U32X4) (hchunks : ∀ c ∈ cs, 4 ≤ c.val.length) (hlen : cs.length ≤ 5552) (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520) (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520), algo.Adler32.compute_loop1 { chunks := cs, remainder := rem } a b ⦃ fun out => ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = AdlerProof.laneRun cs a.val[i]!.val b.val[i]!.val i ⦄ := AdlerProof.remainder_inner_loop_bounded
#check @AdlerProof.remainder_inner_loop_bounded
#print IndependentLemma_11
#print axioms IndependentLemma_11
