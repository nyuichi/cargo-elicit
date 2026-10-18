import Lemmas
set_option pp.fullNames true
open Aeneas Aeneas.Std Result ElicitGenerated
theorem IndependentLemma_19 : ∀ (size : Usize) (it : core.slice.iter.ChunksExact U8) (a b : U32) (av bv : algo.U32X4) (hsize : size.val = 22208) (hchunks : ∀ c ∈ it.chunks, c.val.length = 22208) (ha : a.val ≤ 65535) (hb : b.val ≤ 65535) (hav : ∀ i : Nat, i < 4 → av.val[i]!.val ≤ 65520) (hbv : ∀ i : Nat, i < 4 → bv.val[i]!.val ≤ 65520), algo.Adler32.compute_loop0 size it a b av bv ⦃ fun out => ∃ qs : List (Slice U8), (qs.map Slice.val).flatten = (it.chunks.map Slice.val).flatten ∧ (∀ q ∈ qs, q.val.length = 4) ∧ out.1.val % 65521 = (b.val + 22208 * it.chunks.length * a.val) % 65521 ∧ out.1.val ≤ 65535 ∧ (∀ i : Nat, i < 4 → out.2.1.val[i]!.val = (AdlerProof.laneRun qs av.val[i]!.val bv.val[i]!.val i).1 % 65521 ∧ out.2.2.val[i]!.val = (AdlerProof.laneRun qs av.val[i]!.val bv.val[i]!.val i).2 % 65521 ∧ out.2.1.val[i]!.val ≤ 65520 ∧ out.2.2.val[i]!.val ≤ 65520) ⦄ := AdlerProof.outer_iter_wp
#check @AdlerProof.outer_iter_wp
#print IndependentLemma_19
#print axioms IndependentLemma_19
