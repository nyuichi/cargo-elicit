import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_15 : ∀ (chunk : Slice U8) (cs : List (Slice U8)) (rem : Slice U8) (a b : U32) (av bv : algo.U32X4) (hlen : chunk.val.length = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535) (hav : ∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520) (hbv : ∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520), algo.Adler32.compute_loop0.body 22208#usize a { chunks := chunk :: cs, remainder := rem } b av bv ⦃ fun r => ∃ (qs : List (Slice U8)) (b' : U32) (av' bv' : algo.U32X4), r = .cont ({ chunks := cs, remainder := rem }, b', av', bv') ∧ (qs.map Slice.val).flatten = chunk.val ∧ (∀ q ∈ qs, q.val.length = 4) ∧ b'.val = (b.val + 22208 * a.val) % 65521 ∧ b'.val ≤ 65520 ∧ (∀ i : Nat, i < 4 → av'.val[i]!.val = (AdlerProof.laneRun qs av.val[i]!.val bv.val[i]!.val i).1 % 65521 ∧ bv'.val[i]!.val = (AdlerProof.laneRun qs av.val[i]!.val bv.val[i]!.val i).2 % 65521 ∧ av'.val[i]!.val ≤ 65520 ∧ bv'.val[i]!.val ≤ 65520) ⦄ := AdlerProof.outer_step
#check @AdlerProof.outer_step
#print IndependentLemma_15
#print axioms IndependentLemma_15
