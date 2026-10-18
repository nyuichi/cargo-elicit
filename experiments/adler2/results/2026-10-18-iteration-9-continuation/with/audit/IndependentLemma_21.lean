import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_21 : ∀ (it : core.slice.iter.ChunksExact U8) (a b : algo.U32X4) (hc : ∀ c ∈ it.chunks, 4 ≤ c.val.length) (hn : it.chunks.length ≤ 5552) (ha : ∀ i : Nat, i < 4 → a.val[i]!.val ≤ 65520) (hb : ∀ i : Nat, i < 4 → b.val[i]!.val ≤ 65520), algo.Adler32.compute_loop1 it a b ⦃ fun out => ∀ i : Nat, i < 4 → (out.1.val[i]!.val, out.2.val[i]!.val) = AdlerProof.laneRun it.chunks a.val[i]!.val b.val[i]!.val i ⦄ := AdlerProof.final_inner_iter_wp
#check @AdlerProof.final_inner_iter_wp
#print IndependentLemma_21
#print axioms IndependentLemma_21
